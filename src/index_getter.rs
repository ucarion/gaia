use collision::{Aabb3, Frustum, Relation};

use constants::VERTEX_GRID_SIDE_LENGTH;
use tile::{TileRenderInfo, TileKind};

/// The greatest possible `z` value any vertex may have.
///
/// TODO: Make this dynamically calculated.
const MAX_POSSIBLE_ELEVATION: f32 = 300.0;

/// Returns a list of `TileRenderInfo`s for potentially visible tiles.
pub fn get_indices_and_offsets(
    mvp_matrix: [[f32; 4]; 4],
    camera_pos: [f32; 3],
) -> Vec<TileRenderInfo> {
    let frustum = Frustum::from_matrix4(mvp_matrix.into()).unwrap();
    let (camera_x, camera_z) = (camera_pos[0], camera_pos[2]);

    let max_depth = match camera_z {
        0.0...300.0 => 9,
        300.0...500.0 => 8,
        500.0...650.0 => 7,
        650.0...1200.0 => 6,
        1200.0...2000.0 => 5,
        _ => 4,
    };

    let middle_tile_index = (camera_x / VERTEX_GRID_SIDE_LENGTH as f32).floor() as i64;
    let middle_is_west = middle_tile_index % 2 == 0;

    let left_offset = (middle_tile_index - 1) as f32 * VERTEX_GRID_SIDE_LENGTH as f32;
    let middle_offset = middle_tile_index as f32 * VERTEX_GRID_SIDE_LENGTH as f32;
    let right_offset = (middle_tile_index + 1) as f32 * VERTEX_GRID_SIDE_LENGTH as f32;

    let tiles = vec![
        (
            if middle_is_west {
                TileKind::EastHemisphere
            } else {
                TileKind::WestHemisphere
            },
            left_offset,
            VERTEX_GRID_SIDE_LENGTH,
        ),
        (
            if middle_is_west {
                TileKind::Meridian180
            } else {
                TileKind::Meridian0
            },
            middle_offset - 1.0,
            2,
        ),
        (
            if middle_is_west {
                TileKind::WestHemisphere
            } else {
                TileKind::EastHemisphere
            },
            middle_offset,
            VERTEX_GRID_SIDE_LENGTH,
        ),
        (
            if middle_is_west {
                TileKind::Meridian0
            } else {
                TileKind::Meridian180
            },
            right_offset - 1.0,
            2,
        ),
        (
            if middle_is_west {
                TileKind::EastHemisphere
            } else {
                TileKind::WestHemisphere
            },
            right_offset,
            VERTEX_GRID_SIDE_LENGTH,
        ),
    ];

    tiles
        .into_iter()
        .map(|(kind, x_offset, grid_width)| {
            get_tile_index_and_offset(&frustum, max_depth, kind, x_offset, grid_width)
        })
        .collect()
}

fn get_tile_index_and_offset(
    frustum: &Frustum<f32>,
    max_depth: usize,
    kind: TileKind,
    x_offset: f32,
    grid_width: u32,
) -> TileRenderInfo {
    let top_left = [x_offset, 0.0];
    let indices = get_indices(frustum, max_depth, top_left, grid_width);

    TileRenderInfo {
        indices: indices,
        x_offset: x_offset,
        kind: kind,
    }
}

fn get_indices(
    frustum: &Frustum<f32>,
    max_depth: usize,
    top_left: [f32; 2],
    grid_width: u32,
) -> Vec<u32> {
    let full_rectangle = Rectangle::full_rectangle(top_left, grid_width, VERTEX_GRID_SIDE_LENGTH);

    let mut result = Vec::new();
    append_indices(&mut result, frustum, max_depth, 0, &full_rectangle, true);
    result
}

fn append_indices(
    buf: &mut Vec<u32>,
    frustum: &Frustum<f32>,
    max_depth: usize,
    current_depth: usize,
    rectangle: &Rectangle,
    try_culling: bool,
) {
    if current_depth == max_depth {
        // we've reached the recursion limit, so return the current rectangle's indices
        buf.extend_from_slice(&rectangle.indices());
        return;
    }

    // There are three possible relations between the bounding box surrounding this rectangle and
    // the view frustum:
    //
    // * It is entirely outside the frustum: The rectangle cannot be seen, and no indices from it
    // or any sub-rectangle are necessary.
    //
    // * It crosses the frustum: The rectangle can partially be seen. Indices from it are
    // necessary, but some sub-rectangles might be entirely outside the frustum.
    //
    // * It is entirely within the frustum: The rectangle can entirely be seen. Indices from it are
    // necessary, and there's no chance of any sub-rectangle crossing or falling beyond the
    // frustum.
    let (cull_rectangle, try_culling_sub_rectangles) = if try_culling {
        let relation = rectangle.relate_to_frustum(frustum);
        (relation == Relation::Out, relation == Relation::Cross)
    } else {
        (false, false)
    };

    if cull_rectangle {
        return;
    }

    match rectangle.sub_rectangles() {
        Some(sub_rectangles) => {
            // recursively get indices from sub-rectangles
            for sub_rectangle in sub_rectangles {
                append_indices(
                    buf,
                    frustum,
                    max_depth,
                    current_depth + 1,
                    &sub_rectangle,
                    try_culling_sub_rectangles,
                );
            }
        }

        None => {
            // there are no sub-rectangles to work with, so return current rectangle instead
            buf.extend_from_slice(&rectangle.indices());
        }
    }
}

