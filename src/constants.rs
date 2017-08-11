use tile::Tile;

/// The world is infinitely long along the east-west direction (x-axis). It will appear along the
/// y-axis between 0 and `-WORLD_HEIGHT`.
pub const WORLD_HEIGHT: f32 = 1000.0;

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

/// The highest-detail tiles are of level zero. This is the least-detail, widest-covering level.
/// The two tiles of this level cover a hemisphere each.
pub const MAX_TILE_LEVEL: u8 = 6;

/// `z`-values of vertices cannot be greater than this value. This is used for view frustum
/// culling.
pub const Z_UPPER_BOUND: f32 = 30.0;

lazy_static! {
    pub static ref LEVEL0_TILE_WIDTH: f32 = {
        WORLD_HEIGHT / Tile::num_tiles_across_level_height(0) as f32
    };
}
