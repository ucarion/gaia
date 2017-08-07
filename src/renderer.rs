use errors::*;
use texture_getter;
use tile::{Tile, TileTextures};

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
        for x in 0..128u16 {
            for y in 0..128u16 {
                vertex_data.push(Vertex { coord: [x as f32 / 127.0, y as f32 / 127.0] });
            }
        }
        let vertex_buffer = factory.create_vertex_buffer(&vertex_data);

        Ok(Renderer {
            camera_position: None,
            factory: factory,
            mvp: None,
            pso: pso,
            sampler: sampler,
            vertex_buffer: vertex_buffer,
            texture_cache: LruCache::new(1),
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
        let level0_width = 10.0;
        let level6_width = level0_width * 2u8.pow(6) as f32;

        let camera_position = self.camera_position.unwrap();
        let desired_level: u8 = match camera_position[2] {
            // 0.0...100.0 => 0,
            // 100.0...200.0 => 1,
            // 200.0...300.0 => 2,
            000.0...400.0 => 3,
            400.0...500.0 => 4,
            500.0...600.0 => 5,
            _ => 6,
        };

        let desired_level_width = level0_width * 2u8.pow(desired_level as u32) as f32;
        let tile_x = (camera_position[0] / desired_level_width as f32).floor() as i64;
        let tile_y = (-camera_position[1] / desired_level_width as f32).floor() as i64;

        let num_tiles_across = 128 / 2u8.pow(desired_level as u32);

        let tile_id_x = modulo(tile_x, num_tiles_across as i64);
        use std::cmp;
        let tile_id_y = cmp::max(tile_y, 0);

        let tile = Tile {
            level: desired_level,
            x: tile_id_x as u8,
            y: tile_id_y as u8,
        };

        if !self.texture_cache.contains_key(&tile) {
            let color_texture = texture_getter::get_color_texture(
                &mut self.factory,
                desired_level,
                tile_id_x as u8,
                tile_id_y as u8,
            )?;
            let elevation_texture = texture_getter::get_elevation_texture(
                &mut self.factory,
                desired_level,
                tile_id_x as u8,
                tile_id_y as u8,
            )?;

            self.texture_cache.insert(
                tile.clone(),
                TileTextures {
                    color: color_texture,
                    elevation: elevation_texture,
                },
            );
        }

        let textures = self.texture_cache.get_mut(&tile).unwrap();

        let data = pipe::Data {
            o_color: target.clone(),
            o_depth: stencil.clone(),
            t_color: (textures.color.clone(), self.sampler.clone()),
            t_elevation: (textures.elevation.clone(), self.sampler.clone()),
            u_mvp: self.mvp.ok_or(Error::from("no mvp before call to draw"))?,
            u_offset: [tile_x as f32 * desired_level_width, tile_y as f32 * desired_level_width],
            u_width: desired_level_width,
            vertex_buffer: self.vertex_buffer.clone(),
        };

        let mut index_data = vec![];
        for x in 0..128u16 {
            for y in 0..128u16 {
                if x != 127 && y != 127 {
                    let index = (x + 0) + (y + 0) * 128;
                    let right = (x + 1) + (y + 0) * 128;
                    let below = (x + 0) + (y + 1) * 128;
                    let below_right = (x + 1) + (y + 1) * 128;
                    index_data.extend_from_slice(&[index, below, right, right, below, below_right]);
                }
            }
        }

        let slice = gfx::Slice {
            start: 0,
            end: index_data.len() as u32,
            base_vertex: 0,
            instances: None,
            buffer: self.factory.create_index_buffer(index_data.as_slice()),
        };

        encoder.draw(&slice, &self.pso, &data);
        Ok(())
    }
}

fn modulo(a: i64, b: i64) -> i64 {
    let rem = a % b;
    if rem < 0 {
        rem + b
    } else {
        rem
    }
}
