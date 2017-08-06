use std::io::BufReader;
use std::fs::File;

use byteorder::{ReadBytesExt, LittleEndian};
use gfx;
use image::{self, GenericImage};

use errors::*;

pub fn get_color_texture<R: gfx::Resources, F: gfx::Factory<R>>(
    factory: &mut F,
    level: u8,
    x: u8,
    y: u8,
) -> Result<gfx::handle::ShaderResourceView<R, [f32; 4]>> {
    let path = format!("assets/generated/tiles/{}_{}_{}.jpg", level, x, y);
    let texture_image = image::open(path)
        .chain_err(|| "Could not open tile image file")?;

    let (width, height) = texture_image.dimensions();
    let texture_kind =
        gfx::texture::Kind::D2(width as u16, height as u16, gfx::texture::AaMode::Single);

    let raw_data = texture_image.to_rgba().into_raw();

    let (_, texture_view) = factory
        .create_texture_immutable_u8::<gfx::format::Srgba8>(texture_kind, &[raw_data.as_slice()])
        .chain_err(|| "Could not create color texture")?;

    Ok(texture_view)
}

pub fn get_elevation_texture<R: gfx::Resources, F: gfx::Factory<R>>(
    factory: &mut F,
    level: u8,
    x: u8,
    y: u8,
) -> Result<gfx::handle::ShaderResourceView<R, u32>> {
    let path = format!("assets/generated/tiles/{}_{}_{}.elevation", level, x, y);
    let file = File::open(path)
        .chain_err(|| "Could not tile elevation file")?;
    let mut file = BufReader::new(file);

    let mut buf = Vec::new();
    while let Ok(data) = file.read_u16::<LittleEndian>() {
        buf.push(data_to_elevation(data));
    }

    let texture_kind = gfx::texture::Kind::D2(128, 128, gfx::texture::AaMode::Single);

    let (_, texture_view) = factory
        .create_texture_immutable::<(gfx::format::R16, gfx::format::Uint)>(
            texture_kind,
            &[buf.as_slice()],
        )
        .chain_err(|| "Could not create elevation texture")?;

    Ok(texture_view)
}

fn data_to_elevation(data: u16) -> u16 {
    if data <= 500 {
        0
    } else {
        data - 500
    }
}
