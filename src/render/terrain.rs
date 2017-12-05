use cgmath::{Matrix4, Vector2};
use gfx;
use gfx::traits::FactoryExt;

use asset_getter::TileAssets;
use constants::ELEVATION_TILE_WIDTH;
use errors::*;
use tile::PositionedTile;

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

pub struct TerrainRenderer<R: gfx::Resources, F: gfx::Factory<R>> {
    factory: F,
    pso: gfx::PipelineState<R, pipe::Meta>,
    sampler: gfx::handle::Sampler<R>,
    vertex_buffer: gfx::handle::Buffer<R, Vertex>,
}

impl<R: gfx::Resources, F: gfx::Factory<R>> TerrainRenderer<R, F> {
    pub fn new(mut factory: F) -> Result<TerrainRenderer<R, F>> {
        let sampler = factory.create_sampler(gfx::texture::SamplerInfo::new(
            gfx::texture::FilterMethod::Bilinear,
            gfx::texture::WrapMode::Clamp,
        ));

        let vertex_buffer = Self::create_vertex_buffer(&mut factory);

        let pso = factory
            .create_pipeline_simple(
                include_bytes!("../shaders/terrain.glslv"),
                include_bytes!("../shaders/terrain.glslf"),
                pipe::new(),
            )
            .chain_err(|| "Could not create pipeline")?;

        Ok(TerrainRenderer {
            factory,
            pso,
            sampler,
            vertex_buffer,
        })
    }

    pub fn render<
        C: gfx::CommandBuffer<R>,
    >(
        &mut self,
        encoder: &mut gfx::Encoder<R, C>,
        target: gfx::handle::RenderTargetView<R, gfx::format::Srgba8>,
        stencil: gfx::handle::DepthStencilView<R, gfx::format::DepthStencil>,
        mvp: &Matrix4<f32>,
        positioned_tile: PositionedTile,
        indices: Vec<u16>,
        tile_assets: &TileAssets<R>,
    ) {
        let slice = gfx::Slice {
            start: 0,
            end: indices.len() as u32,
            base_vertex: 0,
            instances: None,
            buffer: self.factory.create_index_buffer(&indices[..]),
        };

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
    }

    fn create_vertex_buffer(factory: &mut F) -> gfx::handle::Buffer<R, Vertex> {
        let mut vertex_data = vec![];
        for y in 0..ELEVATION_TILE_WIDTH {
            for x in 0..ELEVATION_TILE_WIDTH {
                let coord_scale = (ELEVATION_TILE_WIDTH - 1) as f32;
                vertex_data.push(Vertex {
                    coord: [x as f32 / coord_scale, y as f32 / coord_scale],
                });
            }
        }

        factory.create_vertex_buffer(&vertex_data)
    }
}
