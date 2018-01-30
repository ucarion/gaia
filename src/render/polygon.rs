use std::collections::BTreeMap;
use std::f32;
use std::fs::File;
use std::io::BufReader;

use cgmath::Matrix4;
use gaia_assetgen::{PolygonPointData, PolygonProperties, TileMetadata, MAX_LEVEL};
use gfx;
use gfx_draping;
use lru_cache::LruCache;
use serde_json;

use errors::*;

pub struct PolygonRenderer<R: gfx::Resources, F: gfx::Factory<R>> {
    factory: F,
    draping_renderer: gfx_draping::DrapingRenderer<R>,
    polygon_buffers: Vec<gfx_draping::RenderablePolygonBuffer<R>>,
    polygon_indices: BTreeMap<(u8, u64), gfx_draping::PolygonBufferIndices>,
    polygon_indices_cache: LruCache<(u8, Vec<u64>), gfx_draping::RenderablePolygonIndices<R>>,
    polygon_properties: BTreeMap<u64, PolygonProperties>,
}

impl<R: gfx::Resources, F: gfx::Factory<R>> PolygonRenderer<R, F> {
    pub fn new(mut factory: F) -> Result<PolygonRenderer<R, F>> {
        let polygon_point_data: PolygonPointData = serde_json::from_reader(BufReader::new(
            File::open("assets/generated/polygons.json").chain_err(
                || "Error opening polygons.json",
            )?,
        )).chain_err(|| "Error parsing polygons.json")?;

        let mut polygon_buffers = vec![gfx_draping::PolygonBuffer::new(); MAX_LEVEL as usize + 1];
        let mut polygon_indices = BTreeMap::new();
        let mut polygon_properties = BTreeMap::new();

        for (polygon_id, polygon) in polygon_point_data.polygons.into_iter().enumerate() {
            for (level, points) in polygon.levels.iter().enumerate() {
                let drapeable_polygon =
                    gfx_draping::Polygon::new(polygon.bounding_box, points.clone());
                let indices = polygon_buffers[level].add(&drapeable_polygon);

                polygon_indices.insert((level as u8, polygon_id as u64), indices);
            }

            polygon_properties.insert(polygon_id as u64, polygon.properties);
        }

        let polygon_buffers = polygon_buffers
            .into_iter()
            .map(|buf| buf.as_renderable(&mut factory))
            .collect();

        let draping_renderer = gfx_draping::DrapingRenderer::new(&mut factory);

        Ok(PolygonRenderer {
            factory,
            draping_renderer,
            polygon_buffers,
            polygon_indices,
            polygon_indices_cache: LruCache::new(256),
            polygon_properties,
        })
    }

    pub fn render<C: gfx::CommandBuffer<R>>(
        &mut self,
        encoder: &mut gfx::Encoder<R, C>,
        target: gfx::handle::RenderTargetView<R, gfx::format::Srgba8>,
        stencil: gfx::handle::DepthStencilView<R, gfx::format::DepthStencil>,
        mvp: Matrix4<f32>,
        level_of_detail: u8,
        positioned_polygons_to_render: &[(TileMetadata, i16)],
        polygon_color_chooser: &Fn(&PolygonProperties) -> [u8; 4],
    ) {
        // Multiple polygons can only be rendered simultaneously if they share the same color. So
        // we index polygons to render by their color using `polygon_batches`. The keys in
        // `polygon_batches` are pairs of (color, offset), where "offset" determines
        // where in the infinite scrolling map the polygons are situated.
        //
        // The key is not just the color because polygons need to be offset in world-space based on
        // their position in the infinite map. By separating polygons by their color *and* offset,
        // then a single matrix transformation can apply the offset.
        let mut polygon_batches = BTreeMap::new();

        // Build up `polygon_batches`.
        for &(ref metadata, offset) in positioned_polygons_to_render {
            for polygon_id in &metadata.polygons {

                let properties = &self.polygon_properties[polygon_id];
                let color = polygon_color_chooser(properties);
                let batch = polygon_batches.entry((color, offset)).or_insert((
                    Vec::new(),
                    f32::INFINITY,
                    f32::NEG_INFINITY,
                ));

                batch.0.push(*polygon_id);
                batch.1 = batch.1.min(metadata.min_elevation as f32);
                batch.2 = batch.2.max(metadata.max_elevation as f32);
            }
        }

        for ((color, offset), (polygon_ids, min_z, max_z)) in polygon_batches {
            let cache_key = (level_of_detail, polygon_ids);
            if !self.polygon_indices_cache.contains_key(&cache_key) {
                let mut indices = gfx_draping::PolygonBufferIndices::new();
                for polygon_id in &cache_key.1 {
                    indices.extend(&self.polygon_indices[&(level_of_detail, *polygon_id)]);
                }

                self.polygon_indices_cache.insert(
                    cache_key.clone(),
                    indices.as_renderable(
                        &mut self.factory,
                    ),
                );
            }

            let indices = self.polygon_indices_cache.get_mut(&cache_key).unwrap();
            let (min_z, max_z) = (elevation_to_z(min_z) - 0.01, elevation_to_z(max_z) + 0.01);
            let translate_x = 2.0 * offset as f32;

            let transform_polygon = Matrix4::from_translation([translate_x, 0.0, min_z].into()) *
                Matrix4::from_nonuniform_scale(2.0, 1.0, (max_z - min_z));

            let color = [
                color[0] as f32 / 255.0,
                color[1] as f32 / 255.0,
                color[2] as f32 / 255.0,
                color[3] as f32 / 255.0,
            ];

            self.draping_renderer.render(
                encoder,
                target.clone(),
                stencil.clone(),
                (mvp * transform_polygon).into(),
                color,
                &self.polygon_buffers[level_of_detail as usize],
                &indices,
            );
        }
    }
}

fn elevation_to_z(elevation: f32) -> f32 {
    let t = 1.0 - 1.0 / (1.0 + 0.0001 * elevation);
    return t * 0.03;
}
