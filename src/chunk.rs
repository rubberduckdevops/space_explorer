use crate::player::Player;
use macroquad::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{Receiver, Sender, channel};

// World / chunk tuning
pub const CHUNK_SIZE: f32 = 512.0;
pub const NEBULA_SCALE: f32 = 1400.0;
pub const LOAD_RADIUS: i32 = 10;
pub const CHUNK_DEBUG: bool = false;
pub const INTERACT_RANGE: f32 = 70.0;
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

    let density = value_noise(
        world_seed,
        (origin_x + CHUNK_SIZE / 2.0) / NEBULA_SCALE,
        (origin_y + CHUNK_SIZE / 2.0) / NEBULA_SCALE,
    );

    // Space is vast and mostly empty. Objects only appear where the nebula
    // density is high enough, and even there they stay sparse.
    let count = if density < 0.6 {
        0
    } else {
        rng.range_i32(0, 3) + ((density - 0.6) * 5.0) as i32
    };

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

pub fn mix(mut x: u64) -> u64 {
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D049BB133111EB);
    x ^ (x >> 31)
}

/// A minimal deterministic PRNG (splitmix64).
pub struct SeedRng {
    pub state: u64,
}

impl SeedRng {
    pub fn new(seed: u64) -> SeedRng {
        SeedRng { state: seed }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E3779B97F4A7C15);
        mix(self.state)
    }

    /// A float in [0.0, 1.0).
    pub fn next_f32(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }

    /// A float in [lo, hi).
    pub fn range_f32(&mut self, lo: f32, hi: f32) -> f32 {
        lo + (hi - lo) * self.next_f32()
    }

    /// An integer in [lo, hi).
    pub fn range_i32(&mut self, lo: i32, hi: i32) -> i32 {
        lo + (self.next_u64() % (hi - lo) as u64) as i32
    }
}

pub fn hash_seed_string(s: &str) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325; // FNV offset basis
    for byte in s.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3); // FNV prime
    }
    hash
}

pub fn chunk_seed(world_seed: u64, cx: i32, cy: i32) -> u64 {
    let a = world_seed;
    let b = (cx as u64).wrapping_mul(0x9E3779B97F4A7C15);
    let c = (cy as u64).wrapping_mul(0xC2B2AE3D27D4EB4F);
    mix(a ^ b ^ c)
}

fn spawn_generator(seed: u64) -> (Sender<(i32, i32)>, Receiver<Chunk>) {
    let (req_tx, req_rx) = channel::<(i32, i32)>();
    let (res_tx, res_rx) = channel::<Chunk>();
    std::thread::spawn(move || {
        while let Ok((cx, cy)) = req_rx.recv() {
            log::debug!("Generating Chunk: {},{}", &cx, &cy);
            let chunk = generate_chunk(seed, cx, cy);

            if res_tx.send(chunk).is_err() {
                break; // main side hung up
            }
        }
    });

    (req_tx, res_rx)
}
pub struct World {
    #[allow(dead_code)]
    pub seed: u64,
    pub loaded: HashMap<(i32, i32), Chunk>,
    pub pending: HashSet<(i32, i32)>,
    pub req_tx: Sender<(i32, i32)>,
    pub res_rx: Receiver<Chunk>,
}

impl World {
    pub fn new(seed: u64) -> World {
        log::info!("Generating World");
        let (req_tx, res_rx) = spawn_generator(seed);

        World {
            seed,
            loaded: HashMap::new(),
            pending: HashSet::new(),
            req_tx,
            res_rx,
        }
    }

    #[allow(dead_code)]
    pub fn ensure_chunk(&mut self, cx: i32, cy: i32) {
        let seed = self.seed;
        self.loaded
            .entry((cx, cy))
            .or_insert_with(|| generate_chunk(seed, cx, cy));
    }
    pub fn stream_around(&mut self, center: (i32, i32), radius: i32) {
        // 1. Request any chunk in range that's neither loaded nor already pending
        for cy in (center.1 - radius)..=(center.1 + radius) {
            for cx in (center.0 - radius)..=(center.0 + radius) {
                let key = (cx, cy);
                if !self.loaded.contains_key(&key) && !self.pending.contains(&key) {
                    self.pending.insert(key);
                    let _ = self.req_tx.send(key);
                }
            }
        }
        // 2. Collect whatever the worker has finished (non-blocking)
        while let Ok(chunk) = self.res_rx.try_recv() {
            self.pending.remove(&chunk.coord);
            self.loaded.insert(chunk.coord, chunk);
        }
        // 3. Unload distant chunks ( and forget distant pending requests)
        let keep = radius + 1;
        self.loaded
            .retain(|&(cx, cy), _| (cx - center.0).abs() <= keep && (cy - center.1).abs() <= keep);
        self.pending
            .retain(|&(cx, cy)| (cx - center.0).abs() <= keep && (cy - center.1).abs() <= keep);
    }
}

