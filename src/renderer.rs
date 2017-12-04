use std::collections::BTreeMap;
use std::f32;
use std::fs::File;
use std::io::BufReader;
use std::sync::mpsc;
use std::thread;

use cgmath::Matrix4;
use gaia_assetgen::PolygonPointData;
use gfx::traits::FactoryExt;
use gfx;
use gfx_draping;
use hsl::HSL;
use lru_cache::LruCache;
use serde_json;

use asset_getter::{TileAssets, TileAssetData};
use constants::{ELEVATION_TILE_WIDTH, MAX_TILE_LEVEL};
use errors::*;
use tile::Tile;
use tile_chooser;
use tile_fetcher;

#[cfg_attr(rustfmt, rustfmt_skip)]
gfx_vertex_struct!(Vertex {
    coord: [f32; 2] = "a_coord",
});

gfx_pipeline!(pipe {
    o_color: gfx::RenderTarget<gfx::format::Srgba8> = "o_color",
    o_depth: gfx::DepthTarget<gfx::format::DepthStencil> = gfx::preset::depth::LESS_EQUAL_WRITE,
    t_color: gfx::TextureSampler<[f32; 4]> = "t_color",
    t_elevation: gfx::TextureSampler<u32> = "t_elevation",
    u_mvp: gfx::Global<[[f32; 4]; 4]> = "u_mvp",
    vertex_buffer: gfx::VertexBuffer<Vertex> = (),
});

pub struct Renderer<R: gfx::Resources, F: gfx::Factory<R>> {
    asset_cache: LruCache<Tile, TileAssets<R>>,
    camera_position: Option<[f32; 3]>,
    draping_renderer: gfx_draping::DrapingRenderer<R>,
    factory: F,
    mvp: Option<Matrix4<f32>>,
    polygon_buffers: Vec<gfx_draping::RenderablePolygonBuffer<R>>,
    polygon_indices: BTreeMap<(u8, u64), (gfx_draping::PolygonBufferIndices, serde_json::Map<String, serde_json::Value>)>,
    polygon_indices_cache: LruCache<(u8, Vec<u64>), gfx_draping::RenderablePolygonIndices<R>>,
    pso: gfx::PipelineState<R, pipe::Meta>,
    receive_textures: mpsc::Receiver<(Tile, Result<TileAssetData>)>,
    sampler: gfx::handle::Sampler<R>,
    send_tiles: mpsc::Sender<Tile>,
    vertex_buffer: gfx::handle::Buffer<R, Vertex>,
}

impl<R: gfx::Resources, F: gfx::Factory<R>> Renderer<R, F> {
    pub fn new(mut factory: F) -> Result<Renderer<R, F>> {
        let sampler = factory.create_sampler(gfx::texture::SamplerInfo::new(
            gfx::texture::FilterMethod::Bilinear,
            gfx::texture::WrapMode::Clamp,
        ));

        let pso = factory
            .create_pipeline_simple(
                include_bytes!("shaders/terrain.glslv"),
                include_bytes!("shaders/terrain.glslf"),
                pipe::new(),
            )
            .chain_err(|| "Could not create pipeline")?;

        let mut vertex_data = vec![];
        for y in 0..ELEVATION_TILE_WIDTH {
            for x in 0..ELEVATION_TILE_WIDTH {
                let coord_scale = (ELEVATION_TILE_WIDTH - 1) as f32;
                vertex_data.push(Vertex {
                    coord: [x as f32 / coord_scale, y as f32 / coord_scale],
                });
            }
        }

        let vertex_buffer = factory.create_vertex_buffer(&vertex_data);
        let asset_cache = LruCache::new(2048);

        let polygon_point_data: PolygonPointData = serde_json::from_reader(BufReader::new(
            File::open("assets/generated/polygons.json").chain_err(
                || "Error opening polygons.json",
            )?,
        )).chain_err(|| "Error parsing polygons.json")?;

        let mut polygon_buffers = vec![gfx_draping::PolygonBuffer::new(); MAX_TILE_LEVEL as usize + 1];
        let mut polygon_indices = BTreeMap::new();

        for (polygon_id, polygon) in polygon_point_data.polygons.iter().enumerate() {
            for (level, points) in polygon.levels.iter().enumerate() {
                let indices = polygon_buffers[level].add(&gfx_draping::Polygon::new(polygon.bounding_box, points.clone()));
                polygon_indices.insert((level as u8, polygon_id as u64), (indices, polygon.properties.clone()));
            }
        }

        let (send_tiles, receive_tiles) = mpsc::channel::<Tile>();
        let (send_textures, receive_textures) = mpsc::channel();

        thread::Builder::new()
            .name("tile_fetcher".to_string())
            .spawn(move || {
                tile_fetcher::fetch_tiles(receive_tiles, send_textures)
            })
            .chain_err(|| "Error creating texture loader thread")?;

        Ok(Renderer {
            polygon_buffers: polygon_buffers.into_iter().map(|buf| buf.as_renderable(&mut factory)).collect(),
            polygon_indices: polygon_indices,
            polygon_indices_cache: LruCache::new(1024),
            asset_cache: asset_cache,
            camera_position: None,
            draping_renderer: gfx_draping::DrapingRenderer::new(&mut factory),
            factory: factory,
            mvp: None,
            pso: pso,
            receive_textures: receive_textures,
            sampler: sampler,
            send_tiles: send_tiles,
            vertex_buffer: vertex_buffer,
        })
    }

