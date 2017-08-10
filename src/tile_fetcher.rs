use std::collections::BinaryHeap;
use std::sync::mpsc;
use std::time::Instant;

use errors::*;
use texture_getter::TileTextureData;
use tile::Tile;

#[derive(Eq, PartialEq, Ord, PartialOrd)]
struct FetchJob {
    created_at: Instant,
    tile: Tile,
}

impl FetchJob {
    fn new(tile: Tile) -> FetchJob {
        FetchJob {
            created_at: Instant::now(),
            tile: tile,
        }
    }
}

pub fn fetch_tiles(
    receive_tiles: mpsc::Receiver<Tile>,
    send_textures: mpsc::Sender<(Tile, Result<TileTextureData>)>,
) {
    let mut queue = BinaryHeap::new();

    loop {
        if queue.is_empty() {
            queue.push(FetchJob::new(receive_tiles.recv().unwrap()));
        }

        for tile in receive_tiles.try_iter() {
            queue.push(FetchJob::new(tile));
        }

        if let Some(job) = queue.pop() {
            let textures = TileTextureData::new(&job.tile);
            send_textures.send((job.tile, textures)).unwrap();
        }
    }
}
