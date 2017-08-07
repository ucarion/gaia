use gfx;
use lru_cache::LruCache;

use tile::{PositionedTile, Tile, TileTextures};

/// Gets tiles that can be rendered immediately, and tiles that should be fetched.
///
/// The first list contains pairs of positioned tiles, and the indices to use for that tile. These
/// tiles are already in cache, and can be rendered immediately.
///
/// The second list is the desired tiles for the current camera position. These should be fetched
/// and put into cache, so that future calls to this function can use them.
pub fn get_tiles<R: gfx::Resources>(
    camera_position: [f32; 3],
    texture_cache: &mut LruCache<Tile, TileTextures<R>>,
) -> (Vec<(PositionedTile, Vec<u16>)>, Vec<Tile>) {
    let mut tiles_to_render = vec![];
    let mut tiles_to_fetch = vec![];

    let mut index_data = vec![];
    for x in 0..128u16 {
        for y in 0..128u16 {
            if x != 127 && y != 127 {
                let index = (x + 0) + (y + 0) * 128;
                let right = (x + 1) + (y + 0) * 128;
                let below = (x + 0) + (y + 1) * 128;
                let below_right = (x + 1) + (y + 1) * 128;
                index_data.extend_from_slice(&[index, below, right, right, below, below_right]);
            }
        }
    }


    for desired_tile in desired_tiles(camera_position) {
        if texture_cache.contains_key(&desired_tile.tile) {
            tiles_to_render.push((desired_tile, index_data.clone()));
        } else {
            tiles_to_fetch.push(desired_tile.tile);
        }
    }

    (tiles_to_render, tiles_to_fetch)
}

fn desired_tiles(camera_position: [f32; 3]) -> Vec<PositionedTile> {
    let desired_level: u8 = match camera_position[2] {
        // 0.0...100.0 => 0,
        // 100.0...200.0 => 1,
        000.0...300.0 => 2,
        300.0...400.0 => 3,
        400.0...500.0 => 4,
        500.0...600.0 => 5,
        _ => 6,
    };

    let center =
        PositionedTile::enclosing_point(desired_level, camera_position[0], camera_position[1]);

    let mut result = vec![];
    for delta_x in -3..4 {
        for delta_y in -3..4 {
            result.push(PositionedTile::from_level_and_position(
                desired_level,
                [center.position[0] + delta_x, center.position[1] + delta_y],
            ));
        }
    }

    result
}
