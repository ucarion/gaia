use std::cmp;

use collision::Frustum;
use gfx;
use lru_cache::LruCache;

use constants::{ELEVATION_TILE_WIDTH, MAX_TILE_LEVEL};
use texture_getter::TileTextures;
use tile::{PositionedTile, PositionInParent, Tile};

/// Gets tiles that can be rendered immediately, and tiles that should be fetched.
///
/// The first list contains pairs of positioned tiles, and the indices to use for that tile. These
/// tiles are already in cache, and can be rendered immediately.
///
/// The second list is the desired tiles for the current camera position. These should be fetched
/// and put into cache, so that future calls to this function can use them.
pub fn choose_tiles<R: gfx::Resources>(
    camera_position: [f32; 3],
    mvp: [[f32; 4]; 4],
    texture_cache: &mut LruCache<Tile, TileTextures<R>>,
) -> (Vec<(PositionedTile, Vec<u16>)>, Vec<Tile>) {
    let mut tiles_to_render = vec![];
    let mut tiles_to_fetch = vec![];

    for desired_tile in desired_tiles(camera_position, mvp) {
        if !texture_cache.contains_key(&desired_tile.tile) {
            tiles_to_fetch.push(desired_tile.tile.clone());
        }

        if let Some(tile_and_indices) = get_covering_tile(texture_cache, desired_tile.clone()) {
            tiles_to_render.push(tile_and_indices);
        }
    }

    (tiles_to_render, tiles_to_fetch)
}

fn desired_tiles(camera_position: [f32; 3], mvp: [[f32; 4]; 4]) -> Vec<PositionedTile> {
    let frustum = Frustum::from_matrix4(mvp.into()).unwrap();
    let desired_level = match camera_position[2] {
        0.0...100.0 => 0,
        100.0...300.0 => 1,
        300.0...600.0 => 2,
        600.0...800.0 => 3,
        800.0...1000.0 => 4,
        _ => 5,
    };

    let center =
        PositionedTile::enclosing_point(desired_level, camera_position[0], camera_position[1]);

    let center_x = center.position[0];
    let center_y = center.position[1];

    // TODO this is coupled with how the camera controller works; a fully correct solution being
    // rather complicated, instead just document this behavior? (and export as a constant?)
    let num_tiles_around = 7;
    let min_x = center_x - num_tiles_around;
    let max_x = center_x + num_tiles_around;
    let min_y = cmp::max(0, center_y - num_tiles_around);
    let max_y = cmp::min(
        Tile::num_tiles_across_level_height(desired_level) as i64 - 1,
        center_y + num_tiles_around,
    );

    let mut result = vec![];
    for tile_x in min_x..max_x + 1 {
        for tile_y in min_y..max_y + 1 {
            let tile = PositionedTile::from_level_and_position(desired_level, [tile_x, tile_y]);

            if tile.is_in_frustum(&frustum) {
                result.push(tile);
            }
        }
    }

    result
}

fn get_covering_tile<R: gfx::Resources>(
    cache: &mut LruCache<Tile, TileTextures<R>>,
    tile_to_cover: PositionedTile,
) -> Option<(PositionedTile, Vec<u16>)> {
    find_parent_in_cache(tile_to_cover, cache).and_then(|(parent, quadrant_positions)| {
        let (mut width, mut left_x, mut top_y) = (ELEVATION_TILE_WIDTH - 1, 0, 0);

        for position in quadrant_positions.iter().rev() {
            width = width / 2;

            let (next_left_x, next_top_y) = match *position {
                PositionInParent::TopLeft => (left_x, top_y),
                PositionInParent::TopRight => (left_x + width, top_y),
                PositionInParent::BottomLeft => (left_x, top_y + width),
                PositionInParent::BottomRight => (left_x + width, top_y + width),
            };

            left_x = next_left_x;
            top_y = next_top_y;
        }

        let mut index_data = Vec::new();
        for x in left_x..left_x + width {
            for y in top_y..top_y + width {
                let a = (x + 0) + (y + 0) * ELEVATION_TILE_WIDTH;
                let b = (x + 0) + (y + 1) * ELEVATION_TILE_WIDTH;
                let c = (x + 1) + (y + 0) * ELEVATION_TILE_WIDTH;
                let d = (x + 1) + (y + 1) * ELEVATION_TILE_WIDTH;

                index_data.extend_from_slice(&[a, b, c, c, b, d]);
            }
        }

        Some((parent, index_data))
    })
}

fn find_parent_in_cache<R: gfx::Resources>(
    mut tile: PositionedTile,
    cache: &mut LruCache<Tile, TileTextures<R>>,
) -> Option<(PositionedTile, Vec<PositionInParent>)> {
    let mut quadrant_positions = vec![];

    loop {
        if cache.contains_key(&tile.tile) {
            return Some((tile, quadrant_positions));
        }

        if tile.tile.level == MAX_TILE_LEVEL {
            return None;
        } else {
            quadrant_positions.push(tile.tile.position_in_parent());
            tile = tile.parent();
        }
    }
}
