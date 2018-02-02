use std::io::BufReader;
use std::fs::File;

use byteorder::{LittleEndian, ReadBytesExt};
use gaia_assetgen::{TileMetadata, ELEVATION_OFFSET, IMAGERY_TILE_SIZE, ELEVATION_TILE_SIZE};
use gaia_quadtree::Tile;
use gfx;
use image;
use serde_json;

use errors::*;

pub struct TileAssets<R: gfx::Resources> {
    pub color: gfx::handle::ShaderResourceView<R, [f32; 4]>,
    pub elevation: gfx::handle::ShaderResourceView<R, u32>,
    pub metadata: TileMetadata,
}

pub struct TileAssetData {
    pub color: Vec<u8>,
    pub elevation: Vec<u16>,
    pub metadata: TileMetadata,
}

impl TileAssetData {
    pub fn new(tile: &Tile) -> Result<TileAssetData> {
        Ok(TileAssetData {
            color: get_color_data(tile)?,
            elevation: get_elevation_data(tile)?,
            metadata: get_metadata(tile)?,
        })
    }

    pub fn create_assets<R: gfx::Resources, F: gfx::Factory<R>>(
        self,
        factory: &mut F,
    ) -> Result<TileAssets<R>> {
        let color_texture_kind = gfx::texture::Kind::D2(
            IMAGERY_TILE_SIZE as u16,
            IMAGERY_TILE_SIZE as u16,
            gfx::texture::AaMode::Single,
        );
        let (_, color_texture_view) = factory
            .create_texture_immutable_u8::<gfx::format::Srgba8>(
                color_texture_kind,
                &[self.color.as_slice()],
            )
            .chain_err(|| "Could not create color texture")?;

        let elevation_texture_kind = gfx::texture::Kind::D2(
            ELEVATION_TILE_SIZE as u16,
            ELEVATION_TILE_SIZE as u16,
            gfx::texture::AaMode::Single,
        );
        let (_, elevation_texture_view) = factory
            .create_texture_immutable::<(gfx::format::R16, gfx::format::Uint)>(
                elevation_texture_kind,
                &[self.elevation.as_slice()],
            )
            .chain_err(|| "Could not create color texture")?;

        Ok(TileAssets {
            color: color_texture_view,
            elevation: elevation_texture_view,
            metadata: self.metadata,
        })
    }
}

fn get_color_data(tile: &Tile) -> Result<Vec<u8>> {
    let path = format!(
        "assets/generated/tiles/{}_{}_{}.jpg",
        tile.level,
        tile.x,
        tile.y
    );

    let img = image::open(path).chain_err(
        || "Error reading tile image data",
    )?;
    Ok(img.to_rgba().into_raw())
}

fn get_elevation_data(tile: &Tile) -> Result<Vec<u16>> {
    let path = format!(
        "assets/generated/tiles/{}_{}_{}.gray",
        tile.level,
        tile.x,
        tile.y
    );

    let mut file = BufReader::new(File::open(&path).chain_err(
        || "Error opening tile elevation data",
    )?);

    let mut buf = Vec::new();
    while let Ok(elevation) = file.read_u16::<LittleEndian>() {
        buf.push(elevation.saturating_sub(ELEVATION_OFFSET));
    }

    Ok(buf)
}

fn get_metadata(tile: &Tile) -> Result<TileMetadata> {
    let path = format!(
        "assets/generated/tiles/{}_{}_{}.json",
        tile.level,
        tile.x,
        tile.y
    );

    let file = BufReader::new(
        File::open(path).chain_err(|| "Error reading tile metadata")?,
    );

    Ok(serde_json::from_reader(file).chain_err(
        || "Error parsing tile metadata",
    )?)
}
