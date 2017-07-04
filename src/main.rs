extern crate byteorder;
extern crate cam;
extern crate cgmath;
extern crate collision;
extern crate fps_counter;
#[macro_use]
extern crate gfx;
extern crate image;
extern crate piston;
extern crate piston_window;
extern crate time;
extern crate vecmath;

mod camera_controller;
mod constants;
mod index_getter;
mod vertex;
mod vertex_getter;

use camera_controller::CameraController;

use cam::CameraPerspective;
use fps_counter::FPSCounter;
use gfx::traits::*;
use image::GenericImage;
use piston::window::WindowSettings;
use piston_window::*;

gfx_pipeline!( pipe {
    vbuf: gfx::VertexBuffer<vertex::Vertex> = (),
    u_model_view_proj: gfx::Global<[[f32; 4]; 4]> = "u_model_view_proj",
    u_offset_x: gfx::Global<f32> = "u_offset_x",
    t_color: gfx::TextureSampler<[f32; 4]> = "t_color",
    out_color: gfx::RenderTarget<::gfx::format::Srgba8> = "o_Color",
    out_depth: gfx::DepthTarget<::gfx::format::DepthStencil> =
        gfx::preset::depth::LESS_EQUAL_WRITE,
});

fn get_projection(window: &PistonWindow) -> [[f32; 4]; 4] {
    let draw_size = window.window.draw_size();

    CameraPerspective {
        fov: 45.0,
        near_clip: 0.1,
        far_clip: 10000.0,
        aspect_ratio: (draw_size.width as f32) / (draw_size.height as f32),
    }.projection()
}

fn main() {
    let mut window: PistonWindow = WindowSettings::new("Gaia", [960, 520])
        .exit_on_esc(true)
        .opengl(OpenGL::V3_2)
        .build()
        .unwrap();

    let ref mut factory = window.factory.clone();

    let pso = factory
        .create_pipeline_simple(
            include_bytes!("shaders/foobar.glslv"),
            include_bytes!("shaders/foobar.glslf"),
            pipe::new(),
        )
        .unwrap();

    let mut camera_controller = CameraController::new();

    println!("Generating vertices...");
    let vertex_data = vertex_getter::get_vertices();
    let vbuf = factory.create_vertex_buffer(&vertex_data);
    println!("Done generating vertices.");

    println!("Generating textures...");
    let begin = time::now();
    let (texture_views, sampler) = create_world_textures_and_sampler(factory);
    let end = time::now();
    println!("Done. Took: {}ms", (end - begin).num_milliseconds());

    let mut data = pipe::Data {
        vbuf: vbuf,
        u_model_view_proj: [[0.0; 4]; 4],
        u_offset_x: 0.0,
        t_color: (texture_views[1].clone(), sampler),
        out_color: window.output_color.clone(),
        out_depth: window.output_stencil.clone(),
    };

    let mut fps_counter = FPSCounter::new();
    let mut fps = 0;

    let mut glyphs = Glyphs::new("assets/fonts/FiraSans-Regular.ttf", factory.clone()).unwrap();

    while let Some(e) = window.next() {
        camera_controller.event(&e);

        window.draw_3d(&e, |window| {
            window
                .encoder
                .clear(&window.output_color, [0.3, 0.3, 0.3, 1.0]);
            window.encoder.clear_depth(&window.output_stencil, 1.0);

            let model_view_projection = cam::model_view_projection(
                vecmath::mat4_id(),
                camera_controller.view_matrix(),
                get_projection(&window),
            );

            data.u_model_view_proj = model_view_projection;

            let tiles_to_render = index_getter::get_indices_and_offsets(
                model_view_projection,
                camera_controller.camera_position(),
            );

            for (indices, x_offset, tile_is_west) in tiles_to_render {
                let index_buffer = factory.create_index_buffer(indices.as_slice());
                let slice = gfx::Slice {
                    start: 0,
                    end: indices.len() as u32,
                    base_vertex: 0,
                    instances: None,
                    buffer: index_buffer,
                };

                let texture_view = if tile_is_west {
                    texture_views[0].clone()
                } else {
                    texture_views[1].clone()
                };

                data.u_offset_x = x_offset;
                data.t_color.0 = texture_view;
                window.encoder.draw(&slice, &pso, &data);
            }

            fps = fps_counter.tick();
        });

        window.draw_2d(&e, |context, graphics| {
            let camera_height = camera_controller.camera_position()[2];
            text::Text::new_color([0.0, 0.0, 0.0, 1.0], 10).draw(
                &format!("FPS: {} - Camera height: {}", fps, camera_height),
                &mut glyphs,
                &context.draw_state,
                context.transform.trans(10.0, 10.0),
                graphics,
            );
        });

        e.resize(|_, _| {
            data.out_color = window.output_color.clone();
            data.out_depth = window.output_stencil.clone();
        });
    }
}

fn create_world_textures_and_sampler<F, R>(
    factory: &mut F,
) -> (
    [gfx::handle::ShaderResourceView<R, [f32; 4]>; 2],
    gfx::handle::Sampler<R>,
)
where
    R: gfx::Resources,
    F: gfx::Factory<R>,
{
    let texture_view_west = create_world_texture(
        factory,
        [
            include_bytes!("../assets/generated/west_hemisphere-0.bmp"),
            include_bytes!("../assets/generated/west_hemisphere-1.bmp"),
            include_bytes!("../assets/generated/west_hemisphere-2.bmp"),
            include_bytes!("../assets/generated/west_hemisphere-3.bmp"),
        ],
    );

    let texture_view_east = create_world_texture(
        factory,
        [
            include_bytes!("../assets/generated/east_hemisphere-0.bmp"),
            include_bytes!("../assets/generated/east_hemisphere-1.bmp"),
            include_bytes!("../assets/generated/east_hemisphere-2.bmp"),
            include_bytes!("../assets/generated/east_hemisphere-3.bmp"),
        ],
    );

    let sampler = factory.create_sampler(gfx::texture::SamplerInfo::new(
        gfx::texture::FilterMethod::Bilinear,
        gfx::texture::WrapMode::Tile,
    ));

    ([texture_view_west, texture_view_east], sampler)
}

fn create_world_texture<F, R>(
    factory: &mut F,
    image_data: [&[u8]; 4],
) -> gfx::handle::ShaderResourceView<R, [f32; 4]>
where
    R: gfx::Resources,
    F: gfx::Factory<R>,
{
    let image0 = image::load_from_memory(image_data[0]).unwrap();
    let buffer0 = image0.to_rgba().into_raw();

    let (width, height) = image0.dimensions();
    let texture_kind =
        gfx::texture::Kind::D2(width as u16, height as u16, gfx::texture::AaMode::Single);

    let image1 = image::load_from_memory(image_data[1]).unwrap();
    let buffer1 = image1.to_rgba().into_raw();

    let image2 = image::load_from_memory(image_data[2]).unwrap();
    let buffer2 = image2.to_rgba().into_raw();

    let image_data3 = include_bytes!("../assets/generated/east_hemisphere-3.bmp");
    let image3 = image::load_from_memory(image_data3).unwrap();
    let buffer3 = image3.to_rgba().into_raw();

    let texture_data = [
        buffer0.as_slice(),
        buffer1.as_slice(),
        buffer2.as_slice(),
        buffer3.as_slice(),
    ];

    let (_texture, texture_view) = factory
        .create_texture_immutable_u8::<gfx::format::Rgba8>(texture_kind, &texture_data)
        .unwrap();

    texture_view
}
