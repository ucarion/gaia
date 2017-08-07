use tile::PositionedTile;

pub fn desired_tiles(camera_position: [f32; 3]) -> Vec<PositionedTile> {
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
