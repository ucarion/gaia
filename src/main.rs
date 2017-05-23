// extern crate byteorder;
// extern crate image;

// use std::io::{BufReader, Read};
// use std::fs::File;

// use byteorder::{LittleEndian, ReadBytesExt};

// fn main() {
//     let file = File::open("assets/noaa_globe/h10g").unwrap();
//     let mut file = BufReader::new(file);
//     let mut img_buf = image::ImageBuffer::new(10800, 6000);

//     let mut count = 0;

//     while let Ok(byte) = file.read_i16::<LittleEndian>() {
//         let (x, y) = (count % 10800, count / 10800);
//         let color = if byte > 0 { 0 } else { 255 };

//         img_buf.put_pixel(x, y, image::Rgb([color; 3]));

//         count += 1;
//     }

//     let mut img_out = File::create("out/out.png").unwrap();
//     image::ImageRgb8(img_buf).save(&mut img_out, image::PNG);

//     println!("Total count: {}", count);
// }

extern crate piston_window;
extern crate sdl2_window;

use piston_window::{OpenGL, PistonWindow, WindowSettings};
use sdl2_window::Sdl2Window;

fn main() {
    println!("Hello, world!");

    let mut window: PistonWindow<Sdl2Window> =
        WindowSettings::new("piston: cube", [640, 480])
        .exit_on_esc(true)
        .samples(4)
        .opengl(OpenGL::V3_2)
        .build()
        .unwrap();
}
