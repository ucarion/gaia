![Eastern Asia][east-asia]

> Eastern Asia. The Japanese islands are on the right, the Korean peninsula and
> mainland China on the left. At the bottom left is the island of Taiwan.

# Gaia, a planet visualizer in Rust

> Satellite imagery from [NASA Blue Marble][nasa]

> Elevation data from [NOAA GLOBE][noaa]

This is a program that lets you view the world. You can scroll up and down and
move the camera around. It's written entirely in Rust, using the
[Piston](piston) game engine.

## How to compile it

Gaia uses publicly-available data to present the world. First, you must download
the raw data from NASA and NOAA (these amount to about a gigabyte of data), and
then allow Imagemagick to convert these data into the format Gaia requires.

There's a script that does this for you. First, make sure you have Imagemagick's
`convert` available, and then run:

```
$ ./bin/generate_world_assets
```

From the directory this README is in.

**Warning:** This script will first take up 100% of you network bandwidth as it
downloads tremendous files, and then take up 100% of your CPU as it does image
processing on large images. Your computer might be brought to its knees.

Once that's done, you can run the usual Cargo command: (be sure to use the
release mode)

```
$ cargo run --release
```

After a few seconds of compilation and texture-loading, our beautiful planet
should appear before you.

## More Screenshots

![South Island][south-island]

> New Zealand's South Island, also known as *Te Waipounamu*.

![Pyrenees and Alps][pyrenees-and-alps]

> Southern France and northern Italy between the Pyrenees and the Alps. On the
> bottom right is the island of Corsica.

[east-asia]: screenshots/east-asia.jpg
[nasa]: https://visibleearth.nasa.gov/view.php?id=73909
[noaa]: https://www.ngdc.noaa.gov/mgg/topo/globe.html
[piston]: http://www.piston.rs/
[pyrenees-and-alps]: screenshots/pyrenees-and-alps.jpg
[south-island]: screenshots/south-island.jpg
