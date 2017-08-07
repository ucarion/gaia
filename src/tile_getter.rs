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

    vec![
        PositionedTile::enclosing_point(desired_level, camera_position[0], camera_position[1]),
    ]
}
