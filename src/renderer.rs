use std::fs::File;
use std::io::BufReader;
use std::sync::mpsc;
use std::thread;

use constants::ELEVATION_TILE_WIDTH;
use errors::*;
use asset_getter::{TileAssets, TileAssetData};
use tile::Tile;
use tile_chooser;
use tile_fetcher;

use cgmath::Matrix4;
use gaia_assetgen::PolygonPointData;
use gfx::traits::FactoryExt;
use gfx;
use gfx_draping::{DrapingRenderer, DrapeablePolygon};
use lru_cache::LruCache;
use serde_json;

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
    draping_renderer: DrapingRenderer<R>,
    factory: F,
    mvp: Option<Matrix4<f32>>,
    polygon_point_data: PolygonPointData,
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

        let polygon_point_data = serde_json::from_reader(BufReader::new(
            File::open("assets/generated/polygons.json").chain_err(
                || "Error opening polygons.json",
            )?,
        )).chain_err(|| "Error parsing polygons.json")?;

        let (send_tiles, receive_tiles) = mpsc::channel::<Tile>();
        let (send_textures, receive_textures) = mpsc::channel();

        thread::Builder::new()
            .name("tile_fetcher".to_string())
            .spawn(move || {
                tile_fetcher::fetch_tiles(receive_tiles, send_textures)
            })
            .chain_err(|| "Error creating texture loader thread")?;

        Ok(Renderer {
            asset_cache: asset_cache,
            camera_position: None,
            draping_renderer: DrapingRenderer::new(&mut factory),
            factory: factory,
            mvp: None,
            polygon_point_data: polygon_point_data,
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
        let (tiles_to_render, tiles_to_fetch) =
            tile_chooser::choose_tiles(camera_position, mvp, &mut self.asset_cache);

        // Queue tiles to fetch for background thread
        for tile_to_fetch in tiles_to_fetch {
            self.send_tiles.send(tile_to_fetch).chain_err(
                || "Error sending tile to background thread",
            )?;
        }

        for (positioned_tile, index_data) in tiles_to_render {
            let slice = gfx::Slice {
                start: 0,
                end: index_data.len() as u32,
                base_vertex: 0,
                instances: None,
                buffer: self.factory.create_index_buffer(index_data.as_slice()),
            };

            let textures = self.asset_cache.get_mut(&positioned_tile.tile).unwrap();

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
                t_color: (textures.color.clone(), self.sampler.clone()),
                t_elevation: (textures.elevation.clone(), self.sampler.clone()),
                u_mvp: mvp.into(),
                vertex_buffer: self.vertex_buffer.clone(),
            };

            encoder.draw(&slice, &self.pso, &data);
        }

        for polygon in &self.polygon_point_data.polygons {
            if polygon.properties["ADMIN"] == "France" {
                let polygon = DrapeablePolygon::new_from_points(
                    &mut self.factory,
                    &polygon.levels[0],
                    &[(0.0, 2.0), (0.0, 2.0), (-1.0, 1.0)],
                );

                self.draping_renderer.render(
                    encoder,
                    target.clone(),
                    stencil.clone(),
                    mvp.into(),
                    [0.0, 1.0, 1.0, 0.2],
                    &polygon,
                );
            }
        }


        Ok(())
    }
}
