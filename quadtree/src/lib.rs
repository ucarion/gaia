extern crate num;

use num::Integer;

#[derive(PartialEq, Eq, Debug, Hash, Clone)]
pub struct Tile {
    pub offset: i16,
    pub level: u8,
    pub x: u8,
    pub y: u8,
}

#[derive(PartialEq, Eq, Debug, Hash, Clone)]
pub enum PositionInParent {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl Tile {
    pub fn new_at_origin(level: u8, x: u8, y: u8) -> Tile {
        Tile {
            offset: 0,
            level,
            x,
            y,
        }
    }

    pub fn enclosing_point(level: u8, position: [f32; 2]) -> Tile {
        let offset = (position[0] / 2.0).floor();

        let width = Self::level_width(level);
        let x = (position[0] - offset * 2.0) / width;
        let y = position[1] / width;

        Tile {
            offset: offset as i16,
            level,
            x: x as u8,
            y: y as u8,
        }
    }

    pub fn parent(&self) -> Option<Tile> {
        if self.level == 0 {
            return None;
        }

        Some(Tile {
            offset: self.offset,
            level: self.level - 1,
            x: self.x / 2,
            y: self.y / 2,
        })
    }

    pub fn position_in_parent(&self) -> Option<PositionInParent> {
        if self.level == 0 {
            return None;
        }

        Some(match (self.x % 2 == 0, self.y % 2 == 0) {
            (true, true) => PositionInParent::TopLeft,
            (false, true) => PositionInParent::TopRight,
            (true, false) => PositionInParent::BottomLeft,
            (false, false) => PositionInParent::BottomRight,
        })
    }

    pub fn width(&self) -> f32 {
        Self::level_width(self.level)
    }

    pub fn bottom_left_position(&self) -> [f32; 2] {
        [
            2.0 * self.offset as f32 + self.x as f32 * self.width(),
            self.y as f32 * self.width(),
        ]
    }

    pub fn bottom_right_position(&self) -> [f32; 2] {
        let bottom_left = self.bottom_left_position();
        let width = self.width();

        [bottom_left[0] + width, bottom_left[1]]
    }

    pub fn top_left_position(&self) -> [f32; 2] {
        let bottom_left = self.bottom_left_position();
        let width = self.width();

        [bottom_left[0], bottom_left[1] + width]
    }

    pub fn top_right_position(&self) -> [f32; 2] {
        let bottom_left = self.bottom_left_position();
        let width = self.width();

        [bottom_left[0] + width, bottom_left[1] + width]
    }

    /// The width of tiles at level `level`.
    pub fn level_width(level: u8) -> f32 {
        1.0 / 2.0f32.powi(level as i32)
    }

    pub fn tiles_across_width(level: u8) -> u8 {
        2u8.pow(level as u32 + 1)
    }

    pub fn tiles_across_height(level: u8) -> u8 {
        Self::tiles_across_width(level) / 2
    }

    /// Get this tile's neighbor that is `x` tiles to the left/right and `y` tiles to the
    /// top/bottom.
    ///
    /// The y-axis values will be clamped according to the number of tiles along the vertical axis.
    /// The x-axis values will wrap, incrementing or decrementing the `offset` depending on the
    /// direction of the wrap.
    pub fn offset_by(&self, x: i16, y: i16) -> Tile {
        let overflow_x = self.x as i16 + x;
        let overflow_y = self.y as i16 + y;

        let width = Self::tiles_across_width(self.level) as i16;
        let height = Self::tiles_across_height(self.level) as i16;

        let offset = self.offset + overflow_x.div_floor(&width);
        let x = overflow_x.mod_floor(&width) as u8;
        let y = num::clamp(overflow_y, 0, height - 1) as u8;

        Tile {
            offset,
            level: self.level,
            x,
            y,
        }
    }

