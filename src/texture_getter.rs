use gfx;
use image::{self, GenericImage};

pub fn get_texture<R: gfx::Resources, F: gfx::Factory<R>>(
    factory: &mut F,
    level: u8,
    x: u8,
    y: u8,
) -> gfx::handle::ShaderResourceView<R, [f32; 4]> {
    let path = format!("assets/generated/tiles/{}_{}_{}.jpg", level, x, y);
    let texture_image = image::open(path).unwrap();

    let (width, height) = texture_image.dimensions();
    let texture_kind =
        gfx::texture::Kind::D2(width as u16, height as u16, gfx::texture::AaMode::Single);

    let raw_data = texture_image.to_rgba().into_raw();

    let (_, texture_view) = factory
        .create_texture_immutable_u8::<gfx::format::Srgba8>(texture_kind, &[raw_data.as_slice()])
        .unwrap();

    texture_view
}
