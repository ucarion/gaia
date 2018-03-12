extern crate gaia_assetgen;

use gaia_assetgen::PrepareAssetsTask;

fn main() {
    PrepareAssetsTask::new()
        .with_noaa_globe_dir("assets/noaa_globe".into())
        .with_nasa_blue_marble_dir("assets/nasa_blue_marble".into())
        .with_polygons_file("assets/ne_10m_admin_0_countries_lakes.geojson".into())
        .with_points_file("assets/cities.geojson".into())
        .with_simplification_epsilons([1.50, 0.80, 0.40, 0.20, 0.10, 0.05, 0.01])
        .with_output_dir("assets/generated".into())
        .run()
        .unwrap();
}
