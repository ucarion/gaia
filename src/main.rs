extern crate byteorder;
extern crate camera_controllers;
#[macro_use]
extern crate gfx;
extern crate piston_window;
extern crate sdl2_window;
extern crate vecmath;

use std::io::BufReader;
use std::fs::File;

use byteorder::{LittleEndian, ReadBytesExt};
use camera_controllers::{FirstPerson, FirstPersonSettings, CameraPerspective};
use gfx::traits::*;
use piston_window::*;
use sdl2_window::Sdl2Window;

fn get_elevation_data() -> Vec<Vec<f32>> {
    println!("Getting elevation data...");
    let compression_factor = 8;

    let elev_data_width = 10800;
    let elev_data_height = 6000;

    let image_width = elev_data_width / compression_factor;
    let image_height = elev_data_height / compression_factor;

    let file = File::open("assets/noaa_globe/h10g").unwrap();
    let mut file = BufReader::new(file);

    let mut height_data = Vec::with_capacity(image_width);
    for _ in 0..image_height {
        height_data.push(vec![0.0; image_width]);
    }

    let mut count = 0;

    while let Ok(elevation) = file.read_i16::<LittleEndian>() {
        let (x, y) = (count % elev_data_width, count / elev_data_width);

        if x % compression_factor == 0 && y % compression_factor == 0 {
            height_data[y / compression_factor][x / compression_factor] = elevation as f32;
        }

        count += 1;
    }
    println!("Done getting elevation data.");

    height_data
}


gfx_vertex_struct!( Vertex {
    a_pos: [f32; 4] = "a_pos",
    a_tex_coord: [i8; 2] = "a_tex_coord",
});

impl Vertex {
    fn new(pos: [f32; 3], tex_coord: [i8; 2]) -> Vertex {
        Vertex {
            a_pos: [pos[0], pos[1], pos[2], 1.0],
            a_tex_coord: tex_coord,
        }
    }
}

gfx_pipeline!( pipe {
    vbuf: gfx::VertexBuffer<Vertex> = (),
    u_model_view_proj: gfx::Global<[[f32; 4]; 4]> = "u_model_view_proj",
    t_color: gfx::TextureSampler<[f32; 4]> = "t_color",
    out_color: gfx::RenderTarget<::gfx::format::Srgba8> = "o_Color",
    out_depth: gfx::DepthTarget<::gfx::format::DepthStencil> =
        gfx::preset::depth::LESS_EQUAL_WRITE,
});

fn main() {
    let mut window: PistonWindow<Sdl2Window> = WindowSettings::new("Gaia", [960, 520])
        .exit_on_esc(true)
        .samples(4)
        .opengl(OpenGL::V3_2)
        .build()
        .unwrap();

    window.set_capture_cursor(true);

    let ref mut factory = window.factory.clone();

    let pso = factory.create_pipeline_simple(
        include_bytes!("shaders/foobar.glslv"),
        include_bytes!("shaders/foobar.glslf"),
        pipe::new(),
    ).unwrap();

    let (vertex_data, index_data) = get_vertex_data();
    let index_data: &[u32] = &index_data; // TODO do I really have to do this?
    let (vbuf, slice) = factory.create_vertex_buffer_with_slice(&vertex_data, index_data);

    let model = vecmath::mat4_id();
    let mut camera_controller = FirstPerson::new([0.0, 0.0, 5.0], FirstPersonSettings::keyboard_wasd());
    let projection = CameraPerspective {
        fov: 90.0,
        near_clip: 0.1,
        far_clip: 1000.0,
        aspect_ratio: 960.0 / 520.0,
    }.projection();

    let sampler_info = gfx::texture::SamplerInfo::new(
        gfx::texture::FilterMethod::Bilinear,
        gfx::texture::WrapMode::Clamp
    );

    let texels = [
        [0x00, 0xff, 0x00, 0x00],
        [0x00, 0xff, 0x00, 0x00],
        [0x00, 0x00, 0xff, 0x00],
        [0x00, 0x00, 0xff, 0x00],
    ];

    let (_, texture_view) = factory.create_texture_immutable::<gfx::format::Rgba8>(
        gfx::texture::Kind::D2(2, 2, gfx::texture::AaMode::Single),
        &[&texels]
    ).unwrap();

    let mut data = pipe::Data {
        vbuf: vbuf,
        u_model_view_proj: [[0.0; 4]; 4],
        t_color: (texture_view, factory.create_sampler(sampler_info)),
        out_color: window.output_color.clone(),
        out_depth: window.output_stencil.clone(),
    };

    while let Some(e) = window.next() {
        camera_controller.event(&e);

        window.draw_3d(&e, |window| {
            let args = e.render_args().unwrap();

            window.encoder.clear(&window.output_color, [0.3, 0.3, 0.3, 1.0]);
            window.encoder.clear_depth(&window.output_stencil, 1.0);

            data.u_model_view_proj = camera_controllers::model_view_projection(
                model,
                camera_controller.camera(args.ext_dt).orthogonal(),
                projection,
            );

            window.encoder.draw(&slice, &pso, &data);
        });
    }
}

fn get_vertex(x: f32, y: f32, z: f32) -> Vertex {
    let tex_coord = if z > 0.0 { [0, 0] } else { [1, 1] };

    Vertex::new([x / 100.0, y / 100.0, 0.0], tex_coord)
}

fn get_vertex_data() -> (Vec<Vertex>, Vec<u32>) {
    let height_data = get_elevation_data();
    // println!("{:?}", height_data);
    // let height_data = vec![
    //     vec![0.1, 0.5, 0.1],
    //     vec![0.2, -0.2, 0.0],
    // ];

    let height = height_data.len();
    let width = height_data[0].len();

    println!("{} {}", height, width);

    let mut vertex_data = Vec::new();
    let mut index_data = Vec::new();

    for y in 0..height - 1 {
        for x in 0..width - 1 {
            let top_left = height_data[y][x] / 1000.0;
            let top_right = height_data[y][x + 1] / 1000.0;
            let bot_left = height_data[y + 1][x] / 1000.0;
            let bot_right = height_data[y + 1][x + 1] / 1000.0;

            let next_index = vertex_data.len() as u32;

            let x = x as f32;
            let y = y as f32;

            vertex_data.push(get_vertex(x, -y, top_left));
            vertex_data.push(get_vertex(x + 1.0, -y, top_right));
            vertex_data.push(get_vertex(x, -y - 1.0, bot_left));
            vertex_data.push(get_vertex(x + 1.0, -y - 1.0, bot_right));

            index_data.extend([next_index + 0, next_index + 1, next_index + 2].iter().cloned());
            index_data.extend([next_index + 1, next_index + 2, next_index + 3].iter().cloned());
        }
    }

    println!("{:?}", vertex_data.len());

    (vertex_data, index_data)
}
