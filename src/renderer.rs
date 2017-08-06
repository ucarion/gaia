use errors::*;
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
    t_elevation: gfx::TextureSampler<u32> = "t_elevation",
    u_mvp: gfx::Global<[[f32; 4]; 4]> = "u_mvp",
    u_offset: gfx::Global<[f32; 2]> = "u_offset",
    u_width: gfx::Global<f32> = "u_width",
    vertex_buffer: gfx::VertexBuffer<Vertex> = (),
});

pub struct Renderer<R: gfx::Resources, F: gfx::Factory<R>> {
    factory: F,
    mvp: Option<[[f32; 4]; 4]>,
    pso: gfx::PipelineState<R, pipe::Meta>,
    sampler: gfx::handle::Sampler<R>,
    color_texture: gfx::handle::ShaderResourceView<R, [f32; 4]>,
    elevation_texture: gfx::handle::ShaderResourceView<R, u32>,
    vertex_buffer: gfx::handle::Buffer<R, Vertex>,
    vertex_slice: gfx::Slice<R>,
}

impl<R: gfx::Resources, F: gfx::Factory<R>> Renderer<R, F> {
    pub fn new(mut factory: F) -> Result<Renderer<R, F>> {
        let color_texture_view = texture_getter::get_color_texture(&mut factory, 6, 0, 0)?;
        let elevation_texture_view = texture_getter::get_elevation_texture(&mut factory, 6, 0, 0)?;

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
        let mut index_data = vec![];
        for x in 0..128u16 {
            for y in 0..128u16 {
                vertex_data.push(Vertex { coord: [x as f32 / 127.0, y as f32 / 127.0] });

                if x != 127 && y != 127 {
                    let index = (x + 0) + (y + 0) * 128;
                    let right = (x + 1) + (y + 0) * 128;
                    let below = (x + 0) + (y + 1) * 128;
                    let below_right = (x + 1) + (y + 1) * 128;
                    index_data.extend_from_slice(&[index, below, right, right, below, below_right]);
                }
            }
        }

        let (vertex_buffer, vertex_slice) = factory
            .create_vertex_buffer_with_slice(&vertex_data, index_data.as_slice());

        Ok(Renderer {
            factory: factory,
            mvp: None,
            pso: pso,
            sampler: sampler,
            color_texture: color_texture_view,
            elevation_texture: elevation_texture_view,
            vertex_buffer: vertex_buffer,
            vertex_slice: vertex_slice,
        })
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
            t_color: (self.color_texture.clone(), self.sampler.clone()),
            t_elevation: (self.elevation_texture.clone(), self.sampler.clone()),
            u_offset: [0.0, 0.0],
            u_width: 1000.0,
            u_mvp: self.mvp.unwrap(),
            vertex_buffer: self.vertex_buffer.clone(),
        };

        encoder.draw(&self.vertex_slice, &self.pso, &data);
    }
}
