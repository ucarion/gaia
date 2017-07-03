use std::io::BufReader;
use std::fs::File;

use vertex::Vertex;

use byteorder::{LittleEndian, ReadBytesExt};

pub fn get_vertices() -> Vec<Vertex> {
    get_vertices_from_elevation(get_elevation_data())
}

fn get_elevation_data() -> Vec<Vec<i16>> {
    println!("Getting elevation data...");
    let elev_data_width = 4097;
    let elev_data_height = 4097;

    let result_width = elev_data_width;
    let result_height = elev_data_height;

    let file = File::open("assets/west_hemisphere_elevation.bin").unwrap();
    let mut file = BufReader::new(file);

    let mut result = Vec::new();
    for _ in 0..result_height {
        result.push(vec![0; result_width]);
    }

    let mut count = 0;

    while let Ok(elevation) = file.read_i16::<LittleEndian>() {
        let (x, y) = (count % elev_data_width, count / elev_data_width);
        result[y][x] = elevation;
        count += 1;
    }

    println!("Done getting elevation data.");

    result
}

fn get_vertices_from_elevation(elevation_data: Vec<Vec<i16>>) -> Vec<Vertex> {
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