fn lattice_value(seed: u64, ix: i32, iy: i32) -> f32 {
    let mut rng = SeedRng::new(chunk_seed(seed, ix, iy));
    rng.next_f32()
}

fn smoothstep(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t) // eases 0..1, flat at both ends
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

pub fn value_noise(seed: u64, x: f32, y: f32) -> f32 {
    let x0 = x.floor();
    let y0 = y.floor();
    let ix = x0 as i32;
    let iy = y0 as i32;

    let tx = smoothstep(x - x0);
    let ty = smoothstep(y - y0);

    let v00 = lattice_value(seed, ix, iy);
    let v10 = lattice_value(seed, ix + 1, iy);
    let v01 = lattice_value(seed, ix, iy + 1);
    let v11 = lattice_value(seed, ix + 1, iy + 1);

    let top = lerp(v00, v10, tx);
    let bottom = lerp(v01, v11, tx);
    lerp(top, bottom, ty)
}

pub fn world_to_chunk(world_x: f32, world_y: f32) -> (i32, i32) {
    let cx = (world_x / CHUNK_SIZE).floor() as i32;
    let cy = (world_y / CHUNK_SIZE).floor() as i32;
    (cx, cy)
}

pub fn chunk_to_world(cx: i32, cy: i32) -> (f32, f32) {
    (cx as f32 * CHUNK_SIZE, cy as f32 * CHUNK_SIZE)
}

fn object_color(kind: ObjectKind) -> Color {
    match kind {
        ObjectKind::Asteroid => GRAY,
        ObjectKind::Station => SKYBLUE,
        ObjectKind::Dungeon => RED,
    }
}

pub fn draw_world(world: &World, ship: &Player) {
    let half_w = screen_width() / 2.0;
    let half_h = screen_height() / 2.0;
    let (min_cx, min_cy) = world_to_chunk(ship.ship.pos.x - half_w, ship.ship.pos.y - half_h);
    let (max_cx, max_cy) = world_to_chunk(ship.ship.pos.x + half_w, ship.ship.pos.y + half_h);
    for cy in min_cy..=max_cy {
        for cx in min_cx..=max_cx {
            if CHUNK_DEBUG {
                let world_chunk = chunk_to_world(cx, cy);
                draw_rectangle_lines(
                    world_chunk.0,
                    world_chunk.1,
                    CHUNK_SIZE,
                    CHUNK_SIZE,
                    2.0,
                    GREEN,
                );
            }
            if let Some(chunk) = world.loaded.get(&(cx, cy)) {
                for obj in &chunk.objects {
                    let color =
                        if obj.kind == ObjectKind::Dungeon && ship.is_dungeon_cleared(&obj.id) {
                            DARKGRAY
                        } else {
                            object_color(obj.kind)
                        };
                    draw_circle(obj.x, obj.y, obj.radius, color);
                }
            }
        }
    }
}

pub fn draw_nebula(world: &World, player: &Player, seed: u64) {
    let _ = world;
    let half_w = screen_width() / 2.0;
    let half_h = screen_height() / 2.0;
    let (min_cx, min_cy) = world_to_chunk(player.ship.pos.x - half_w, player.ship.pos.y - half_h);
    let (max_cx, max_cy) = world_to_chunk(player.ship.pos.x + half_w, player.ship.pos.y + half_h);

    for cy in min_cy..=max_cy {
        for cx in min_cx..=max_cx {
            let wx = cx as f32 * CHUNK_SIZE;
            let wy = cy as f32 * CHUNK_SIZE;

            let n = value_noise(
                seed,
                (wx + CHUNK_SIZE / 2.0) / NEBULA_SCALE,
                (wy + CHUNK_SIZE / 2.0) / NEBULA_SCALE,
            );
            let intensity = (n - 0.4).max(0.0) * 0.5;
            let color = Color::new(0.4, 0.1, 0.6, intensity);
            draw_rectangle(wx, wy, CHUNK_SIZE, CHUNK_SIZE, color);
        }
    }
}

pub fn nearest_dungeon(world: &World, player: &Player) -> Option<SpaceObject> {
    let mut best: Option<SpaceObject> = None;
    let mut best_dist = INTERACT_RANGE;

    for chunk in world.loaded.values() {
        for obj in &chunk.objects {
            if obj.kind != ObjectKind::Dungeon || player.is_dungeon_cleared(&obj.id) {
                continue;
            }
            let dx = obj.x - player.ship.pos.x;
            let dy = obj.y - player.ship.pos.y;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist < best_dist {
                best_dist = dist;
                best = Some(*obj);
            }
        }
    }
    best
}
