extern crate cam;
extern crate fps_counter;
extern crate gaia;
extern crate gfx;
extern crate piston;
extern crate piston_window;
extern crate vecmath;

// extern crate byteorder;
// extern crate cam;
// extern crate cgmath;
// extern crate collision;
// extern crate fps_counter;
// extern crate image;
// extern crate piston;
// extern crate piston_window;
// extern crate time;

mod camera_controller;
// mod constants;
// mod index_getter;
// mod texture_getter;
// mod tile;
// mod vertex;
// mod vertex_getter;

use camera_controller::CameraController;
// use tile::TileKind;

use cam::CameraPerspective;
use fps_counter::FPSCounter;
use piston::window::WindowSettings;
use piston_window::*;

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

    let mut gaia_renderer = gaia::Renderer::new(factory.clone());

    let mut camera_controller = CameraController::new();

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

            let mvp = cam::model_view_projection(
                vecmath::mat4_id(),
                camera_controller.view_matrix(),
                get_projection(window),
            );

            gaia_renderer.set_mvp(mvp);
            gaia_renderer.draw(
                &mut window.encoder,
                window.output_color.clone(),
                window.output_stencil.clone(),
            );

            fps = fps_counter.tick();
        });

        window.draw_2d(&e, |context, graphics| {
            // let camera_height = camera_controller.camera_position()[2];
            let camera_height = 1337.0;
            text::Text::new_color([0.0, 0.0, 0.0, 1.0], 10).draw(
                &format!("FPS: {} - Camera height: {}", fps, camera_height),
                &mut glyphs,
                &context.draw_state,
                context.transform.trans(10.0, 10.0),
                graphics,
            );
        });

        e.resize(|_, _| {
            // data.out_color = window.output_color.clone();
            // data.out_depth = window.output_stencil.clone();
        });
    }
}
