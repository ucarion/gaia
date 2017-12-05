use gfx;
use gfx_draping;
use lru_cache::LruCache;

use errors::*;

pub struct PolygonRenderer<R: gfx::Resources, F: gfx::Factory<R>> {
    factory: F,
    polygon_indices_cache: LruCache<(u8, Vec<u64>), gfx_draping::RenderablePolygonIndices<R>>,
}

impl<R: gfx::Resources, F: gfx::Factory<R>> PolygonRenderer<R, F> {
    pub fn new(factory: F) -> Result<PolygonRenderer<R, F>> {
        Ok(PolygonRenderer {
            factory: factory,
            polygon_indices_cache: LruCache::new(256),
        })
    }
}
