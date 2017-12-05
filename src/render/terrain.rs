use gfx;
use gfx::traits::FactoryExt;

use errors::*;

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
    pso: gfx::PipelineState<R, pipe::Meta>,
    factory: F,
}

impl<R: gfx::Resources, F: gfx::Factory<R>> TerrainRenderer<R, F> {
    pub fn new(mut factory: F) -> Result<TerrainRenderer<R, F>> {
        let pso = factory
            .create_pipeline_simple(
                include_bytes!("../shaders/terrain.glslv"),
                include_bytes!("../shaders/terrain.glslf"),
                pipe::new(),
            )
            .chain_err(|| "Could not create pipeline")?;

        Ok(TerrainRenderer { factory, pso })
    }
}
