#[derive(Debug)]
pub struct TileRenderInfo {
    pub indices: Vec<u32>,
    pub x_offset: f32,
    pub kind: TileKind,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum TileKind {
    WestHemisphere,
    EastHemisphere,
    Meridian0,
    Meridian180,
}
