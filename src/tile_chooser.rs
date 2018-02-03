use cgmath::{Matrix4, Vector2};
use collision::{Aabb3, Frustum, Relation};
use gaia_assetgen::ELEVATION_TILE_SIZE;
use gaia_quadtree::{PositionInParent, Tile};
use gfx;
use lru_cache::LruCache;

use constants::Z_UPPER_BOUND;
use tile_asset_getter::TileAssets;

/// Gets tiles that can be rendered immediately, and tiles that should be fetched.
///
/// `tiles_to_render` contains pairs of positioned tiles, and the indices to use for that tile.
/// These tiles are already in cache, and can be rendered immediately.
///
/// `tiles_to_fetch` is the desired tiles for the current camera position that are not in cache.
/// These should be fetched and put into cache, so that future calls to this function can use them.
pub fn choose_tiles<R: gfx::Resources>(
    level_chooser: &Fn(f32) -> u8,
    texture_cache: &mut LruCache<Tile, TileAssets<R>>,
    mvp: Matrix4<f32>,
    look_at: Vector2<f32>,
    camera_height: f32,
) -> (u8, Vec<(Tile, Vec<u32>)>, Vec<Tile>) {
    let mut tiles_to_render = vec![];
    let mut tiles_to_fetch = vec![];

    let desired_level = level_chooser(camera_height);

    for desired_tile in desired_tiles(desired_level, look_at, mvp) {
        if !texture_cache.contains_key(&desired_tile.to_origin()) {
            tiles_to_fetch.push(desired_tile.clone());
        }

        if let Some(tile_and_indices) = get_covering_tile(texture_cache, desired_tile) {
            tiles_to_render.push(tile_and_indices);
        }
    }

    (desired_level, tiles_to_render, tiles_to_fetch)
}

fn desired_tiles(desired_level: u8, look_at: Vector2<f32>, mvp: Matrix4<f32>) -> Vec<Tile> {
    let frustum = Frustum::from_matrix4(mvp).unwrap();
    let center = Tile::enclosing_point(desired_level, look_at.into());

    let mut result = Vec::new();

    // TODO this is coupled with how the camera controller works; a fully correct solution being
    // rather complicated, instead just document this behavior? (and export as a constant?)
    let num_tiles_around = 6;

    for delta_x in -num_tiles_around..num_tiles_around {
        for delta_y in -num_tiles_around..num_tiles_around {
            let tile = center.offset_by(delta_x, delta_y);

            if tile_in_frustum(&tile, &frustum) && !result.contains(&tile) {
                result.push(tile);
            }
        }
    }

    result
}

fn tile_in_frustum(tile: &Tile, frustum: &Frustum<f32>) -> bool {
    let a = tile.top_left_position();
    let b = tile.bottom_right_position();
    let bounding_box = Aabb3::new([a[0], a[1], 0.0].into(), [b[0], b[1], Z_UPPER_BOUND].into());

    frustum.contains(&bounding_box) != Relation::Out
}

fn get_covering_tile<R: gfx::Resources>(
    cache: &mut LruCache<Tile, TileAssets<R>>,
    tile_to_cover: Tile,
) -> Option<(Tile, Vec<u32>)> {
    find_parent_in_cache(tile_to_cover, cache).and_then(|(parent, quadrant_positions)| {
        let (mut width, mut left_x, mut top_y) = (ELEVATION_TILE_SIZE - 1, 0, 0);

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
                let a = (x + 0) + (y + 0) * ELEVATION_TILE_SIZE;
                let b = (x + 0) + (y + 1) * ELEVATION_TILE_SIZE;
                let c = (x + 1) + (y + 0) * ELEVATION_TILE_SIZE;
                let d = (x + 1) + (y + 1) * ELEVATION_TILE_SIZE;

                index_data.extend_from_slice(&[a, b, c, c, b, d]);
            }
        }

        Some((parent, index_data))
    })
}

fn find_parent_in_cache<R: gfx::Resources>(
    mut tile: Tile,
    cache: &mut LruCache<Tile, TileAssets<R>>,
) -> Option<(Tile, Vec<PositionInParent>)> {
    let mut quadrant_positions = vec![];

    loop {
        if cache.contains_key(&tile.to_origin()) {
            return Some((tile, quadrant_positions));
        }

        if let Some(parent) = tile.parent() {
            quadrant_positions.push(tile.position_in_parent().unwrap());
            tile = parent;
        } else {
            return None;
        }
    }
}
