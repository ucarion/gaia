use std::io::BufReader;
use std::fs::File;

use byteorder::{ReadBytesExt, LittleEndian};
use gfx;
use image::{self, GenericImage};

pub fn get_color_texture<R: gfx::Resources, F: gfx::Factory<R>>(
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

pub fn get_elevation_texture<R: gfx::Resources, F: gfx::Factory<R>>(
    factory: &mut F,
    level: u8,
    x: u8,
    y: u8,
) -> gfx::handle::ShaderResourceView<R, u32> {
    let path = format!("assets/generated/tiles/{}_{}_{}.elevation", level, x, y);
    let mut file = BufReader::new(File::open(path).unwrap());

    let mut buf = Vec::new();
    while let Ok(data) = file.read_u16::<LittleEndian>() {
        buf.push(data);
    }

    let texture_kind = gfx::texture::Kind::D2(128, 128, gfx::texture::AaMode::Single);

    let (_, texture_view) = factory
        .create_texture_immutable::<(gfx::format::R16, gfx::format::Uint)>(
            texture_kind,
            &[buf.as_slice()],
        )
        .unwrap();

    texture_view
}
