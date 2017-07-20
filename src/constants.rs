/// In the NOAA GLOBE data, sea level is marked as -500 meters. So that Imagemagick can work with
/// the data as a grayscale image, the data are offset by 500 so that no negative values appear
/// anywhere.
///
/// When reading elevation data from Imagemagick's output, the elevation should be converted back.
pub const ELEVATION_DATA_OFFSET: u16 = 500;

pub const ROW1_ELEVATION_GRID_HEIGHT: u32 = 1025;
pub const ROW2_ELEVATION_GRID_HEIGHT: u32 = 1024;

/// The width of the grid found in assets/generated/tiles/*/elevation.bin
pub const ELEVATION_GRID_WIDTH: u32 = 1025;
