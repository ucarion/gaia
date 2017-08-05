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
        let texture_data = [
            0xff,
            0x00,
            0x00,
            0xff,
            0x00,
            0xff,
            0x00,
            0xff,
            0x00,
            0x00,
            0xff,
            0xff,
            0xff,
            0xff,
            0x00,
            0xff,
        ];

        let texture_kind = gfx::texture::Kind::D2(2, 2, gfx::texture::AaMode::Single);
        let (_, texture_view) = factory
            .create_texture_immutable_u8::<gfx::format::Srgba8>(texture_kind, &[&texture_data])
            .unwrap();

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

// let pso = factory
//     .create_pipeline_simple(
//         include_bytes!("shaders/terrain.glslv"),
//         include_bytes!("shaders/terrain.glslf"),
//         pipe::new(),
//     )
// .unwrap();

// let mut camera_controller = CameraController::new();

// println!("Generating vertices...");
// let begin = time::now();

// let vertices_by_kind = vertex_getter::get_vertices();
// let vertex_buffers_by_kind: HashMap<_, _> = vertices_by_kind
//     .iter()
//     .map(|(kind, vertices)| {
//         (kind, factory.create_vertex_buffer(&vertices))
//     })
//     .collect();

// let end = time::now();
// println!("Done. Took: {}ms", (end - begin).num_milliseconds());

// println!("Generating textures...");
// let begin = time::now();

// let (textures_by_kind, sampler) =
//     texture_getter::create_world_textures_and_sampler(&mut factory);

// let end = time::now();
// println!("Done. Took: {}ms", (end - begin).num_milliseconds());

// let mut data = pipe::Data {
//     vbuf: vertex_buffers_by_kind[&TileKind::A1].clone(),
//     u_model_view_proj: [[0.0; 4]; 4],
//     u_offset: [0.0, 0.0],
//     t_color: (textures_by_kind[&TileKind::A1].clone(), sampler),
//     out_color: window.output_color.clone(),
//     out_depth: window.output_stencil.clone(),
// };

// let model_view_projection = cam::model_view_projection(
//     vecmath::mat4_id(),
//     camera_controller.view_matrix(),
//     get_projection(&window),
// );

// data.u_model_view_proj = model_view_projection;

// let tiles_to_render = index_getter::get_indices_and_offsets(
//     model_view_projection,
//     camera_controller.camera_position(),
// );

// for tile_info in tiles_to_render {
//     let index_buffer = factory.create_index_buffer(tile_info.indices.as_slice());
//     let slice = gfx::Slice {
//         start: 0,
//         end: tile_info.indices.len() as u32,
//         base_vertex: 0,
//         instances: None,
//         buffer: index_buffer,
//     };

//     data.vbuf = vertex_buffers_by_kind[&tile_info.kind].clone();
//     data.u_offset = tile_info.offset;
//     data.t_color.0 = textures_by_kind[&tile_info.kind].clone();
//     window.encoder.draw(&slice, &pso, &data);
// }

// gfx_pipeline!( pipe {
//     vbuf: gfx::VertexBuffer<vertex::Vertex> = (),
//     u_model_view_proj: gfx::Global<[[f32; 4]; 4]> = "u_model_view_proj",
//     u_offset: gfx::Global<[f32; 2]> = "u_offset",
//     t_color: gfx::TextureSampler<[f32; 4]> = "t_color",
//     out_color: gfx::RenderTarget<::gfx::format::Srgba8> = "o_Color",
//     out_depth: gfx::DepthTarget<::gfx::format::DepthStencil> =
//         gfx::preset::depth::LESS_EQUAL_WRITE,
// });
