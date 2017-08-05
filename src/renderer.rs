use texture_getter;

use gfx;
use gfx::traits::FactoryExt;

#[cfg_attr(rustfmt, rustfmt_skip)]
gfx_vertex_struct!(Vertex {
    coord: [f32; 2] = "a_coord",
});

gfx_pipeline!(pipe {
    o_color: gfx::RenderTarget<gfx::format::Srgba8> = "o_color",
    o_depth: gfx::DepthTarget<gfx::format::DepthStencil> = gfx::preset::depth::LESS_EQUAL_WRITE,
    t_color: gfx::TextureSampler<[f32; 4]> = "t_color",
    u_mvp: gfx::Global<[[f32; 4]; 4]> = "u_mvp",
    vertex_buffer: gfx::VertexBuffer<Vertex> = (),
});

pub struct Renderer<R: gfx::Resources, F: gfx::Factory<R>> {
    factory: F,
    mvp: Option<[[f32; 4]; 4]>,
    pso: gfx::PipelineState<R, pipe::Meta>,
    sampler: gfx::handle::Sampler<R>,
    texture: gfx::handle::ShaderResourceView<R, [f32; 4]>,
    vertex_buffer: gfx::handle::Buffer<R, Vertex>,
    vertex_slice: gfx::Slice<R>,
}

impl<R: gfx::Resources, F: gfx::Factory<R>> Renderer<R, F> {
    pub fn new(mut factory: F) -> Renderer<R, F> {
        let texture_view = texture_getter::get_texture(&mut factory, 6, 0, 0);

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
            .unwrap();

        let vertex_data = vec![
            Vertex { coord: [0.0, 0.0] },
            Vertex { coord: [1.0, 0.0] },
            Vertex { coord: [0.0, 1.0] },
            Vertex { coord: [1.0, 1.0] },
        ];

        let index_data: &[u16] = &[0, 1, 2, 1, 2, 3];
        let (vertex_buffer, vertex_slice) = factory
            .create_vertex_buffer_with_slice(&vertex_data, index_data);

        Renderer {
            factory: factory,
            mvp: None,
            pso: pso,
            sampler: sampler,
            texture: texture_view,
            vertex_buffer: vertex_buffer,
            vertex_slice: vertex_slice,
        }
    }

    pub fn set_mvp(&mut self, mvp: [[f32; 4]; 4]) {
        self.mvp = Some(mvp);
    }

    pub fn draw<C: gfx::CommandBuffer<R>>(
        &mut self,
        encoder: &mut gfx::Encoder<R, C>,
        target: gfx::handle::RenderTargetView<R, gfx::format::Srgba8>,
        stencil: gfx::handle::DepthStencilView<R, gfx::format::DepthStencil>,
    ) {
        let data = pipe::Data {
            o_color: target,
            o_depth: stencil,
            t_color: (self.texture.clone(), self.sampler.clone()),
            u_mvp: self.mvp.unwrap(),
            vertex_buffer: self.vertex_buffer.clone(),
        };

        encoder.draw(&self.vertex_slice, &self.pso, &data);
    }
}
