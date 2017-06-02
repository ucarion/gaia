extern crate byteorder;
extern crate cam;
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
        .samples(4)
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
    let vertex_tree = VertexTree::new(elevation_data);
    let (vertex_data, _index_data) = vertex_tree.get_vertex_data();
    println!("Done generating vertices.");

    println!("Generating indexes...");
    let index_data = index_getter::get_indices(camera_controller.camera_position(), [0.0, 0.0]);
    let index_data: &[u32] = &index_data; // TODO do I really have to do this?
    let (vbuf, slice) = factory.create_vertex_buffer_with_slice(&vertex_data, index_data);
    println!("Done generating indices.");

    println!("{} {:?}", vertex_data.len(), index_data);

    let sampler_info = gfx::texture::SamplerInfo::new(
        gfx::texture::FilterMethod::Bilinear,
        gfx::texture::WrapMode::Clamp
    );

    println!("Generating texture");
    let texels = get_texels();

    let (_, texture_view) = factory.create_texture_immutable::<gfx::format::Rgba8>(
        gfx::texture::Kind::D2(4097, 4097, gfx::texture::AaMode::Single),
        &[&texels]
    ).unwrap();
    println!("Done generating texture");

    let mut data = pipe::Data {
        vbuf: vbuf,
        u_model_view_proj: [[0.0; 4]; 4],
        u_offset_x: 0.0,
        t_color: (texture_view, factory.create_sampler(sampler_info)),
        out_color: window.output_color.clone(),
        out_depth: window.output_stencil.clone(),
    };

    while let Some(e) = window.next() {
        camera_controller.event(&e);

        window.draw_3d(&e, |window| {
            window.encoder.clear(&window.output_color, [0.3, 0.3, 0.3, 1.0]);
            window.encoder.clear_depth(&window.output_stencil, 1.0);

            data.u_model_view_proj = cam::model_view_projection(
                model,
                camera_controller.view_matrix(),
                get_projection(&window),
            );

            let indices = index_getter::get_indices(camera_controller.camera_position(), [0.0, 0.0]);
            let index_buffer = factory.create_index_buffer(indices.as_slice());
            let slice = gfx::Slice {
                start: 0,
                end: indices.len() as u32,
                base_vertex: 0,
                instances: None,
                buffer: index_buffer,
            };

            println!("{}", indices.len());

            data.u_offset_x = 0.0;
            window.encoder.draw(&slice, &pso, &data);
        });

        e.resize(|_, _| {
            data.out_color = window.output_color.clone();
            data.out_depth = window.output_stencil.clone();
        });
    }
}

fn get_texels() -> Vec<[u8; 4]> {
    let world_image = image::open("assets/east_hemisphere.jpg").unwrap();
    println!("{:?}", world_image.dimensions());

    let mut result = Vec::new();

    for (_, _, rgba) in world_image.pixels() {
        result.push(rgba.data);
    }

    result
}

fn get_vertex(x: usize, y: usize, elevation: i16) -> Vertex {
    let tex_coord = [x as f32 / 4097.0, y as f32 / 4097.0];
    let elevation = elevation - 500;
    let z = if elevation <= 0 {
        0.0
    } else {
        elevation as f32 / 200.0
    };

    Vertex::new([x as f32, -(y as f32), z], tex_coord)
}

struct VertexTree {
    elevation_data: Vec<Vec<i16>>,
}

impl VertexTree {
    fn new(elevation_data: Vec<Vec<i16>>) -> VertexTree {
        VertexTree { elevation_data: elevation_data }
    }

    fn get_vertex_data(&self) -> (Vec<Vertex>, Vec<u32>) {
        let height = self.elevation_data.len();
        let width = self.elevation_data[0].len();

        let mut vertex_data = Vec::new();
        let mut index_data = Vec::new();

        for y in 0..height {
            for x in 0..width {
                vertex_data.push(get_vertex(x, y, self.elevation_data[y][x]));
                // let top_left  = self.elevation_data[y + 0][x + 0];
                // let top_right = self.elevation_data[y + 0][x + 1];
                // let bot_left  = self.elevation_data[y + 1][x + 0];
                // let bot_right = self.elevation_data[y + 1][x + 1];

                // let next_index = vertex_data.len() as u32;

                // vertex_data.push(get_vertex(x + 0, y + 0, top_left));
                // vertex_data.push(get_vertex(x + 1, y + 0, top_right));
                // vertex_data.push(get_vertex(x + 0, y + 1, bot_left));
                // vertex_data.push(get_vertex(x + 1, y + 1, bot_right));

                // index_data.extend([next_index + 0, next_index + 1, next_index + 2].iter().cloned());
                // index_data.extend([next_index + 1, next_index + 2, next_index + 3].iter().cloned());
            }
        }

        (vertex_data, index_data)
    }
}

