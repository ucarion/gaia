extern crate byteorder;

use std::io::{BufReader, BufWriter};
use std::fs::File;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

fn main() {
    let compression_factor = 5;
    let out_path = "assets/elevation.bin";

    // This information was taken from the NOAA GLOBE documentation manual:
    //
    // https://www.ngdc.noaa.gov/mgg/topo/report/globedocumentationmanual.pdf
    let columns_per_tile = 10800;
    let elevation_data_tiles = vec![
        // (num_rows, tile_names)
        (4800, vec!["a10g", "b10g", "c10g", "d10g"]),
        (6000, vec!["e10g", "f10g", "g10g", "h10g"]),
        (6000, vec!["i10g", "j10g", "k10g", "l10g"]),
        (4800, vec!["m10g", "n10g", "o10g", "p10g"]),
    ];

    let full_width = columns_per_tile * 4;
    let full_height: usize = elevation_data_tiles.iter().map(|&(num_rows, _)| num_rows).sum();

    let result_width = full_width / compression_factor;
    let result_height = full_height / compression_factor;

    let mut result = Vec::new();
    for _ in 0..result_height {
        result.push(vec![0; result_width]);
    }

    println!("Result will be {}x{}", result_width, result_height);

    let mut offset_y = 0;

    for (num_rows, tile_names) in elevation_data_tiles {
        let mut offset_x = 0;

        for tile_name in tile_names {
            println!("Processing {} ...", tile_name);
            let file = File::open(format!("assets/noaa_globe/{}", tile_name)).unwrap();
            let mut file = BufReader::new(file);

            let mut count = 0;
            while let Ok(elevation) = file.read_i16::<LittleEndian>() {
                let (x, y) = (count % columns_per_tile, count / columns_per_tile);

                if x % compression_factor == 0 && y % compression_factor == 0 {
                    let result_y = offset_y + y / compression_factor;
                    let result_x = offset_x + x / compression_factor;
                    result[result_y][result_x] = elevation;
                }

                count += 1;
            }

            offset_x += columns_per_tile / compression_factor;
        }

        offset_y += num_rows / compression_factor;
    }

    println!("Writing result to {} ...", out_path);

    let out_file = File::create(out_path).unwrap();
    let mut out_file = BufWriter::new(out_file);

    for y in 0..result_height {
        for x in 0..result_width {
            let elevation = result[y][x];

            out_file.write_i16::<LittleEndian>(elevation).unwrap();
        }
    }
}
