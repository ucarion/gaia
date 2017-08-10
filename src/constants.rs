/// In the NOAA GLOBE data, sea level is marked as -500 meters. So that Imagemagick can work with
/// the data as a grayscale image, the data are offset by 500 so that no negative values appear
/// anywhere.
///
/// When reading elevation data from Imagemagick's output, the elevation should be converted back.
pub const ELEVATION_DATA_OFFSET: u16 = 500;

/// The width of a tile's color texture.
pub const COLOR_TILE_WIDTH: u16 = 256;

/// The width of a tile's elevation heightmap.
///
/// This must be `2^N + 1` for some integer `N >= 6`, so that the tiles can be sub-divided with
/// overlapping edges, and that a level 6 tile can be divided 6 times.
pub const ELEVATION_TILE_WIDTH: u16 = 65;

/// The displayed width of each tile on level zero.
pub const LEVEL0_TILE_WIDTH: f32 = 10.0;

/// The highest-detail tiles are of level zero. This is the least-detail, widest-covering level.
/// The two tiles of this level cover a hemisphere each.
pub const MAX_TILE_LEVEL: u8 = 6;
