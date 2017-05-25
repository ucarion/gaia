// extern crate byteorder;
// extern crate image;

// use std::io::{BufReader, Read};
// use std::fs::File;

// use byteorder::{LittleEndian, ReadBytesExt};

// fn main() {
//     let file = File::open("assets/noaa_globe/h10g").unwrap();
//     let mut file = BufReader::new(file);
//     let mut img_buf = image::ImageBuffer::new(10800, 6000);

//     let mut count = 0;

//     while let Ok(byte) = file.read_i16::<LittleEndian>() {
//         let (x, y) = (count % 10800, count / 10800);
//         let color = if byte > 0 { 0 } else { 255 };

//         img_buf.put_pixel(x, y, image::Rgb([color; 3]));

//         count += 1;
//     }

//     let mut img_out = File::create("out/out.png").unwrap();
//     image::ImageRgb8(img_buf).save(&mut img_out, image::PNG);

//     println!("Total count: {}", count);
// }

extern crate camera_controllers;
#[macro_use]
extern crate gfx;
extern crate piston_window;
extern crate sdl2_window;
extern crate vecmath;

use camera_controllers::{FirstPerson, FirstPersonSettings, CameraPerspective};
use gfx::traits::*;
use piston_window::*;
use sdl2_window::Sdl2Window;

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
    let mut window: PistonWindow<Sdl2Window> = WindowSettings::new("Gaia", [640, 480])
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
    let index_data: &[u16] = &index_data; // TODO do I really have to do this?
    let (vbuf, slice) = factory.create_vertex_buffer_with_slice(&vertex_data, index_data);

    let model = vecmath::mat4_id();
    let mut camera_controller = FirstPerson::new([0.0, 0.0, 4.0], FirstPersonSettings::keyboard_wasd());
    let projection = CameraPerspective {
        fov: 90.0,
        near_clip: 0.1,
        far_clip: 1000.0,
        aspect_ratio: 640.0 / 480.0
    }.projection();

    let sampler_info = gfx::texture::SamplerInfo::new(
        gfx::texture::FilterMethod::Bilinear,
        gfx::texture::WrapMode::Clamp
    );

    let texels = [
        [0xff, 0xff, 0xff, 0x00],
        [0xff, 0x00, 0x00, 0x00],
        [0x00, 0xff, 0x00, 0x00],
        [0x00, 0x00, 0xff, 0x00]
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

fn get_vertex_data() -> (Vec<Vertex>, Vec<u16>) {
    let height_data = vec![
        vec![0.0, 0.0, 0.0],
        vec![0.5, 1.0, 0.0],
        vec![0.0, 0.5, 0.5],
        vec![0.0, 0.5, 0.0],
    ];

    let height = height_data.len();
    let width = height_data[0].len();

    let mut vertex_data = Vec::new();
    let mut index_data = Vec::new();

    for y in 0..height - 1 {
        for x in 0..width - 1 {
            let top_left = height_data[y][x];
            let top_right = height_data[y][x + 1];
            let bot_left = height_data[y + 1][x];
            let bot_right = height_data[y + 1][x + 1];

            let next_index = vertex_data.len() as u16;

            let x = x as f32;
            let y = y as f32;

            vertex_data.push(Vertex::new([x, -y, top_left], [0, 0]));
            vertex_data.push(Vertex::new([x + 1.0, -y, top_right], [0, 1]));
            vertex_data.push(Vertex::new([x, -y - 1.0, bot_left], [1, 0]));
            vertex_data.push(Vertex::new([x + 1.0, -y - 1.0, bot_right], [1, 1]));

            index_data.extend([next_index + 0, next_index + 1, next_index + 2].iter().cloned());
            index_data.extend([next_index + 1, next_index + 2, next_index + 3].iter().cloned());
        }
    }

    println!("{:?}", vertex_data);

    (vertex_data, index_data)
}