#[derive(Debug)]
struct Rectangle {
    top_left: [f32; 2],
    top_left_index: u32,
    width: u32,
    height: u32,
    grid_width: u32,
}

impl Rectangle {
    fn full_rectangle(top_left: [f32; 2], grid_width: u32, grid_height: u32) -> Rectangle {
        Rectangle {
            top_left: top_left,
            top_left_index: 0,
            width: grid_width,
            height: grid_height,
            grid_width: grid_width,
        }
    }

    fn relate_to_frustum(&self, frustum: &Frustum<f32>) -> Relation {
        let point_a = [self.top_left[0], self.top_left[1], 0.0];
        let point_b = [
            self.top_left[0] + (self.width - 1) as f32,
            self.top_left[1] - (self.height - 1) as f32,
            MAX_POSSIBLE_ELEVATION,
        ];

        let bounding_box = Aabb3::new(point_a.into(), point_b.into());
        frustum.contains(bounding_box)
    }

    fn indices(&self) -> [u32; 6] {
        [
            self.top_left_index,
            self.bottom_left_index(),
            self.top_right_index(),

            self.top_right_index(),
            self.bottom_left_index(),
            self.bottom_right_index(),
        ]
    }

    fn sub_rectangles(&self) -> Option<Vec<Rectangle>> {
        if !self.can_divide_vertically() && !self.can_divide_horizontally() {
            return None;
        }

        let left_width = self.width / 2 + 1;
        let right_width = self.width - left_width + 1;
        let middle_x = self.top_left[0] + left_width as f32 - 1.0;
        let top_middle = [middle_x, self.top_left[1]];

        if self.can_divide_vertically() && !self.can_divide_horizontally() {
            // divide into a left and right half
            let left = Rectangle {
                top_left: self.top_left,
                top_left_index: self.top_left_index,
                width: left_width,
                height: self.height,
                grid_width: self.grid_width,
            };

            let right = Rectangle {
                top_left: top_middle,
                top_left_index: self.top_middle_index(),
                width: right_width,
                height: self.height,
                grid_width: self.grid_width,
            };

            return Some(vec![left, right]);
        }

        let top_height = self.height / 2 + 1;
        let bottom_height = self.height - top_height + 1;
        let middle_y = self.top_left[1] - top_height as f32 + 1.0;
        let left_middle = [self.top_left[0], middle_y];

        if !self.can_divide_vertically() && self.can_divide_horizontally() {
            // divide into a top and bottom half
            let top = Rectangle {
                top_left: self.top_left,
                top_left_index: self.top_left_index,
                width: self.width,
                height: top_height,
                grid_width: self.grid_width,
            };

            let bottom = Rectangle {
                top_left: left_middle,
                top_left_index: self.left_middle_index(),
                width: self.width,
                height: bottom_height,
                grid_width: self.grid_width,
            };

            return Some(vec![top, bottom]);
        }

        // divide into four quadrants
        let center = [middle_x, middle_y];

        let top_left = Rectangle {
            top_left: self.top_left,
            top_left_index: self.top_left_index,
            width: left_width,
            height: top_height,
            grid_width: self.grid_width,
        };

        let top_right = Rectangle {
            top_left: top_middle,
            top_left_index: self.top_middle_index(),
            width: right_width,
            height: top_height,
            grid_width: self.grid_width,
        };

        let bottom_left = Rectangle {
            top_left: left_middle,
            top_left_index: self.left_middle_index(),
            width: left_width,
            height: bottom_height,
            grid_width: self.grid_width,
        };

        let bottom_right = Rectangle {
            top_left: center,
            top_left_index: self.center_index(),
            width: right_width,
            height: bottom_height,
            grid_width: self.grid_width,
        };

        Some(vec![top_left, top_right, bottom_left, bottom_right])
    }

    fn can_divide_vertically(&self) -> bool {
        self.width > 2
    }

    fn can_divide_horizontally(&self) -> bool {
        self.height > 2
    }

    fn top_right_index(&self) -> u32 {
        self.top_left_index + self.width - 1
    }

    fn bottom_left_index(&self) -> u32 {
        self.top_left_index + (self.height - 1) * self.grid_width
    }

    fn bottom_right_index(&self) -> u32 {
        self.bottom_left_index() + self.width - 1
    }

    fn top_middle_index(&self) -> u32 {
        self.top_left_index + self.width / 2
    }

    fn left_middle_index(&self) -> u32 {
        self.top_left_index + (self.height / 2) * self.grid_width
    }

    fn center_index(&self) -> u32 {
        self.left_middle_index() + self.width / 2
    }
}