    /// This tile, but with `offset` set to zero.
    pub fn to_origin(&self) -> Tile {
        Tile {
            offset: 0,
            level: self.level,
            x: self.x,
            y: self.y,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parent() {
        assert_eq!(
            None,
            Tile {
                offset: 0,
                level: 0,
                x: 0,
                y: 0,
            }.parent()
        );

        assert_eq!(
            Some(Tile {
                offset: -123,
                level: 2,
                x: 4,
                y: 2,
            }),
            Tile {
                offset: -123,
                level: 3,
                x: 9,
                y: 4,
            }.parent()
        );
    }

    #[test]
    fn position_in_parent() {
        assert_eq!(
            None,
            Tile {
                offset: 0,
                level: 0,
                x: 0,
                y: 0,
            }.position_in_parent()
        );

        assert_eq!(
            Some(PositionInParent::TopRight),
            Tile {
                offset: -123,
                level: 3,
                x: 9,
                y: 4,
            }.position_in_parent()
        );
    }

    #[test]
    fn width() {
        assert_eq!(
            1.0,
            Tile {
                offset: 0,
                level: 0,
                x: 0,
                y: 0,
            }.width()
        );

        assert_eq!(
            0.125,
            Tile {
                offset: -123,
                level: 3,
                x: 9,
                y: 4,
            }.width()
        );
    }

    #[test]
    fn position() {
        let tile_a = Tile {
            offset: 0,
            level: 0,
            x: 0,
            y: 0,
        };

        let tile_b = Tile {
            offset: -123,
            level: 3,
            x: 9,
            y: 4,
        };

        assert_eq!([0.0, 0.0], tile_a.bottom_left_position());
        assert_eq!([-244.875, 0.5], tile_b.bottom_left_position());
        assert_eq!([1.0, 0.0], tile_a.bottom_right_position());
        assert_eq!([-244.75, 0.5], tile_b.bottom_right_position());
        assert_eq!([0.0, 1.0], tile_a.top_left_position());
        assert_eq!([-244.875, 0.625], tile_b.top_left_position());
        assert_eq!([1.0, 1.0], tile_a.top_right_position());
        assert_eq!([-244.75, 0.625], tile_b.top_right_position());
    }

    #[test]
    fn enclosing_point() {
        assert_eq!(
            Tile {
                offset: 0,
                level: 0,
                x: 0,
                y: 0,
            },
            Tile::enclosing_point(0, [0.0, 0.0])
        );

        assert_eq!(
            Tile {
                offset: -123,
                level: 3,
                x: 9,
                y: 4,
            },
            Tile::enclosing_point(3, [-244.875, 0.5])
        );
    }

    #[test]
    fn tiles_across_width() {
        assert_eq!(2, Tile::tiles_across_width(0));
        assert_eq!(16, Tile::tiles_across_width(3));
    }

    #[test]
    fn tiles_across_height() {
        assert_eq!(1, Tile::tiles_across_height(0));
        assert_eq!(8, Tile::tiles_across_height(3));
    }

    #[test]
    fn offset_by() {
        assert_eq!(
            Tile {
                offset: 0,
                level: 1,
                x: 1,
                y: 1,
            },
            Tile::new_at_origin(1, 0, 0).offset_by(1, 1)
        );

        assert_eq!(
            Tile {
                offset: 1,
                level: 1,
                x: 1,
                y: 1,
            },
            Tile::new_at_origin(1, 0, 0).offset_by(5, 1)
        );

        assert_eq!(
            Tile {
                offset: -3,
                level: 1,
                x: 3,
                y: 0,
            },
            Tile::new_at_origin(1, 0, 0).offset_by(-9, -10)
        );
    }

    #[test]
    fn to_origin() {
        assert_eq!(
            Tile {
                offset: 0,
                level: 1,
                x: 1,
                y: 1,
            },
            Tile {
                offset: 1,
                level: 1,
                x: 1,
                y: 1,
            }.to_origin()
        )
    }
}
