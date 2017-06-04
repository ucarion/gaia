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
mod index_getter;

use std::io::BufReader;
use std::fs::File;

use camera_controller::CameraController;

use byteorder::{LittleEndian, ReadBytesExt};
use cam::CameraPerspective;
use fps_counter::FPSCounter;
use gfx::traits::*;
use image::GenericImage;
use piston::window::WindowSettings;
use piston_window::*;

fn get_elevation_data() -> Vec<Vec<i16>> {
    println!("Getting elevation data...");
    let elev_data_width = 4097;
    let elev_data_height = 4097;

    let compression_factor = 1;

    let result_width = elev_data_width / compression_factor;
    let result_height = elev_data_height / compression_factor;

    let file = File::open("assets/east_hemisphere_elevation.bin").unwrap();
    let mut file = BufReader::new(file);

    let mut result = Vec::new();
    for _ in 0..result_height {
        result.push(vec![0; result_width]);
    }

    let mut count = 0;

    while let Ok(elevation) = file.read_i16::<LittleEndian>() {
        let (x, y) = (count % elev_data_width, count / elev_data_width);

        if x % compression_factor == 0 && y % compression_factor == 0 {
            result[y / compression_factor][x / compression_factor] = elevation;
        }

        count += 1;
    }

    println!("Done getting elevation data.");

    result
}


gfx_vertex_struct!( Vertex {
    a_pos: [f32; 4] = "a_pos",
    a_tex_coord: [f32; 2] = "a_tex_coord",
});

impl Vertex {
    fn new(pos: [f32; 3], tex_coord: [f32; 2]) -> Vertex {
        Vertex {
            a_pos: [pos[0], pos[1], pos[2], 1.0],
            a_tex_coord: tex_coord,
        }
    }
}

gfx_pipeline!( pipe {
    vbuf: gfx::VertexBuffer<Vertex> = (),
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

    let pso = factory.create_pipeline_simple(
        include_bytes!("shaders/foobar.glslv"),
        include_bytes!("shaders/foobar.glslf"),
        pipe::new(),
    ).unwrap();

    let model = vecmath::mat4_id();
    let mut camera_controller = CameraController::new();

    let elevation_data = get_elevation_data();

    println!("Generating vertices...");
    let vertex_data = get_vertex_data(elevation_data);
    let vbuf = factory.create_vertex_buffer(&vertex_data);
    println!("Done generating vertices.");

    let sampler_info = gfx::texture::SamplerInfo::new(
        gfx::texture::FilterMethod::Mipmap,
        gfx::texture::WrapMode::Tile,
    );

    let image_data1 = include_bytes!("../assets/east_hemisphere_small.jpg");
    let world_image1 = image::load_from_memory(image_data1).unwrap();
    let image_buffer1 = world_image1.to_rgba().into_raw();

    let image_data2 = include_bytes!("../assets/east_hemisphere_medium.jpg");
    let world_image2 = image::load_from_memory(image_data2).unwrap();
    let (width2, height2) = world_image2.dimensions();
    let image_buffer2 = world_image2.to_rgba().into_raw();

    // let image_data3 = include_bytes!("../assets/east_hemisphere_big.jpg");
    // let world_image3 = image::load_from_memory(image_data3).unwrap();
    // let (width3, height3) = world_image3.dimensions();
    // let image_buffer3 = world_image3.to_rgba().into_raw();

    let texture_kind = gfx::texture::Kind::D2(width2 as u16, height2 as u16, gfx::texture::AaMode::Single);
    let texture_data = [/*image_buffer3.as_slice(), */image_buffer2.as_slice(), image_buffer1.as_slice()];
    let (_texture, texture_view) = factory.create_texture_immutable_u8::<gfx::format::Rgba8>(
        texture_kind,
        &texture_data
    ).unwrap();

    let mut data = pipe::Data {
        vbuf: vbuf,
        u_model_view_proj: [[0.0; 4]; 4],
        u_offset_x: 0.0,
        t_color: (texture_view, factory.create_sampler(sampler_info)),
        out_color: window.output_color.clone(),
        out_depth: window.output_stencil.clone(),
    };

    let mut fps_counter = FPSCounter::new();

    while let Some(e) = window.next() {
        camera_controller.event(&e);

        window.draw_3d(&e, |window| {
            window.encoder.clear(&window.output_color, [0.3, 0.3, 0.3, 1.0]);
            window.encoder.clear_depth(&window.output_stencil, 1.0);

            let model_view_projection = cam::model_view_projection(
                model,
                camera_controller.view_matrix(),
                get_projection(&window),
            );

            let indices = index_getter::get_indices(
                model_view_projection,
                camera_controller.camera_position(),
                [0.0, 0.0],
            );
            let index_buffer = factory.create_index_buffer(indices.as_slice());
            let slice = gfx::Slice {
                start: 0,
                end: indices.len() as u32,
                base_vertex: 0,
                instances: None,
                buffer: index_buffer,
            };

            data.u_model_view_proj = model_view_projection;
            data.u_offset_x = 0.0;
            window.encoder.draw(&slice, &pso, &data);

            println!("{} - {}", fps_counter.tick(), camera_controller.camera_position()[2]);
        });

        e.resize(|_, _| {
            data.out_color = window.output_color.clone();
            data.out_depth = window.output_stencil.clone();
        });
    }
}

fn get_vertex(x: usize, y: usize, elevation: i16) -> Vertex {
    let tex_coord = [x as f32 / 4097.0, y as f32 / 4097.0];
    let z = get_z(elevation as f32 - 500.0);

    Vertex::new([x as f32, -(y as f32), z], tex_coord)
}

fn get_z(elevation: f32) -> f32 {
    if elevation <= 0.0 {
        0.0
    } else {
        elevation / 50.0
    }
}

fn get_vertex_data(elevation_data: Vec<Vec<i16>>) -> Vec<Vertex> {
    let height = elevation_data.len();
    let width = elevation_data[0].len();

    let mut vertex_data = Vec::new();

    for y in 0..height {
        for x in 0..width {
            vertex_data.push(get_vertex(x, y, elevation_data[y][x]));
        }
    }

    vertex_data
}
