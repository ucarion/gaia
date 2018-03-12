#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate serde_derive;

extern crate geo;
extern crate geojson;
extern crate serde;
extern crate serde_json;
extern crate tempdir;

use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

use geojson::GeoJson;
use geojson::conversion::TryInto;
use geo::boundingbox::BoundingBox;
use geo::simplifyvw::SimplifyVW;
use tempdir::TempDir;

mod errors;
mod imagemagick;

use errors::*;
use imagemagick::Convert;

pub const ELEVATION_OFFSET: u16 = 500;

/// The maximum level of detail to be created. This value plus one is how many levels exist,
/// because the least-detailed level is zero.
pub const MAX_LEVEL: u8 = 6;

/// The size of satellite imagery tiles. This should be equal to the size of a NASA tile (21600)
/// divided by how many tiles are to be made from them (32 -- a quarter of
/// TILES_ACROSS_MAX_LEVEL_WIDTH).
///
/// (21600 / 32 = 675).
pub const IMAGERY_TILE_SIZE: u32 = 675;

/// How many tiles at the greatest level of detail are to be created from a NASA tile. A NASA tile
/// is a quarter of the width of the world, and there are TILES_ACROSS_MAX_LEVEL_WIDTH = 128 tiles
/// across the width.  Thus, there are 128/4 = 32 tiles across a single NASA tile.
const TILES_ACROSS_NASA_TILE: u32 = 32;

/// The size of an elevation tile.
///
/// This should be a power of two plus one, so that it can be combined with another tile that has
/// one column of overlap to produce another tile of power of two plus one.
pub const ELEVATION_TILE_SIZE: u32 = 129;

/// The size of an elevation crop. This should be ELEVATION_TILE_SIZE - 1.
const ELEVATION_CROP_SIZE: u32 = 128;

/// How many *crops* -- not tiles -- to generate from the width of a single NOAA tile.
const CROPS_ACROSS_NOAA_TILE: u32 = 32;

/// Prior to creating the crops, what size to resize NOAA tiles to. This is CROPS_ACROSS_NOAA_TILE
/// * ELEVATION_CROP_SIZE.
const NOAA_TILE_RESIZE_WIDTH: u32 = 4096;

/// The metadata for the tile metadata on the first pass; only maxima and minima, computed by
/// Imagemagick, are present. These elevations are correct -- they do not need to be offset by
/// ELEVATION_OFFSET.
#[derive(Serialize, Deserialize, Debug)]
struct FirstPassTileMetadata {
    min_elevation: u16,
    max_elevation: u16,
}

