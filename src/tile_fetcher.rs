use std::sync::mpsc;

use errors::*;
use texture_getter::TileTextureData;
use tile::Tile;

pub fn fetch_tiles(
    receive_tiles: mpsc::Receiver<Tile>,
    send_textures: mpsc::Sender<(Tile, Result<TileTextureData>)>,
) {
    let mut jobs = Vec::new();

    loop {
        if jobs.is_empty() {
            let tile = receive_tiles.recv().unwrap();
            jobs.push(tile);
        }

        for tile in receive_tiles.try_iter() {
            if !jobs.contains(&tile) {
                jobs.push(tile);
            }
        }

        let tile = jobs.pop().unwrap();
        let textures = TileTextureData::new(&tile);
        send_textures.send((tile, textures)).unwrap();
    }
}
