use std::sync::mpsc;
use std::thread;

use constants::ELEVATION_TILE_WIDTH;
use errors::*;
use texture_getter::{TileTextures, TileTextureData};
use tile::Tile;
use tile_chooser;
use tile_fetcher;

use gfx::traits::FactoryExt;
use gfx;
use lru_cache::LruCache;

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
    u_offset: gfx::Global<[f32; 2]> = "u_offset",
    u_width: gfx::Global<f32> = "u_width",
    vertex_buffer: gfx::VertexBuffer<Vertex> = (),
});

pub struct Renderer<R: gfx::Resources, F: gfx::Factory<R>> {
    camera_position: Option<[f32; 3]>,
    factory: F,
    level0_tile_width: f32,
    mvp: Option<[[f32; 4]; 4]>,
    pso: gfx::PipelineState<R, pipe::Meta>,
    receive_textures: mpsc::Receiver<(Tile, Result<TileTextureData>)>,
    sampler: gfx::handle::Sampler<R>,
    send_tiles: mpsc::Sender<Tile>,
    texture_cache: LruCache<Tile, TileTextures<R>>,
    vertex_buffer: gfx::handle::Buffer<R, Vertex>,
}

impl<R: gfx::Resources, F: gfx::Factory<R>> Renderer<R, F> {
    pub fn new(mut factory: F, world_height: f32) -> Result<Renderer<R, F>> {
        let level0_tile_width = world_height / Tile::num_tiles_across_level_height(0) as f32;

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
        let texture_cache = LruCache::new(2048);

        let (send_tiles, receive_tiles) = mpsc::channel::<Tile>();
        let (send_textures, receive_textures) = mpsc::channel();

        thread::Builder::new()
            .name("tile_fetcher".to_string())
            .spawn(move || {
                tile_fetcher::fetch_tiles(receive_tiles, send_textures)
            })
            .chain_err(|| "Error creating texture loader thread")?;

        Ok(Renderer {
            camera_position: None,
            factory: factory,
            level0_tile_width: level0_tile_width,
            mvp: None,
            pso: pso,
            receive_textures: receive_textures,
            sampler: sampler,
            send_tiles: send_tiles,
            texture_cache: texture_cache,
            vertex_buffer: vertex_buffer,
        })
    }

    pub fn set_view_info(&mut self, camera_position: [f32; 3], mvp: [[f32; 4]; 4]) {
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
            let tile_textures = tile_texture_data?.create_textures(&mut self.factory)?;
            self.texture_cache.insert(tile, tile_textures);
        }

        // Using the updated cache, get tiles to render and those that should be added to cache
        let (tiles_to_render, tiles_to_fetch) = tile_chooser::choose_tiles(
            self.level0_tile_width,
            self.camera_position.unwrap(),
            &mut self.texture_cache,
        );

        // Queue tiles to fetch for background thread
        for tile_to_fetch in tiles_to_fetch {
            self.send_tiles
                .send(tile_to_fetch)
                .chain_err(|| "Error sending tile to background thread")?;
        }

        for (positioned_tile, index_data) in tiles_to_render {
            let slice = gfx::Slice {
                start: 0,
                end: index_data.len() as u32,
                base_vertex: 0,
                instances: None,
                buffer: self.factory.create_index_buffer(index_data.as_slice()),
            };

            let textures = self.texture_cache.get_mut(&positioned_tile.tile).unwrap();

            let data = pipe::Data {
                o_color: target.clone(),
                o_depth: stencil.clone(),
                t_color: (textures.color.clone(), self.sampler.clone()),
                t_elevation: (textures.elevation.clone(), self.sampler.clone()),
                u_mvp: self.mvp.ok_or(Error::from("no mvp before call to draw"))?,
                u_offset: positioned_tile.offset(self.level0_tile_width),
                u_width: positioned_tile.tile.width(self.level0_tile_width),
                vertex_buffer: self.vertex_buffer.clone(),
            };

            encoder.draw(&slice, &self.pso, &data);
        }

        Ok(())
    }
}
