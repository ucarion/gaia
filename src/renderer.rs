use constants::ELEVATION_TILE_WIDTH;
use errors::*;
use texture_getter;
use tile::{Tile, TileTextures};
use tile_getter;

use gfx;
use gfx::traits::FactoryExt;
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
    mvp: Option<[[f32; 4]; 4]>,
    pso: gfx::PipelineState<R, pipe::Meta>,
    sampler: gfx::handle::Sampler<R>,
    vertex_buffer: gfx::handle::Buffer<R, Vertex>,
    texture_cache: LruCache<Tile, TileTextures<R>>,
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

        let mut texture_cache = LruCache::new(100);
        let tile = Tile {
            level: 6,
            x: 0,
            y: 0,
        };
        let textures = texture_getter::get_textures(&mut factory, &tile)?;
        texture_cache.insert(tile, textures);
        let tile = Tile {
            level: 6,
            x: 1,
            y: 0,
        };
        let textures = texture_getter::get_textures(&mut factory, &tile)?;
        texture_cache.insert(tile, textures);
        let tile = Tile {
            level: 5,
            x: 1,
            y: 1,
        };
        let textures = texture_getter::get_textures(&mut factory, &tile)?;
        texture_cache.insert(tile, textures);

        Ok(Renderer {
            camera_position: None,
            factory: factory,
            mvp: None,
            pso: pso,
            sampler: sampler,
            vertex_buffer: vertex_buffer,
            texture_cache: texture_cache,
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
        let (tiles_to_render, tiles_to_fetch) =
            tile_getter::get_tiles(self.camera_position.unwrap(), &mut self.texture_cache);

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
                u_offset: positioned_tile.offset(),
                u_width: positioned_tile.tile.width(),
                vertex_buffer: self.vertex_buffer.clone(),
            };

            encoder.draw(&slice, &self.pso, &data);
        }

        // for tile in tiles_to_fetch {
        //     let textures = texture_getter::get_textures(&mut self.factory, &tile)?;
        //     self.texture_cache.insert(tile, textures);
        // }

        Ok(())
    }
}
