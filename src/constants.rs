/// In the NOAA GLOBE data, sea level is marked as -500 meters. So that Imagemagick can work with
/// the data as a grayscale image, the data are offset by 500 so that no negative values appear
/// anywhere.
///
/// When reading elevation data from Imagemagick's output, the elevation should be converted back.
pub const ELEVATION_DATA_OFFSET: u16 = 500;

/// The width of each tile on level zero.
pub const LEVEL0_TILE_WIDTH: f32 = 10.0;
