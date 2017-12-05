use cgmath::{Matrix4, Vector2};
use gfx;
use lru_cache::LruCache;

use errors::*;

mod terrain;
mod polygon;

use asset_getter::TileAssets;
use self::terrain::TerrainRenderer;
use self::polygon::PolygonRenderer;
use tile::Tile;
use tile_chooser;

pub struct Renderer<R: gfx::Resources, F: gfx::Factory<R>> {
    asset_cache: LruCache<Tile, TileAssets<R>>,
    terrain_renderer: TerrainRenderer<R, F>,
    polygon_renderer: PolygonRenderer<R, F>,
}

impl<R: gfx::Resources, F: gfx::Factory<R> + Clone> Renderer<R, F> {
    pub fn new(factory: F) -> Result<Renderer<R, F>> {
        Ok(Renderer {
            asset_cache: LruCache::new(512),
            terrain_renderer: TerrainRenderer::new(factory.clone())?,
            polygon_renderer: PolygonRenderer::new(factory.clone())?,
        })
    }

    pub fn render<
        C: gfx::CommandBuffer<R>,
        Matrix: Into<Matrix4<f32>>,
        Vector: Into<Vector2<f32>>,
    >(
        &mut self,
        encoder: &mut gfx::Encoder<R, C>,
        target: gfx::handle::RenderTargetView<R, gfx::format::Srgba8>,
        stencil: gfx::handle::DepthStencilView<R, gfx::format::DepthStencil>,
        mvp: Matrix,
        look_at: Vector,
        camera_height: f32,
    ) -> Result<()> {
        let (level_of_detail, tiles_to_render, tiles_to_fetch) =
            tile_chooser::choose_tiles(&mut self.asset_cache, mvp.into(), look_at.into(), camera_height);

        Ok(())
    }
}
