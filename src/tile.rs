use constants::LEVEL0_TILE_WIDTH;

use gfx;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
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
        LEVEL0_TILE_WIDTH * 2.0f32.powi(level as i32)
    }

    /// For a given level, the number of tiles across the width of the map.
    pub fn num_tiles_across_level(level: u8) -> u8 {
        128 / 2u8.pow(level as u32)
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
        let offset_x = (x / width).floor() as i64;
        let offset_y = (-y / width).floor() as i64;

        Self::from_level_and_position(level, [offset_x, offset_y])
    }

    pub fn from_level_and_position(level: u8, position: [i64; 2]) -> PositionedTile {
        let tile_x = modulo(position[0], Tile::num_tiles_across_level(level));
        let tile_y = modulo(position[1], Tile::num_tiles_across_level(level) / 2);

        PositionedTile {
            tile: Tile {
                level: level,
                x: tile_x,
                y: tile_y,
            },
            position: position,
        }
    }

    pub fn offset(&self) -> [f32; 2] {
        let width = self.tile.width();
        [
            self.position[0] as f32 * width,
            self.position[1] as f32 * width,
        ]
    }

    pub fn parent(&self) -> PositionedTile {
        PositionedTile {
            tile: self.tile.parent(),
            position: [(self.position[0] as f64 / 2.0).floor() as i64, self.position[1] / 2],
        }
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

pub struct TileTextures<R: gfx::Resources> {
    pub color: gfx::handle::ShaderResourceView<R, [f32; 4]>,
    pub elevation: gfx::handle::ShaderResourceView<R, u32>,
}
