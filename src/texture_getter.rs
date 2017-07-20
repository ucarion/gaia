use std::collections::HashMap;

use tile::{TileKind, TILE_KINDS};

use image::{self, GenericImage};
use gfx;

pub fn create_world_textures_and_sampler<F, R>(
    factory: &mut F,
) -> (
    HashMap<TileKind, gfx::handle::ShaderResourceView<R, [f32; 4]>>,
    gfx::handle::Sampler<R>,
)
where
    R: gfx::Resources,
    F: gfx::Factory<R>,
{
    let mut textures_by_kind = HashMap::new();

    for tile_kind in TILE_KINDS.iter() {
        textures_by_kind.insert(
            tile_kind.clone(),
            create_world_texture(factory, &tile_kind.texture_data()),
        );
    }

    let sampler = factory.create_sampler(gfx::texture::SamplerInfo::new(
        gfx::texture::FilterMethod::Bilinear,
        gfx::texture::WrapMode::Tile,
    ));


    (textures_by_kind, sampler)
}

fn create_world_texture<F, R>(
    factory: &mut F,
    texture_data: &[&[u8]],
) -> gfx::handle::ShaderResourceView<R, [f32; 4]>
where
    R: gfx::Resources,
    F: gfx::Factory<R>,
{
    let images: Vec<_> = texture_data
        .iter()
        .map(|data| image::load_from_memory(data).unwrap())
        .collect();

    let (width, height) = images[0].dimensions();
    let texture_kind =
        gfx::texture::Kind::D2(width as u16, height as u16, gfx::texture::AaMode::Single);

    let raw_image_data: Vec<_> = images
        .iter()
        .map(|image| image.to_rgba().into_raw())
        .collect();

    let raw_image_slices: Vec<_> = raw_image_data.iter().map(|raw| raw.as_slice()).collect();

    let (_texture, texture_view) = factory
        .create_texture_immutable_u8::<gfx::format::Rgba8>(texture_kind, &raw_image_slices)
        .unwrap();

    texture_view
}
