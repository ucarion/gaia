use std::collections::HashMap;
use std::io::Cursor;

use constants::{ELEVATION_DATA_OFFSET, VERTEX_GRID_SIDE_LENGTH};
use tile::TileKind;
use vertex::Vertex;

use byteorder::{LittleEndian, ReadBytesExt};

/// Scale an elevation in meters into a "z" value by dividing by this amount.
const ELEVATION_SCALING_FACTOR: f32 = 200.0;

pub fn get_vertices() -> HashMap<TileKind, Vec<Vertex>> {
    let mut result = HashMap::new();
    result.insert(
        TileKind::WestHemisphere,
        get_vertices_for_tile(
            include_bytes!("../assets/generated/tiles/west_hemisphere/elevation.bin"),
            VERTEX_GRID_SIDE_LENGTH,
            VERTEX_GRID_SIDE_LENGTH,
        ),
    );
    result.insert(
        TileKind::EastHemisphere,
        get_vertices_for_tile(
            include_bytes!("../assets/generated/tiles/east_hemisphere/elevation.bin"),
            VERTEX_GRID_SIDE_LENGTH,
            VERTEX_GRID_SIDE_LENGTH,
        ),
    );
    result
}

fn get_vertices_for_tile(elevation_data: &[u8], width: u32, height: u32) -> Vec<Vertex> {
    let mut result = Vec::new();
    add_vertices(&mut result, elevation_data, width, height);
    result
}

fn add_vertices(buf: &mut Vec<Vertex>, elevation_data: &[u8], width: u32, height: u32) {
    let mut cursor = Cursor::new(elevation_data);
    let mut count = 0;

    while let Ok(elevation) = cursor.read_u16::<LittleEndian>() {
        let (x, y) = (count % width, count / width);

        let actual_elevation = elevation as i16 - ELEVATION_DATA_OFFSET as i16;
        buf.push(get_vertex(width, height, x, y, actual_elevation));

        count += 1;
    }
}

fn get_vertex(width: u32, height: u32, x: u32, y: u32, elevation: i16) -> Vertex {
    let tex_coord = [x as f32 / width as f32, y as f32 / height as f32];

    Vertex::new([x as f32, -(y as f32), get_z(elevation)], tex_coord)
}

fn get_z(elevation: i16) -> f32 {
    if elevation <= 0 {
        0.0
    } else {
        elevation as f32 / ELEVATION_SCALING_FACTOR
    }
}
