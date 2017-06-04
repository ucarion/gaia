use collision::{Aabb3, Frustum, Relation};

/// The greatest ratio of area to distance-from-camera that a square can have without being
/// subdivided.
const LEVEL_OF_DETAIL_THRESHOLD: f32 = 0.3;

/// The width of the vertex grid. Used to calculate indices into the vertex grid.
const VERTEX_GRID_SIDE_LENGTH: u32 = 4097;

/// The greatest possible `z` value any vertex may have.
///
/// TODO: Make this dynamically calculated.
const MAX_POSSIBLE_ELEVATION: f32 = 300.0;

/// If the camera height is above this value, do not attempt to use frustum culling.
const MAX_HEIGHT_FOR_CULLING: f32 = 1000.0;

pub fn get_indices(mvp_matrix: [[f32; 4]; 4], camera_pos: [f32; 3], top_left: [f32; 2]) -> Vec<u32> {
    let frustum = Frustum::from_matrix4(mvp_matrix.into()).unwrap();
    let try_culling = camera_pos[2] < MAX_HEIGHT_FOR_CULLING;

    let mut result = Vec::new();
    add_indices(&mut result, frustum, try_culling, camera_pos, top_left, 0, VERTEX_GRID_SIDE_LENGTH);
    result
}

fn add_indices(buf: &mut Vec<u32>, frustum: Frustum<f32>, try_culling: bool, camera_pos: [f32; 3],
               top_left: [f32; 2], top_left_index: u32, side_length: u32) {
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

    let area = side_length.pow(2) as f32;
    let distance = euclidean_distance(camera_pos, center_point(top_left, side_length));

    if area / distance < LEVEL_OF_DETAIL_THRESHOLD {
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

        add_indices(buf, frustum, try_culling_subsquares, camera_pos,
                    top_left, top_left_index, next_side_length);

        add_indices(buf, frustum, try_culling_subsquares, camera_pos,
                    top_right, top_right_index, next_side_length);

        add_indices(buf, frustum, try_culling_subsquares, camera_pos,
                    bottom_left, bottom_left_index, next_side_length);

        add_indices(buf, frustum, try_culling_subsquares, camera_pos,
                    bottom_right, bottom_right_index, next_side_length);
    }
}

fn relate_square_to_frustum(frustum: Frustum<f32>, top_left: [f32; 2], side_length: u32) -> Relation {
    let side_length = side_length as f32;
    let point_a = [top_left[0], top_left[1], 0.0];
    let point_b = [top_left[0] + side_length, top_left[1] - side_length, MAX_POSSIBLE_ELEVATION];

    let bounding_box = Aabb3::new(point_a.into(), point_b.into());
    frustum.contains(bounding_box)
}

fn center_point(top_left: [f32; 2], side_length: u32) -> [f32; 3] {
    let half_side_length = side_length as f32 / 2.0;
    [top_left[0] + half_side_length, top_left[0] - half_side_length, 0.0]
}

fn euclidean_distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).powf(0.5)
}
