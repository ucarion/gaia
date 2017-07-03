use collision::{Aabb3, Frustum, Relation};

use constants::VERTEX_GRID_SIDE_LENGTH;

/// The greatest possible `z` value any vertex may have.
///
/// TODO: Make this dynamically calculated.
const MAX_POSSIBLE_ELEVATION: f32 = 300.0;

pub fn get_indices(mvp_matrix: [[f32; 4]; 4], camera_pos: [f32; 3], top_left: [f32; 2]) -> Vec<u32> {
    let frustum = Frustum::from_matrix4(mvp_matrix.into()).unwrap();

    let max_depth = match camera_pos[2] {
        0.0 ... 300.0 => { 9 },
        300.0 ... 500.0 => { 8 },
        500.0 ... 650.0 => { 7 },
        650.0 ... 1200.0 => { 6 },
        1200.0 ... 2000.0 => { 5 },
        _ => { 4 },
    };

    let mut result = Vec::new();
    add_indices(&mut result, frustum, true, max_depth, top_left, 0, VERTEX_GRID_SIDE_LENGTH, 0);
    result
}

fn add_indices(buf: &mut Vec<u32>, frustum: Frustum<f32>, try_culling: bool, max_depth: usize,
               top_left: [f32; 2], top_left_index: u32, side_length: u32, current_depth: usize) {
    // There are three possible relations between the bounding box surrounding this square and the
    // view frustum:
    //
    // * It is entirely outside the frustum: The square cannot be seen, and no indices from it or
    // any subsquare are necessary.
    //
    // * It crosses the frustum: The square can partially be seen. Indices from it are necessary,
    // but some subsquares might be entirely outside the frustum.
    //
    // * It is entirely within the frustum: The square can entirely be seen. Indices from it are
    // necessary, and there's no chance of any subsquare crossing or falling beyond the frustum.
    let (cull_square, try_culling_subsquares) = if try_culling {
        let relation = relate_square_to_frustum(frustum, top_left, side_length);

        (relation == Relation::Out, relation == Relation::Cross)
    } else {
        (false, false)
    };

    if cull_square {
        return;
    }

    if current_depth == max_depth {
        // this square is good enough, so return two triangles formed from this square's corners
        let top_right_index = top_left_index + side_length - 1;
        let bottom_left_index = top_left_index + (side_length - 1) * VERTEX_GRID_SIDE_LENGTH;
        let bottom_right_index = bottom_left_index + side_length - 1;

        buf.extend_from_slice(&[top_left_index, bottom_left_index, top_right_index]);
        buf.extend_from_slice(&[top_right_index, bottom_left_index, bottom_right_index]);
    } else {
        // this square is not good enough, so split it into four quadrants and recursively get
        // indices for those squares
        let next_side_length = (side_length / 2) + 1;
        let middle_x = top_left[0] + next_side_length as f32 - 1.0;
        let middle_y = top_left[1] - next_side_length as f32 + 1.0;

        let top_right_index = top_left_index + next_side_length - 1;
        let top_right = [middle_x, top_left[1]];

        let bottom_left_index = top_left_index + (next_side_length - 1) * VERTEX_GRID_SIDE_LENGTH;
        let bottom_left = [top_left[0], middle_y];

        let bottom_right_index = bottom_left_index + next_side_length - 1;
        let bottom_right = [middle_x, middle_y];

        add_indices(buf, frustum, try_culling_subsquares, max_depth,
                    top_left, top_left_index, next_side_length, current_depth + 1);

        add_indices(buf, frustum, try_culling_subsquares, max_depth,
                    top_right, top_right_index, next_side_length, current_depth + 1);

        add_indices(buf, frustum, try_culling_subsquares, max_depth,
                    bottom_left, bottom_left_index, next_side_length, current_depth + 1);

        add_indices(buf, frustum, try_culling_subsquares, max_depth,
                    bottom_right, bottom_right_index, next_side_length, current_depth + 1);
    }
}

fn relate_square_to_frustum(frustum: Frustum<f32>, top_left: [f32; 2], side_length: u32) -> Relation {
    let side_length = side_length as f32;
    let point_a = [top_left[0], top_left[1], 0.0];
    let point_b = [top_left[0] + side_length, top_left[1] - side_length, MAX_POSSIBLE_ELEVATION];

    let bounding_box = Aabb3::new(point_a.into(), point_b.into());
    frustum.contains(bounding_box)
}