pub type Properties = serde_json::Map<String, serde_json::Value>;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TileMetadata {
    pub min_elevation: u16,
    pub max_elevation: u16,
    pub polygons: Vec<u64>,
    pub points: Vec<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FeaturesData {
    pub polygons: Vec<MultiLevelPolygon>,
    pub points: Vec<MultiLevelPoint>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MultiLevelPolygon {
    pub properties: Properties,
    /// `(min, max)` along x- and y-axis
    pub bounding_box: [(f32, f32); 2],
    /// The same polygon simplified according to epsilons in `simplification_epsilons`.
    pub levels: Vec<Vec<(f32, f32)>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MultiLevelPoint {
    pub properties: Properties,
    pub coordinates: [f32; 2],
    pub levels: Vec<f32>,
}

#[derive(Debug)]
pub struct PrepareAssetsTask {
    noaa_globe_dir: PathBuf,
    nasa_blue_marble_dir: PathBuf,
    polygons_file: PathBuf,
    points_file: PathBuf,
    simplification_epsilons: [f32; MAX_LEVEL as usize + 1],
    output_dir: PathBuf,
}

impl PrepareAssetsTask {
    pub fn new() -> PrepareAssetsTask {
        PrepareAssetsTask {
            noaa_globe_dir: "".into(),
            nasa_blue_marble_dir: "".into(),
            polygons_file: "".into(),
            points_file: "".into(),
            simplification_epsilons: [0.0; MAX_LEVEL as usize + 1],
            output_dir: "".into(),
        }
    }

    pub fn with_noaa_globe_dir(self, noaa_globe_dir: PathBuf) -> PrepareAssetsTask {
        PrepareAssetsTask {
            noaa_globe_dir,
            ..self
        }
    }

    pub fn with_nasa_blue_marble_dir(self, nasa_blue_marble_dir: PathBuf) -> PrepareAssetsTask {
        PrepareAssetsTask {
            nasa_blue_marble_dir,
            ..self
        }
    }

    pub fn with_polygons_file(self, polygons_file: PathBuf) -> PrepareAssetsTask {
        PrepareAssetsTask {
            polygons_file,
            ..self
        }
    }

    pub fn with_points_file(self, points_file: PathBuf) -> PrepareAssetsTask {
        PrepareAssetsTask {
            points_file,
            ..self
        }
    }

    pub fn with_output_dir(self, output_dir: PathBuf) -> PrepareAssetsTask {
        PrepareAssetsTask { output_dir, ..self }
    }

    pub fn with_simplification_epsilons(
        self,
        simplification_epsilons: [f32; MAX_LEVEL as usize + 1],
    ) -> PrepareAssetsTask {
        PrepareAssetsTask {
            simplification_epsilons,
            ..self
        }
    }

    pub fn run(&self) -> Result<()> {
        if !self.tiles_dir().is_dir() {
            fs::create_dir(&self.tiles_dir()).chain_err(|| "Could not create tiles directory")?;
        }

        self.create_nasa_max_level_tiles()?;
        for level in (0..MAX_LEVEL).rev() {
            self.create_nasa_level(level)?;
        }

        let temp_crop_dir =
            TempDir::new(&format!("crops")).chain_err(|| "Error creating temporary dir")?;

        self.create_noaa_max_level_crops(&temp_crop_dir.path())?;
        self.create_noaa_level(&temp_crop_dir.path(), MAX_LEVEL)?;
        for level in (0..MAX_LEVEL).rev() {
            self.create_noaa_crop_level(&temp_crop_dir.path(), level)?;
            self.create_noaa_level(&temp_crop_dir.path(), level)?;
        }

        self.create_polygon_data()?;

        Ok(())
    }

    fn create_nasa_max_level_tiles(&self) -> Result<()> {
        let test_file = format!("{}_0_0.jpg", MAX_LEVEL);
        if self.tiles_dir().join(test_file).exists() {
            return Ok(());
        }

        self.create_nasa_max_level_tile("A1", TILES_ACROSS_NASA_TILE * 0, TILES_ACROSS_NASA_TILE)?;
        self.create_nasa_max_level_tile("A2", TILES_ACROSS_NASA_TILE * 0, 0)?;
        self.create_nasa_max_level_tile("B1", TILES_ACROSS_NASA_TILE * 1, TILES_ACROSS_NASA_TILE)?;
        self.create_nasa_max_level_tile("B2", TILES_ACROSS_NASA_TILE * 1, 0)?;
        self.create_nasa_max_level_tile("C1", TILES_ACROSS_NASA_TILE * 2, TILES_ACROSS_NASA_TILE)?;
        self.create_nasa_max_level_tile("C2", TILES_ACROSS_NASA_TILE * 2, 0)?;
        self.create_nasa_max_level_tile("D1", TILES_ACROSS_NASA_TILE * 3, TILES_ACROSS_NASA_TILE)?;
        self.create_nasa_max_level_tile("D2", TILES_ACROSS_NASA_TILE * 3, 0)?;

        Ok(())
    }

    fn create_nasa_max_level_tile(
        &self,
        nasa_tile: &str,
        x_offset: u32,
        y_offset: u32,
    ) -> Result<()> {
        let temp_dir = TempDir::new(&format!("nasa_{}", nasa_tile))
            .chain_err(|| "Error creating temporary dir")?;

        let file_name = format!("world.topo.bathy.200412.3x21600x21600.{}.jpg", nasa_tile);

        Convert::new()
            .monitor()
            .input(&self.nasa_blue_marble_dir.join(file_name))
            .crops(IMAGERY_TILE_SIZE)
            .output(&temp_dir.path().join("out.jpg"))
            .run()?;

        for x in 0..TILES_ACROSS_NASA_TILE {
            for y in 0..TILES_ACROSS_NASA_TILE {
                let inverted_y = TILES_ACROSS_NASA_TILE - 1 - y;
                let crop_filename = format!("out-{}.jpg", inverted_y * TILES_ACROSS_NASA_TILE + x);
                let crop_path = temp_dir.path().join(crop_filename);

                let tile_filename = format!("{}_{}_{}.jpg", MAX_LEVEL, x_offset + x, y_offset + y);
                let tile_path = self.tiles_dir().join(tile_filename);

                fs::copy(crop_path, tile_path).chain_err(|| "Error copying NASA tile crop")?;
            }
        }

        Ok(())
    }

    fn create_noaa_max_level_crops(&self, temp_crop_dir: &Path) -> Result<()> {
        let test_file = format!("{}_0_0.pgm", MAX_LEVEL);
        if self.tiles_dir().join(test_file).exists() {
            return Ok(());
        }

        self.create_noaa_max_level_tile(
            temp_crop_dir,
            true,
            "a10g",
            "e10g",
            0 * CROPS_ACROSS_NOAA_TILE,
            1 * CROPS_ACROSS_NOAA_TILE,
        )?;
        self.create_noaa_max_level_tile(
            temp_crop_dir,
            true,
            "b10g",
            "f10g",
            1 * CROPS_ACROSS_NOAA_TILE,
            1 * CROPS_ACROSS_NOAA_TILE,
        )?;
        self.create_noaa_max_level_tile(
            temp_crop_dir,
            true,
            "c10g",
            "g10g",
            2 * CROPS_ACROSS_NOAA_TILE,
            1 * CROPS_ACROSS_NOAA_TILE,
        )?;
        self.create_noaa_max_level_tile(
            temp_crop_dir,
            true,
            "d10g",
            "h10g",
            3 * CROPS_ACROSS_NOAA_TILE,
            1 * CROPS_ACROSS_NOAA_TILE,
        )?;
        self.create_noaa_max_level_tile(
            temp_crop_dir,
            false,
            "i10g",
            "m10g",
            0 * CROPS_ACROSS_NOAA_TILE,
            0 * CROPS_ACROSS_NOAA_TILE,
        )?;
        self.create_noaa_max_level_tile(
            temp_crop_dir,
            false,
            "j10g",
            "n10g",
            1 * CROPS_ACROSS_NOAA_TILE,
            0 * CROPS_ACROSS_NOAA_TILE,
        )?;
        self.create_noaa_max_level_tile(
            temp_crop_dir,
            false,
            "k10g",
            "o10g",
            2 * CROPS_ACROSS_NOAA_TILE,
            0 * CROPS_ACROSS_NOAA_TILE,
        )?;
        self.create_noaa_max_level_tile(
            temp_crop_dir,
            false,
            "l10g",
            "p10g",
            3 * CROPS_ACROSS_NOAA_TILE,
            0 * CROPS_ACROSS_NOAA_TILE,
        )?;

        Ok(())
    }

    fn create_noaa_max_level_tile(
        &self,
        temp_crop_dir: &Path,
        is_north: bool,
        top_tile: &str,
        bottom_tile: &str,
        x_offset: u32,
        y_offset: u32,
    ) -> Result<()> {
        let (top_height, bottom_height) = if is_north { (4800, 6000) } else { (6000, 4800) };

        Convert::new()
            .monitor()
            .grayscale_input((10800, top_height), &self.noaa_globe_dir.join(top_tile))
            .grayscale_input(
                (10800, bottom_height),
                &self.noaa_globe_dir.join(bottom_tile),
            )
            .offset_each_pixel(ELEVATION_OFFSET)
            .append_vertically()
            .resize(&format!("{}", NOAA_TILE_RESIZE_WIDTH))
            .output(&temp_crop_dir.join("out.pgm"))
            .run()?;

        Convert::new()
            .monitor()
            .input(&temp_crop_dir.join("out.pgm"))
            .crops(ELEVATION_CROP_SIZE)
            .adjoin()
            .output(&temp_crop_dir.join("out.pgm"))
            .run()?;

        for x in 0..CROPS_ACROSS_NOAA_TILE {
            for y in 0..CROPS_ACROSS_NOAA_TILE {
                let inverted_y = CROPS_ACROSS_NOAA_TILE - 1 - y;
                let crop_filename = format!("out-{}.pgm", inverted_y * TILES_ACROSS_NASA_TILE + x);
                let crop_path = temp_crop_dir.join(crop_filename);

                let out_crop_filename =
                    format!("{}_{}_{}.pgm", MAX_LEVEL, x_offset + x, y_offset + y);
                let out_crop_path = temp_crop_dir.join(out_crop_filename);

                fs::copy(crop_path, out_crop_path).chain_err(|| "Error copying NASA tile crop")?;
            }
        }

        Ok(())
    }

    fn create_noaa_crop_level(&self, temp_crop_dir: &Path, level: u8) -> Result<()> {
        let test_file = format!("{}_0_0.pgm", level);
        if self.tiles_dir().join(test_file).exists() {
            return Ok(());
        }

        let tiles_across_width = 2u32.pow(1 + level as u32);
        let tiles_across_height = 2u32.pow(level as u32);

        for x in 0..tiles_across_width {
            for y in 0..tiles_across_height {
                let crop = format!("{}_{}_{}.pgm", level, x, y);

                let top_y = y * 2 + 1;
                let bottom_y = y * 2;
                let left_x = x * 2;
                let right_x = x * 2 + 1;

                let top_left = format!("{}_{}_{}.pgm", level + 1, left_x, top_y);
                let top_right = format!("{}_{}_{}.pgm", level + 1, right_x, top_y);
                let bottom_left = format!("{}_{}_{}.pgm", level + 1, left_x, bottom_y);
                let bottom_right = format!("{}_{}_{}.pgm", level + 1, right_x, bottom_y);

                Convert::new()
                    .group(|convert| {
                        convert
                            .input(&temp_crop_dir.join(top_left))
                            .input(&temp_crop_dir.join(top_right))
                            .append_horizontally()
                    })
                    .group(|convert| {
                        convert
                            .input(&temp_crop_dir.join(bottom_left))
                            .input(&temp_crop_dir.join(bottom_right))
                            .append_horizontally()
                    })
                    .append_vertically()
                    .resize("50%")
                    .output(&temp_crop_dir.join(crop))
                    .run()?;
            }
        }

        Ok(())
    }

    fn create_noaa_level(&self, temp_crop_dir: &Path, level: u8) -> Result<()> {
        let test_file = format!("{}_0_0.pgm", level);
        if self.tiles_dir().join(test_file).exists() {
            return Ok(());
        }

        let tiles_across_width = 2u32.pow(1 + level as u32);
        let tiles_across_height = 2u32.pow(level as u32);

        for x in 0..tiles_across_width {
            for y in 0..tiles_across_height {
                let tile = format!("{}_{}_{}.pgm", level, x, y);
                let gray_tile = format!("{}_{}_{}.gray", level, x, y);

                // copy from bottom and right -- if you're on the bottom row, use your own bottom
                // row as your "beneath's top row".
                let (below_y, below_offset_y) = if y == 0 {
                    (0, ELEVATION_CROP_SIZE - 1)
                } else {
                    (y - 1, 0)
                };

                let right_x = if x == tiles_across_width - 1 {
                    0
                } else {
                    x + 1
                };

                let main_crop = format!("{}_{}_{}.pgm", level, x, y);
                let right_crop = format!("{}_{}_{}.pgm", level, right_x, y);
                let below_crop = format!("{}_{}_{}.pgm", level, x, below_y);
                let below_right_crop = format!("{}_{}_{}.pgm", level, right_x, below_y);

                let right_crop_size = (1, ELEVATION_CROP_SIZE);
                let right_crop_offset = (0, 0);
                let below_crop_size = (ELEVATION_CROP_SIZE, 1);
                let below_crop_offset = (0, below_offset_y);
                let below_right_crop_size = (1, 1);
                let below_right_crop_offset = (0, below_offset_y);

                let (min, max) = Convert::new()
                    .group(|convert| {
                        // Top row
                        convert
                            .input(&temp_crop_dir.join(main_crop))
                            .group(|convert| {
                                convert
                                    .input(&temp_crop_dir.join(right_crop))
                                    .crop_one(right_crop_size, right_crop_offset)
                            })
                            .append_horizontally()
                    })
                    .group(|convert| {
                        // Bottom row
                        convert
                            .group(|convert| {
                                convert
                                    .input(&temp_crop_dir.join(below_crop))
                                    .crop_one(below_crop_size, below_crop_offset)
                            })
                            .group(|convert| {
                                convert
                                    .input(&temp_crop_dir.join(below_right_crop))
                                    .crop_one(below_right_crop_size, below_right_crop_offset)
                            })
                            .append_horizontally()
                    })
                    .append_vertically()
                    .report_max_min()
                    .output(&self.tiles_dir().join(&tile))
                    .run_with_max_min()?;

                let min_elevation =
                    ((min * u16::max_value() as f32) as u16).saturating_sub(ELEVATION_OFFSET);
                let max_elevation =
                    ((max * u16::max_value() as f32) as u16).saturating_sub(ELEVATION_OFFSET);

                let metadata = FirstPassTileMetadata {
                    min_elevation,
                    max_elevation,
                };

                let metadata_path = format!("{}_{}_{}-first-pass.json", level, x, y);
                let metadata_file = File::create(self.tiles_dir().join(&metadata_path))
                    .chain_err(|| "Error creating first-pass metadata file")?;

                serde_json::to_writer(metadata_file, &metadata)
                    .chain_err(|| "Error writing out first-pass metadata file")?;

                Convert::new()
                    .input(&self.tiles_dir().join(&tile))
                    .depth(16)
                    .output(&self.tiles_dir().join(&gray_tile))
                    .run()?;
            }
        }

        Ok(())
    }

    fn create_nasa_level(&self, level: u8) -> Result<()> {
        let test_file = format!("{}_0_0.jpg", level);
        if self.tiles_dir().join(test_file).exists() {
            return Ok(());
        }

        let tiles_across_width = 2u32.pow(1 + level as u32);
        let tiles_across_height = 2u32.pow(level as u32);

        for x in 0..tiles_across_width {
            for y in 0..tiles_across_height {
                let tile = format!("{}_{}_{}.jpg", level, x, y);

                let top_y = y * 2 + 1;
                let bottom_y = y * 2;
                let left_x = x * 2;
                let right_x = x * 2 + 1;

                let top_left = format!("{}_{}_{}.jpg", level + 1, left_x, top_y);
                let top_right = format!("{}_{}_{}.jpg", level + 1, right_x, top_y);
                let bottom_left = format!("{}_{}_{}.jpg", level + 1, left_x, bottom_y);
                let bottom_right = format!("{}_{}_{}.jpg", level + 1, right_x, bottom_y);

                Convert::new()
                    .group(|convert| {
                        convert
                            .input(&self.tiles_dir().join(top_left))
                            .input(&self.tiles_dir().join(top_right))
                            .append_horizontally()
                    })
                    .group(|convert| {
                        convert
                            .input(&self.tiles_dir().join(bottom_left))
                            .input(&self.tiles_dir().join(bottom_right))
                            .append_horizontally()
                    })
                    .append_vertically()
                    .resize("50%")
                    .output(&self.tiles_dir().join(tile))
                    .run()?;
            }
        }

        Ok(())
    }

    fn create_polygon_data(&self) -> Result<()> {
        if self.output_dir.join("features.json").exists() {
            return Ok(());
        }

        let (polygons, polygon_properties) = self.load_polygons()?;
        let (points, point_properties) = self.load_points()?;

        self.create_final_metadata(&polygons, &points)?;
        self.create_features_file(polygons, polygon_properties, points, point_properties)?;

        Ok(())
    }

    fn load_polygons(&self) -> Result<(Vec<geo::Polygon<f32>>, Vec<Properties>)> {
        let mut features_file =
            File::open(&self.polygons_file).chain_err(|| "Could not open features file")?;

        let mut geojson = String::new();
        features_file
            .read_to_string(&mut geojson)
            .chain_err(|| "Could not read features file to string")?;

        let geojson: GeoJson = geojson
            .parse()
            .chain_err(|| "Error parsing features file as GeoJson")?;

        let feature_collection = match geojson {
            GeoJson::FeatureCollection(fc) => Ok(fc),
            _ => Err("Features file was not a GeoJson FeatureCollection at the top level"),
        }?;

        let mut polygons = Vec::new();
        let mut polygon_properties = Vec::new();

        for feature in feature_collection.features {
            let properties = feature.properties.unwrap_or(serde_json::Map::new());
            let geometry: geo::Geometry<f32> = feature.geometry.unwrap().value.try_into().unwrap();

            let feature_polygons = match geometry {
                geo::Geometry::Polygon(polygon) => vec![polygon],
                geo::Geometry::MultiPolygon(multi_polygon) => multi_polygon.0,
                _ => panic!("Feature file contained something other than Polygon and MultiPolygon"),
            };

            polygon_properties.extend_from_slice(&vec![properties.clone(); feature_polygons.len()]);
            polygons.extend_from_slice(&feature_polygons);
        }

        Ok((polygons, polygon_properties))
    }

    fn load_points(&self) -> Result<(Vec<geo::Point<f32>>, Vec<Properties>)> {
        let mut points_file =
            File::open(&self.points_file).chain_err(|| "Could not open points file")?;

        let mut geojson = String::new();
        points_file
            .read_to_string(&mut geojson)
            .chain_err(|| "Could not read points file to string")?;

        let geojson: GeoJson = geojson
            .parse()
            .chain_err(|| "Error parsing points file as GeoJson")?;

        let feature_collection = match geojson {
            GeoJson::FeatureCollection(fc) => Ok(fc),
            _ => Err("Features file was not a GeoJson FeatureCollection at the top level"),
        }?;

        let mut points = Vec::new();
        let mut point_properties = Vec::new();

        for feature in feature_collection.features {
            let properties = feature.properties.unwrap_or(serde_json::Map::new());
            let geometry: geo::Geometry<f32> = feature.geometry.unwrap().value.try_into().unwrap();

            let feature_point = match geometry {
                geo::Geometry::Point(point) => point,
                _ => panic!("Feature file contained something other than Polygon and MultiPolygon"),
            };

            points.push(feature_point);
            point_properties.push(properties);
        }

        Ok((points, point_properties))
    }

    fn create_final_metadata(
        &self,
        polygons: &[geo::Polygon<f32>],
        points: &[geo::Point<f32>],
    ) -> Result<()> {
        for level in 0..MAX_LEVEL + 1 {
            self.create_final_metadata_level(polygons, points, level)?;
        }

        Ok(())
    }

    fn create_final_metadata_level(
        &self,
        polygons: &[geo::Polygon<f32>],
        points: &[geo::Point<f32>],
        level: u8,
    ) -> Result<()> {
        let tiles_across_width = 2u32.pow(1 + level as u32);
        let tiles_across_height = 2u32.pow(level as u32);

        let mut tile_polygons = BTreeMap::new();
        let mut tile_points = BTreeMap::new();

        for x in 0..tiles_across_width {
            for y in 0..tiles_across_height {
                tile_polygons.insert((x, y), Vec::new());
                tile_points.insert((x, y), Vec::new());
            }
        }

        for (polygon_index, polygon) in polygons.iter().enumerate() {
            let bounding_box = polygon.exterior.bbox().unwrap();
            let x_min = tiles_across_width as f32 * self.map_x_coord(bounding_box.xmin);
            let x_max = tiles_across_width as f32 * self.map_x_coord(bounding_box.xmax);
            let y_min = tiles_across_height as f32 * self.map_y_coord(bounding_box.ymin);
            let y_max = tiles_across_height as f32 * self.map_y_coord(bounding_box.ymax);

            for x in x_min.floor() as u32..x_max.ceil() as u32 {
                for y in y_min.floor() as u32..y_max.ceil() as u32 {
                    tile_polygons
                        .get_mut(&(x, y))
                        .unwrap()
                        .push(polygon_index as u64);
                }
            }
        }

        for (point_index, point) in points.iter().enumerate() {
            let x = (tiles_across_width as f32 * self.map_x_coord(point.x())).floor() as u32;
            let y = (tiles_across_height as f32 * self.map_y_coord(point.y())).floor() as u32;

            tile_points
                .get_mut(&(x, y))
                .unwrap()
                .push(point_index as u64);
        }

        for x in 0..tiles_across_width {
            for y in 0..tiles_across_height {
                let polygon_indices = tile_polygons.get(&(x, y)).unwrap();
                let point_indices = tile_points.get(&(x, y)).unwrap();

                let first_pass_path = format!("{}_{}_{}-first-pass.json", level, x, y);
                let first_pass_file = File::open(self.tiles_dir().join(first_pass_path))
                    .chain_err(|| "Error opening first-pass metadata file")?;

                let first_pass_metadata: FirstPassTileMetadata =
                    serde_json::from_reader(first_pass_file)
                        .chain_err(|| "Error parsing first-pass metadata")?;

                let tile_metadata = TileMetadata {
                    min_elevation: first_pass_metadata.min_elevation,
                    max_elevation: first_pass_metadata.max_elevation,
                    polygons: polygon_indices.clone(),
                    points: point_indices.clone(),
                };

                let metadata_path = format!("{}_{}_{}.json", level, x, y);
                let metadata_file = File::create(self.tiles_dir().join(metadata_path))
                    .chain_err(|| "Error creating metadata file")?;

                serde_json::to_writer(metadata_file, &tile_metadata)
                    .chain_err(|| "Error writing out metadata file")?;
            }
        }

        Ok(())
    }

    fn create_features_file(
        &self,
        polygons: Vec<geo::Polygon<f32>>,
        polygon_properties: Vec<Properties>,
        points: Vec<geo::Point<f32>>,
        point_properties: Vec<Properties>,
    ) -> Result<()> {
        let polygons: Vec<_> = polygons
            .into_iter()
            .zip(polygon_properties)
            .map(|(polygon, properties)| {
                let bounding_box = polygon.bbox().unwrap();
                let bounding_box = [
                    (
                        self.map_x_coord(bounding_box.xmin),
                        self.map_x_coord(bounding_box.xmax),
                    ),
                    (
                        self.map_y_coord(bounding_box.ymin),
                        self.map_y_coord(bounding_box.ymax),
                    ),
                ];

                let levels = (0..MAX_LEVEL + 1)
                    .map(|level| {
                        let simplified_polygon =
                            polygon.simplifyvw(&self.simplification_epsilons[level as usize]);
                        let interior_points =
                            simplified_polygon.interiors.into_iter().flat_map(|line| {
                                line.into_iter().map(|point| {
                                    (self.map_x_coord(point.x()), self.map_y_coord(point.y()))
                                })
                            });

                        let exterior_points = simplified_polygon.exterior.into_iter().map(
                            |point| (self.map_x_coord(point.x()), self.map_y_coord(point.y())),
                        );

                        exterior_points.chain(interior_points).collect()
                    })
                    .collect();

                MultiLevelPolygon {
                    properties,
                    bounding_box,
                    levels,
                }
            })
            .collect();

        let points: Vec<_> = points
            .into_iter()
            .zip(point_properties)
            .map(|(point, properties)| {
                let coordinates = [self.map_x_coord(point.x()), self.map_y_coord(point.y())];

                let levels = (0..MAX_LEVEL + 1)
                    .map(|level| {
                        let point_x = self.map_x_coord(point.x());
                        let point_y = self.map_y_coord(point.y());

                        let tiles_across_width = 2u32.pow(1 + level as u32);
                        let tiles_across_height = 2u32.pow(level as u32);

                        let tile_x = (tiles_across_width as f32 * point_x).floor();
                        let tile_min_x = tile_x / tiles_across_width as f32;

                        let tile_y = (tiles_across_height as f32 * point_y).floor();
                        let tile_min_y = tile_y / tiles_across_height as f32;

                        let tile = format!("{}_{}_{}.pgm", level, tile_x, tile_y);
                        let tile_width = 1.0 / tiles_across_width as f32;
                        let tile_height = 1.0 / tiles_across_height as f32;

                        let x_in_tile = (point_x - tile_min_x) / tile_width;
                        let y_in_tile = (point_y - tile_min_y) / tile_height;

                        let x_in_tile = ELEVATION_TILE_SIZE as f32 * x_in_tile;
                        let y_in_tile = ELEVATION_TILE_SIZE as f32 * y_in_tile;

                        let x_in_tile = x_in_tile.floor() as u32;
                        let y_in_tile = ELEVATION_TILE_SIZE - y_in_tile.floor() as u32 - 1;

                        let elevation = Convert::new()
                            .input(&self.tiles_dir().join(tile))
                            .report_value_at_point((x_in_tile, y_in_tile))
                            .run_with_value()
                            .unwrap();

                        elevation.saturating_sub(ELEVATION_OFFSET) as f32
                    })
                    .collect();

                MultiLevelPoint {
                    coordinates,
                    properties,
                    levels,
                }
            })
            .collect();

        let features_data = FeaturesData { polygons, points };

        let features_file = File::create(self.output_dir.join("features.json"))
            .chain_err(|| "Error creating features file")?;

        serde_json::to_writer(features_file, &features_data)
            .chain_err(|| "Error writing out features file")?;

        Ok(())
    }

    fn map_x_coord(&self, x: f32) -> f32 {
        (x + 180.0) / 360.0
    }

    fn map_y_coord(&self, y: f32) -> f32 {
        (y + 90.0) / 180.0
    }

    fn tiles_dir(&self) -> PathBuf {
        self.output_dir.join("tiles")
    }
}
