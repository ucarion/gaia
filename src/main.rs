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
mod texture_getter;
mod tile;
mod vertex;
mod vertex_getter;

use camera_controller::CameraController;
use tile::TileKind;

use cam::CameraPerspective;
use fps_counter::FPSCounter;
use gfx::traits::*;
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

    let mut factory = window.factory.clone();

    let pso = factory
        .create_pipeline_simple(
            include_bytes!("shaders/terrain.glslv"),
            include_bytes!("shaders/terrain.glslf"),
            pipe::new(),
        )
        .unwrap();

    let mut camera_controller = CameraController::new();

    println!("Generating vertices...");
    let begin = time::now();
    let vertex_data = vertex_getter::get_vertices();
    let vbuf = factory.create_vertex_buffer(&vertex_data);
    let end = time::now();
    println!("Done. Took: {}ms", (end - begin).num_milliseconds());

    println!("Generating textures...");
    let begin = time::now();
    let (texture_views, sampler) = texture_getter::create_world_textures_and_sampler(&mut factory);
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

            for tile_info in tiles_to_render {
                let index_buffer = factory.create_index_buffer(tile_info.indices.as_slice());
                let slice = gfx::Slice {
                    start: 0,
                    end: tile_info.indices.len() as u32,
                    base_vertex: 0,
                    instances: None,
                    buffer: index_buffer,
                };

                let texture_view = match tile_info.kind {
                    TileKind::WestHemisphere => texture_views[0].clone(),
                    TileKind::EastHemisphere => texture_views[1].clone(),
                };

                data.u_offset_x = tile_info.x_offset;
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
