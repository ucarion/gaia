use std::collections::HashMap;
use std::io::Cursor;

use constants::{ELEVATION_DATA_OFFSET, ELEVATION_GRID_WIDTH};
use tile::{TileKind, TILE_KINDS};
use vertex::Vertex;

use byteorder::{LittleEndian, ReadBytesExt};

/// Scale an elevation in meters into a "z" value by dividing by this amount.
const ELEVATION_SCALING_FACTOR: f32 = 200.0;

pub fn get_vertices() -> HashMap<TileKind, Vec<Vertex>> {
    let mut result = HashMap::new();

    for tile_kind in TILE_KINDS.iter() {
        result.insert(tile_kind.clone(), get_vertices_for_tile(tile_kind));
    }

    result
}

fn get_vertices_for_tile(tile_kind: &TileKind) -> Vec<Vertex> {
    let mut result = Vec::new();
    let mut cursor = Cursor::new(tile_kind.elevation_data());
    let mut count = 0;

    while let Ok(elevation) = cursor.read_u16::<LittleEndian>() {
        let (x, y) = (count % ELEVATION_GRID_WIDTH, count / ELEVATION_GRID_WIDTH);

        let actual_elevation = elevation as i16 - ELEVATION_DATA_OFFSET as i16;
        result.push(get_vertex(tile_kind, x, y, actual_elevation));

        count += 1;
    }
    result
}

fn get_vertex(tile_kind: &TileKind, x: u32, y: u32, elevation: i16) -> Vertex {
    let tex_coord = [
        x as f32 / ELEVATION_GRID_WIDTH as f32,
        y as f32 / tile_kind.elevation_grid_height() as f32,
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