    pub fn set_view_info(&mut self, camera_position: [f32; 3], mvp: Matrix4<f32>) {
        self.camera_position = Some(camera_position);
        self.mvp = Some(mvp);
    }

    pub fn draw<C: gfx::CommandBuffer<R>>(
        &mut self,
        encoder: &mut gfx::Encoder<R, C>,
        target: gfx::handle::RenderTargetView<R, gfx::format::Srgba8>,
        stencil: gfx::handle::DepthStencilView<R, gfx::format::DepthStencil>,
    ) -> Result<()> {
        // Get tiles loaded in background thread, and put them in the cache
        for (tile, tile_texture_data) in self.receive_textures.try_iter() {
            let assets = tile_texture_data?.create_assets(&mut self.factory)?;
            self.asset_cache.insert(tile, assets);
        }

        let camera_position = self.camera_position.ok_or(Error::from(
            "camera_position missing; maybe a missing call to set_view_info?",
        ))?;
        let mvp = self.mvp.ok_or(Error::from(
            "mvp missing; maybe a missing call to set_view_info?",
        ))?;

        // Using the updated cache, get tiles to render and those that should be added to cache
        let (level_of_detail, tiles_to_render, tiles_to_fetch) =
            tile_chooser::choose_tiles(camera_position, mvp, &mut self.asset_cache);

        // Queue tiles to fetch for background thread
        for tile_to_fetch in tiles_to_fetch {
            self.send_tiles.send(tile_to_fetch).chain_err(
                || "Error sending tile to background thread",
            )?;
        }

        // let mut polygons_to_render = BTreeMap::new();
        // let mut polygon_indices = gfx_draping::PolygonBufferIndices::new();
        // let mut min_z = f32::INFINITY;
        // let mut max_z = f32::NEG_INFINITY;

        let mut polygon_color_groups = BTreeMap::new();

        for (positioned_tile, index_data) in tiles_to_render {
            let slice = gfx::Slice {
                start: 0,
                end: index_data.len() as u32,
                base_vertex: 0,
                instances: None,
                buffer: self.factory.create_index_buffer(index_data.as_slice()),
            };

            let tile_assets = self.asset_cache.get_mut(&positioned_tile.tile).unwrap();

            let offset = Matrix4::from_translation(positioned_tile.offset().into());
            let scale = Matrix4::from_nonuniform_scale(
                positioned_tile.tile.width(),
                -positioned_tile.tile.width(),
                1.0,
            );
            let mvp = mvp * offset * scale;

            let data = pipe::Data {
                o_color: target.clone(),
                o_depth: stencil.clone(),
                t_color: (tile_assets.color.clone(), self.sampler.clone()),
                t_elevation: (tile_assets.elevation.clone(), self.sampler.clone()),
                u_mvp: mvp.into(),
                vertex_buffer: self.vertex_buffer.clone(),
            };

            encoder.draw(&slice, &self.pso, &data);

            for polygon_id in &tile_assets.metadata.polygons {
                let (ref _indices, ref properties) = self.polygon_indices[&(level_of_detail, *polygon_id)];
                // println!("About to get color");
                let color = properties["MAPCOLOR13"].as_f64().unwrap() as u8;
                // println!("Just got color");
                let &mut (ref mut polygon_ids, ref mut min_z, ref mut max_z) = polygon_color_groups.entry(color).or_insert((Vec::new(), f32::INFINITY, f32::NEG_INFINITY));
                polygon_ids.push(*polygon_id);
                *min_z = min_z.min(tile_assets.metadata.min_elevation as f32);
                *max_z = max_z.max(tile_assets.metadata.max_elevation as f32);


                // let (ref indices, ref _properties) = self.polygon_indices[&(level_of_detail, *polygon_id)];
                // polygon_indices.extend(indices);
            }
        }

        for (color, (polygon_ids, min_z, max_z)) in polygon_color_groups {
            let cache_key = (level_of_detail, polygon_ids);
            if !self.polygon_indices_cache.contains_key(&cache_key) {
                let mut polygon_indices = gfx_draping::PolygonBufferIndices::new();
                for polygon_id in &cache_key.1 {
                    let (ref indices, ref _properties) = self.polygon_indices[&(level_of_detail, *polygon_id)];
                    polygon_indices.extend(indices);
                }

                let renderable_indices = polygon_indices.as_renderable(&mut self.factory);
                self.polygon_indices_cache.insert(cache_key.clone(), renderable_indices);
            }

            let renderable_indices = self.polygon_indices_cache.get_mut(&cache_key).unwrap();
            let (min_z, max_z) = (elevation_to_z(min_z) - 0.01, elevation_to_z(max_z) + 0.01);
            let transform_polygon = Matrix4::from_nonuniform_scale(1.0, 1.0, (max_z - min_z)) *
                Matrix4::from_translation([0.0, 0.0, min_z].into());

            let (r, g, b) = HSL { h: 360.0 * (color as f64 / 13.0), s: 1.0, l: 0.5 }.to_rgb();

            self.draping_renderer.render(
                encoder,
                target.clone(),
                stencil.clone(),
                (mvp * transform_polygon).into(),
                [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 0.1],
                &self.polygon_buffers[level_of_detail as usize],
                &renderable_indices,
            );
        }

        Ok(())
    }
}

fn elevation_to_z(elevation: f32) -> f32 {
    let t = 1.0 - 1.0 / (1.0 + 0.0001 * elevation);
    return t * 0.03;
}
