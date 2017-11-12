use collision::{Aabb3, Frustum, Relation};

use constants::{LEVEL0_TILE_WIDTH, Z_UPPER_BOUND, WORLD_HEIGHT};

#[derive(Clone, Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct Tile {
    pub level: u8,
    pub x: u8,
    pub y: u8,
}

#[derive(Debug)]
pub enum PositionInParent {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl Tile {
    /// The displayed width of tiles at a given level. Higher levels cover a greater area.
    pub fn level_tile_width(level: u8) -> f32 {
        *LEVEL0_TILE_WIDTH * 2.0f32.powi(level as i32)
    }

    /// For a given level, the number of tiles across the width of the map.
    pub fn num_tiles_across_level_width(level: u8) -> u8 {
        128 / 2u8.pow(level as u32)
    }

    pub fn num_tiles_across_level_height(level: u8) -> u8 {
        Self::num_tiles_across_level_width(level) / 2
    }

    pub fn width(&self) -> f32 {
        Self::level_tile_width(self.level)
    }

    pub fn parent(&self) -> Tile {
        Tile {
            level: self.level + 1,
            x: self.x / 2,
            y: self.y / 2,
        }
    }

    pub fn position_in_parent(&self) -> PositionInParent {
        match (self.x % 2 == 0, self.y % 2 == 0) {
            (true, true) => PositionInParent::TopLeft,
            (false, true) => PositionInParent::TopRight,
            (true, false) => PositionInParent::BottomLeft,
            (false, false) => PositionInParent::BottomRight,
        }
    }
}

/// A regular tile is a key for asset data, but could be anywhere in the world (since the map
/// tesselates infinitely east and west). A `PositionedTile` is a `Tile`, but also can calculate
/// its position relative to the origin.
///
/// The `position` is that of its top-left corner, in units of its own width.
#[derive(Clone, Debug)]
pub struct PositionedTile {
    pub tile: Tile,
    pub position: [i64; 2],
}

impl PositionedTile {
    pub fn enclosing_point(level: u8, x: f32, y: f32) -> PositionedTile {
        let width = Tile::level_tile_width(level);
        let offset_x = (x / width).floor();
        let offset_y = Tile::num_tiles_across_level_height(level) - (y / width).floor() as u8;

        Self::from_level_and_position(level, [offset_x as i64, offset_y as i64])
    }

    pub fn from_level_and_position(level: u8, position: [i64; 2]) -> PositionedTile {
        let tile_x = modulo(position[0], Tile::num_tiles_across_level_width(level));
        let tile_y = modulo(position[1], Tile::num_tiles_across_level_height(level));

        PositionedTile {
            tile: Tile {
                level: level,
                x: tile_x,
                y: tile_y,
            },
            position: position,
        }
    }

    pub fn offset(&self) -> [f32; 3] {
        [
            self.position[0] as f32 * self.tile.width(),
            WORLD_HEIGHT - self.position[1] as f32 * self.tile.width(),
            0.0,
        ]
    }

    pub fn parent(&self) -> PositionedTile {
        PositionedTile {
            tile: self.tile.parent(),
            position: [
                (self.position[0] as f64 / 2.0).floor() as i64,
                self.position[1] / 2,
            ],
        }
    }

    pub fn is_in_frustum(&self, frustum: &Frustum<f32>) -> bool {
        let top_left = self.offset();
        let point1 = [top_left[0], top_left[1], 0.0];
        let point2 = [
            point1[0] + self.tile.width(),
            point1[1] - self.tile.width(),
            Z_UPPER_BOUND,
        ];

        let bounding_box = Aabb3::new(point1.into(), point2.into());
        frustum.contains(&bounding_box) != Relation::Out
    }
}

fn modulo(a: i64, b: u8) -> u8 {
    let rem = a % b as i64;
    if rem < 0 {
        (rem + b as i64) as u8
    } else {
        rem as u8
    }
}
