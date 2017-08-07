use gfx;

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct Tile {
    pub level: u8,
    pub x: u8,
    pub y: u8,
}

pub struct TileTextures<R: gfx::Resources> {
    pub color: gfx::handle::ShaderResourceView<R, [f32; 4]>,
    pub elevation: gfx::handle::ShaderResourceView<R, u32>,
}
