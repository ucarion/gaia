use std::thread;
use std::sync::mpsc;

use cgmath::{Matrix4, Vector2};
use gfx;
use lru_cache::LruCache;

mod terrain;
mod polygon;

use asset_getter::{TileAssets, TileAssetData};
use errors::*;
use self::polygon::PolygonRenderer;
use self::terrain::TerrainRenderer;
use tile::Tile;
use tile_chooser;
use tile_fetcher;

pub struct Renderer<R: gfx::Resources, F: gfx::Factory<R>> {
    asset_cache: LruCache<Tile, TileAssets<R>>,
    factory: F,
    polygon_renderer: PolygonRenderer<R, F>,
    terrain_renderer: TerrainRenderer<R, F>,
    texture_receiver: mpsc::Receiver<(Tile, Result<TileAssetData>)>,
    tile_sender: mpsc::Sender<Tile>,
}

impl<R: gfx::Resources, F: gfx::Factory<R> + Clone> Renderer<R, F> {
    pub fn new(factory: F) -> Result<Renderer<R, F>> {
        let (tile_sender, tile_receiver) = mpsc::channel();
        let (texture_sender, texture_receiver) = mpsc::channel();

        let polygon_renderer = PolygonRenderer::new(factory.clone())?;
        let terrain_renderer = TerrainRenderer::new(factory.clone())?;

        thread::Builder::new()
            .name("tile_fetcher".to_string())
            .spawn(move || {
                tile_fetcher::fetch_tiles(tile_receiver, texture_sender)
            })
            .chain_err(|| "Error creating texture loader thread")?;

        Ok(Renderer {
            asset_cache: LruCache::new(512),
            factory,
            polygon_renderer,
            terrain_renderer,
            texture_receiver,
            tile_sender,
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
        // Get tiles loaded in background thread, and put them in the cache
        for (tile, tile_texture_data) in self.texture_receiver.try_iter() {
            let assets = tile_texture_data?.create_assets(&mut self.factory)?;
            self.asset_cache.insert(tile, assets);
        }

        let mvp = mvp.into();

        let (level_of_detail, tiles_to_render, tiles_to_fetch) = tile_chooser::choose_tiles(
            &mut self.asset_cache,
            mvp.clone(),
            look_at.into(),
            camera_height,
        );

        for tile_to_fetch in tiles_to_fetch {
            self.tile_sender.send(tile_to_fetch).chain_err(
                || "Error sending tile to background thread",
            )?;
        }

        let mut polygon_metadatas = Vec::new();

        for (positioned_tile, indices) in tiles_to_render {
            let tile_assets = self.asset_cache.get_mut(&positioned_tile.tile).unwrap();

            self.terrain_renderer.render(
                encoder,
                target.clone(),
                stencil.clone(),
                &mvp,
                positioned_tile,
                indices,
                tile_assets,
            );

            polygon_metadatas.push(tile_assets.metadata.clone());
        }

        self.polygon_renderer.render(
            encoder,
            target,
            stencil,
            mvp,
            level_of_detail,
            &polygon_metadatas,
        );

        Ok(())
    }
}
