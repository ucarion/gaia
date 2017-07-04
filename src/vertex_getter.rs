use std::io::Cursor;

use constants::{ELEVATION_DATA_OFFSET, VERTEX_GRID_SIDE_LENGTH};
use vertex::Vertex;

use byteorder::{LittleEndian, ReadBytesExt};

/// Scale an elevation in meters into a "z" value by dividing by this amount.
const ELEVATION_SCALING_FACTOR: f32 = 100.0;

pub fn get_vertices() -> Vec<Vertex> {
    let mut result = Vec::new();
    add_vertices(
        &mut result,
        include_bytes!("../assets/generated/west_hemisphere_elevation.bin"),
    );
    add_vertices(
        &mut result,
        include_bytes!("../assets/generated/east_hemisphere_elevation.bin"),
    );
    result
}

fn add_vertices(buf: &mut Vec<Vertex>, elevation_data: &[u8]) {
    let mut cursor = Cursor::new(elevation_data);
    let mut count = 0;

    while let Ok(elevation) = cursor.read_u16::<LittleEndian>() {
        let (x, y) = (
            count % VERTEX_GRID_SIDE_LENGTH,
            count / VERTEX_GRID_SIDE_LENGTH,
        );

        let actual_elevation = elevation as i16 - ELEVATION_DATA_OFFSET as i16;
        buf.push(get_vertex(x, y, actual_elevation));

        count += 1;
    }
}

fn get_vertex(x: u32, y: u32, elevation: i16) -> Vertex {
    let tex_coord = [
        x as f32 / VERTEX_GRID_SIDE_LENGTH as f32,
        y as f32 / VERTEX_GRID_SIDE_LENGTH as f32,
    ];

    Vertex::new([x as f32, -(y as f32), get_z(elevation)], tex_coord)
}

fn get_z(elevation: i16) -> f32 {
    if elevation <= 0 {
        0.0
    } else {
        elevation as f32 / ELEVATION_SCALING_FACTOR
    }
}
