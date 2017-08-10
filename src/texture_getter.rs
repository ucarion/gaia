use std::io::BufReader;
use std::fs::File;

use byteorder::{ReadBytesExt, LittleEndian};
use gfx;
use image;

use constants::{COLOR_TILE_WIDTH, ELEVATION_DATA_OFFSET, ELEVATION_TILE_WIDTH};
use errors::*;
use tile::Tile;

pub struct TileTextures<R: gfx::Resources> {
    pub color: gfx::handle::ShaderResourceView<R, [f32; 4]>,
    pub elevation: gfx::handle::ShaderResourceView<R, u32>,
}

pub struct TileTextureData {
    pub color: Vec<u8>,
    pub elevation: Vec<u16>,
}

impl TileTextureData {
    pub fn new(tile: &Tile) -> Result<TileTextureData> {
        Ok(TileTextureData {
            color: get_color_data(tile)?,
            elevation: get_elevation_data(tile)?,
        })
    }

    pub fn create_textures<R: gfx::Resources, F: gfx::Factory<R>>(
        &self,
        factory: &mut F,
    ) -> Result<TileTextures<R>> {
        let color_texture_kind = gfx::texture::Kind::D2(
            COLOR_TILE_WIDTH,
            COLOR_TILE_WIDTH,
            gfx::texture::AaMode::Single,
        );
        let (_, color_texture_view) = factory
            .create_texture_immutable_u8::<gfx::format::Srgba8>(
                color_texture_kind,
                &[self.color.as_slice()],
            )
            .chain_err(|| "Could not create color texture")?;

        let elevation_texture_kind = gfx::texture::Kind::D2(
            ELEVATION_TILE_WIDTH,
            ELEVATION_TILE_WIDTH,
            gfx::texture::AaMode::Single,
        );
        let (_, elevation_texture_view) = factory
            .create_texture_immutable::<(gfx::format::R16, gfx::format::Uint)>(
                elevation_texture_kind,
                &[self.elevation.as_slice()],
            )
            .chain_err(|| "Could not create color texture")?;

        Ok(TileTextures {
            color: color_texture_view,
            elevation: elevation_texture_view,
        })
    }
}

fn get_color_data(tile: &Tile) -> Result<Vec<u8>> {
    let path = format!(
        "assets/generated/tiles/{}_{}_{}.jpg",
        tile.level,
        tile.x,
        tile.y
    );

    let img = image::open(path)
        .chain_err(|| "Error while opening tile image")?;
    Ok(img.to_rgba().into_raw())
}

fn get_elevation_data(tile: &Tile) -> Result<Vec<u16>> {
    let path = format!(
        "assets/generated/tiles/{}_{}_{}.elevation",
        tile.level,
        tile.x,
        tile.y
    );

    let mut file = BufReader::new(File::open(path)
        .chain_err(|| "Could not tile elevation file")?);

    let mut buf = Vec::new();
    while let Ok(data_point) = file.read_u16::<LittleEndian>() {
        let elevation = data_point.saturating_sub(ELEVATION_DATA_OFFSET);
        buf.push(elevation);
    }

    Ok(buf)
}
