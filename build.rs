extern crate pretty_env_logger;
extern crate gaia_assetgen;

use gaia_assetgen::PrepareAssetsTask;

fn main() {
    pretty_env_logger::init().unwrap();

    PrepareAssetsTask::new()
        .with_noaa_globe_dir("assets/noaa_globe".into())
        .with_nasa_blue_marble_dir("assets/nasa_blue_marble".into())
        .with_features_file("assets/ne_10m_admin_0_countries.geojson".into())
        .with_simplification_epsilons([0.01, 0.05, 0.10, 0.20, 0.40, 0.80, 1.50])
        .with_output_dir("assets/generated".into())
        .run()
        .unwrap();
}
