pub struct TileRenderInfo {
    pub indices: Vec<u32>,
    pub x_offset: f32,
    pub kind: TileKind,
}

#[derive(Eq, PartialEq, Hash)]
pub enum TileKind {
    WestHemisphere,
    EastHemisphere,
}
