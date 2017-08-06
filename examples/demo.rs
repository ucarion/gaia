extern crate cam;
extern crate fps_counter;
extern crate gaia;
extern crate gfx;
extern crate piston;
extern crate piston_window;
extern crate vecmath;

mod camera_controller;

use camera_controller::CameraController;

use cam::CameraPerspective;
use fps_counter::FPSCounter;
use gfx::Device;
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

fn get_mvp(window: &PistonWindow, camera_controller: &CameraController) -> [[f32; 4]; 4] {
    cam::model_view_projection(
        vecmath::mat4_id(),
        camera_controller.view_matrix(),
        get_projection(window),
    )
}

fn main() {
    if let Err(ref e) = run() {
        println!("error: {}", e);

        for e in e.iter().skip(1) {
            println!("caused by: {}", e);
        }

        if let Some(backtrace) = e.backtrace() {
            println!("{:?}", backtrace);
        }

        std::process::exit(1);
    }
}

fn run() -> gaia::Result<()> {
    use gaia::errors::ResultExt;

    let mut window: PistonWindow = WindowSettings::new("Gaia", [960, 520])
        .exit_on_esc(true)
        .opengl(OpenGL::V3_2)
        .build()
        .map_err(gaia::Error::from)
        .chain_err(|| "Could not create window")?;

    let mut camera_controller = CameraController::new();
    let mut gaia_renderer = gaia::Renderer::new(window.factory.clone())
        .chain_err(|| "Could not create renderer")?;

    let mut fps_counter = FPSCounter::new();
    let mut fps = 0;

    // TODO get the actual error
    let mut glyphs = Glyphs::new("assets/fonts/FiraSans-Regular.ttf", window.factory.clone())
        .map_err(|_err| gaia::Error::from("glyph error"))
        .chain_err(|| "Could not create glyphs")?;

    gaia_renderer.set_mvp(get_mvp(&window, &camera_controller));

    while let Some(e) = window.next() {
        camera_controller.event(&e);

        window.draw_3d(&e, |window| {
            window
                .encoder
                .clear(&window.output_color, [0.3, 0.3, 0.3, 1.0]);
            window.encoder.clear_depth(&window.output_stencil, 1.0);

            gaia_renderer.set_mvp(get_mvp(&window, &camera_controller));
            gaia_renderer.draw(
                &mut window.encoder,
                window.output_color.clone(),
                window.output_stencil.clone(),
            );

            window.device.cleanup();

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
            gaia_renderer.set_mvp(get_mvp(&window, &camera_controller));
        });
    }

    Ok(())
}