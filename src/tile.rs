use constants::{ROW1_ELEVATION_GRID_HEIGHT, ROW2_ELEVATION_GRID_HEIGHT};

#[derive(Debug)]
pub struct TileRenderInfo {
    pub indices: Vec<u32>,
    pub offset: [f32; 2],
    pub kind: TileKind,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum TileKind {
    A1,
    A2,
    B1,
    B2,
    C1,
    C2,
    D1,
    D2,
}

pub const TILE_KINDS: [TileKind; 8] = [
    TileKind::A1,
    TileKind::A2,
    TileKind::B1,
    TileKind::B2,
    TileKind::C1,
    TileKind::C2,
    TileKind::D1,
    TileKind::D2,
];


impl TileKind {
    pub fn elevation_grid_height(&self) -> u32 {
        if self.is_row1() {
            ROW1_ELEVATION_GRID_HEIGHT
        } else {
            ROW2_ELEVATION_GRID_HEIGHT
        }
    }

    fn is_row1(&self) -> bool {
        match *self {
            TileKind::A1 | TileKind::B1 | TileKind::C1 | TileKind::D1 => true,
            _ => false,
        }
    }

    pub fn elevation_data(&self) -> &'static [u8] {
        match *self {
            TileKind::A1 => include_bytes!("../assets/generated/tiles/A1/elevation.bin"),
            TileKind::A2 => include_bytes!("../assets/generated/tiles/A2/elevation.bin"),
            TileKind::B1 => include_bytes!("../assets/generated/tiles/B1/elevation.bin"),
            TileKind::B2 => include_bytes!("../assets/generated/tiles/B2/elevation.bin"),
            TileKind::C1 => include_bytes!("../assets/generated/tiles/C1/elevation.bin"),
            TileKind::C2 => include_bytes!("../assets/generated/tiles/C2/elevation.bin"),
            TileKind::D1 => include_bytes!("../assets/generated/tiles/D1/elevation.bin"),
            TileKind::D2 => include_bytes!("../assets/generated/tiles/D2/elevation.bin"),
        }
    }

    pub fn texture_data(&self) -> [&'static [u8]; 4] {
        match *self {
            TileKind::A1 => {
                [
                    include_bytes!("../assets/generated/tiles/A1/0.jpg"),
                    include_bytes!("../assets/generated/tiles/A1/1.jpg"),
                    include_bytes!("../assets/generated/tiles/A1/2.jpg"),
                    include_bytes!("../assets/generated/tiles/A1/3.jpg"),
                ]
            }
            TileKind::A2 => {
                [
                    include_bytes!("../assets/generated/tiles/A2/0.jpg"),
                    include_bytes!("../assets/generated/tiles/A2/1.jpg"),
                    include_bytes!("../assets/generated/tiles/A2/2.jpg"),
                    include_bytes!("../assets/generated/tiles/A2/3.jpg"),
                ]
            }
            TileKind::B1 => {
                [
                    include_bytes!("../assets/generated/tiles/B1/0.jpg"),
                    include_bytes!("../assets/generated/tiles/B1/1.jpg"),
                    include_bytes!("../assets/generated/tiles/B1/2.jpg"),
                    include_bytes!("../assets/generated/tiles/B1/3.jpg"),
                ]
            }
            TileKind::B2 => {
                [
                    include_bytes!("../assets/generated/tiles/B2/0.jpg"),
                    include_bytes!("../assets/generated/tiles/B2/1.jpg"),
                    include_bytes!("../assets/generated/tiles/B2/2.jpg"),
                    include_bytes!("../assets/generated/tiles/B2/3.jpg"),
                ]
            }
            TileKind::C1 => {
                [
                    include_bytes!("../assets/generated/tiles/C1/0.jpg"),
                    include_bytes!("../assets/generated/tiles/C1/1.jpg"),
                    include_bytes!("../assets/generated/tiles/C1/2.jpg"),
                    include_bytes!("../assets/generated/tiles/C1/3.jpg"),
                ]
            }
            TileKind::C2 => {
                [
                    include_bytes!("../assets/generated/tiles/C2/0.jpg"),
                    include_bytes!("../assets/generated/tiles/C2/1.jpg"),
                    include_bytes!("../assets/generated/tiles/C2/2.jpg"),
                    include_bytes!("../assets/generated/tiles/C2/3.jpg"),
                ]
            }
            TileKind::D1 => {
                [
                    include_bytes!("../assets/generated/tiles/D1/0.jpg"),
                    include_bytes!("../assets/generated/tiles/D1/1.jpg"),
                    include_bytes!("../assets/generated/tiles/D1/2.jpg"),
                    include_bytes!("../assets/generated/tiles/D1/3.jpg"),
                ]
            }
            TileKind::D2 => {
                [
                    include_bytes!("../assets/generated/tiles/D2/0.jpg"),
                    include_bytes!("../assets/generated/tiles/D2/1.jpg"),
                    include_bytes!("../assets/generated/tiles/D2/2.jpg"),
                    include_bytes!("../assets/generated/tiles/D2/3.jpg"),
                ]
            }
        }
    }
}
