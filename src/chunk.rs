use crate::{
    CHUNK_SIZE, NEBULA_SCALE, seed::{SeedRng, chunk_seed, mix, value_noise},
};

#[derive(Clone, Copy, PartialEq)]
pub enum ObjectKind {
    Asteroid,
    Station,
    Dungeon,
}

#[derive(Clone, Copy)]
pub struct SpaceObject {
    pub id: u64,
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub kind: ObjectKind,
}

pub struct Chunk {
    pub coord: (i32, i32),
    pub objects: Vec<SpaceObject>,
}

pub fn generate_chunk(world_seed: u64, cx: i32, cy: i32) -> Chunk {
    let seed = chunk_seed(world_seed, cx, cy);
    let mut rng = SeedRng::new(seed);

    let origin_x = cx as f32 * CHUNK_SIZE;
    let origin_y = cy as f32 * CHUNK_SIZE;

    let density = value_noise(world_seed,
        (origin_x + CHUNK_SIZE / 2.0) / NEBULA_SCALE,
        (origin_y + CHUNK_SIZE / 2.0) / NEBULA_SCALE,
    );

    let base = rng.range_i32(1, 4);
    let bonus = (density * 8.0) as i32;
    let count = base + bonus;

    let mut objects = Vec::new();
    

    for i in 0..count {
        let local_x = rng.range_f32(0.0, CHUNK_SIZE);
        let local_y = rng.range_f32(0.0, CHUNK_SIZE);
        let roll = rng.next_f32();
        let kind = if roll < 0.70 {
            ObjectKind::Asteroid
        } else if roll < 0.92 {
            ObjectKind::Station
        } else {
            ObjectKind::Dungeon
        };

        let radius = match kind {
            ObjectKind::Asteroid => rng.range_f32(8.0, 26.0),
            ObjectKind::Station => 20.0,
            ObjectKind::Dungeon => 26.0,
        };
        objects.push(SpaceObject {
            id: mix(seed ^ (i as u64).wrapping_mul(0x9E3779B97F4A7C15)),
            x: origin_x + local_x,
            y: origin_y + local_y,
            radius,
            kind,
        });
    }

    Chunk {
        coord: (cx, cy),
        objects,
    }
}

