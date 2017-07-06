use image::{self, GenericImage};
use gfx;

pub fn create_world_textures_and_sampler<F, R>(
    factory: &mut F,
) -> (
    [gfx::handle::ShaderResourceView<R, [f32; 4]>; 2],
    gfx::handle::Sampler<R>,
)
where
    R: gfx::Resources,
    F: gfx::Factory<R>,
{
    let texture_view_west = create_world_texture(
        factory,
        [
            include_bytes!("../assets/generated/west_hemisphere-0.jpg"),
            include_bytes!("../assets/generated/west_hemisphere-1.jpg"),
            include_bytes!("../assets/generated/west_hemisphere-2.jpg"),
            include_bytes!("../assets/generated/west_hemisphere-3.jpg"),
        ],
    );

    let texture_view_east = create_world_texture(
        factory,
        [
            include_bytes!("../assets/generated/east_hemisphere-0.jpg"),
            include_bytes!("../assets/generated/east_hemisphere-1.jpg"),
            include_bytes!("../assets/generated/east_hemisphere-2.jpg"),
            include_bytes!("../assets/generated/east_hemisphere-3.jpg"),
        ],
    );

    let sampler = factory.create_sampler(gfx::texture::SamplerInfo::new(
        gfx::texture::FilterMethod::Bilinear,
        gfx::texture::WrapMode::Tile,
    ));

    ([texture_view_west, texture_view_east], sampler)
}

fn create_world_texture<F, R>(
    factory: &mut F,
    image_data: [&[u8]; 4],
) -> gfx::handle::ShaderResourceView<R, [f32; 4]>
where
    R: gfx::Resources,
    F: gfx::Factory<R>,
{
    let image0 = image::load_from_memory(image_data[0]).unwrap();
    let buffer0 = image0.to_rgba().into_raw();

    let (width, height) = image0.dimensions();
    let texture_kind =
        gfx::texture::Kind::D2(width as u16, height as u16, gfx::texture::AaMode::Single);

    let image1 = image::load_from_memory(image_data[1]).unwrap();
    let buffer1 = image1.to_rgba().into_raw();

    let image2 = image::load_from_memory(image_data[2]).unwrap();
    let buffer2 = image2.to_rgba().into_raw();

    let image3 = image::load_from_memory(image_data[3]).unwrap();
    let buffer3 = image3.to_rgba().into_raw();

    let texture_data = [
        buffer0.as_slice(),
        buffer1.as_slice(),
        buffer2.as_slice(),
        buffer3.as_slice(),
    ];

    let (_texture, texture_view) = factory
        .create_texture_immutable_u8::<gfx::format::Rgba8>(texture_kind, &texture_data)
        .unwrap();

    texture_view
}
