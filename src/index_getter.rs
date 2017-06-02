const LEVEL_OF_DETAIL_THRESHOLD: f32 = 2.0;
const VERTEX_GRID_SIDE_LENGTH: u32 = 4097;

pub fn get_indices(camera_pos: [f32; 3], top_left: [f32; 2]) -> Vec<u32> {
    let mut result = Vec::new();
    add_indices(&mut result, camera_pos, top_left, 0, VERTEX_GRID_SIDE_LENGTH);
    result
}

fn add_indices(buf: &mut Vec<u32>, camera_pos: [f32; 3],
               top_left: [f32; 2], top_left_index: u32, side_length: u32) {
    let area = side_length.pow(2) as f32;

    let center_point = center_point(top_left, side_length);
    let distance = euclidean_distance(camera_pos, center_point);

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
        let top_right = [middle_x, top_left[0]];

        let bottom_left_index = top_left_index + (next_side_length - 1) * VERTEX_GRID_SIDE_LENGTH;
        let bottom_left = [top_left[0], middle_y];

        let bottom_right_index = bottom_left_index + next_side_length - 1;
        let bottom_right = [middle_x, middle_y];

        add_indices(buf, camera_pos, top_left, top_left_index, next_side_length);
        add_indices(buf, camera_pos, top_right, top_right_index, next_side_length);
        add_indices(buf, camera_pos, bottom_left, bottom_left_index, next_side_length);
        add_indices(buf, camera_pos, bottom_right, bottom_right_index, next_side_length);
    }
}

fn center_point(top_left: [f32; 2], side_length: u32) -> [f32; 3] {
    let half_side_length = side_length as f32 / 2.0;
    [top_left[0] + half_side_length, top_left[0] - half_side_length, 0.0]
}

fn euclidean_distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).powf(0.5)
}
