pub struct TileRenderInfo {
    pub indices: Vec<u32>,
    pub x_offset: f32,
    pub kind: TileKind,
}

pub enum TileKind {
    WestHemisphere,
    EastHemisphere,
}
