use crate::chunk::{Chunk, generate_chunk};
use std::{collections::HashMap, hash::Hash};

/// splitmix64 finalizer — scrambles a u64 thoroughly.
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

pub struct World {
    pub seed: u64,
    pub loaded: HashMap<(i32, i32), Chunk>,
}

impl World {
    pub fn new(seed: u64) -> World {
        log::info!("Generating World");
        World {
            seed,
            loaded: HashMap::new(),
        }
    }

    pub fn ensure_chunk(&mut self, cx: i32, cy: i32) {
        let seed = self.seed;
        self.loaded
            .entry((cx, cy))
            .or_insert_with(|| generate_chunk(seed, cx, cy));
    }
    pub fn stream_around(&mut self, center: (i32, i32), radius: i32) {
        // Load Chunks in the Radius of Player
        for cy in (center.1 - radius)..=(center.1 + radius) {
            for cx in (center.0 - radius)..=(center.0 + radius) {
                self.ensure_chunk(cx, cy);
            }
        }
        // Unload Chunks that drifted away from Player
        let keep = radius + 1;
        self.loaded
            .retain(|&(cx, cy), _| (cx - center.0).abs() <= keep && (cy - center.1).abs() <= keep);
    }
}

fn lattice_value(seed: u64, ix: i32, iy: i32) -> f32 {
    let mut rng = SeedRng::new(chunk_seed(seed, ix,iy));
    rng.next_f32()
}

fn smoothstep(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)// eases 0..1, flat at both ends
}

fn lerp(a:f32, b:f32, t: f32) -> f32 {
    a + (b -a) * t
}

pub fn value_noise(seed: u64, x:f32, y: f32) -> f32 {
    let x0 = x.floor(); 
    let y0 = y.floor();
    let ix = x0 as i32; 
    let iy = y0 as i32;

    let tx = smoothstep(x - x0); 
    let ty = smoothstep(y - y0);

    let v00 = lattice_value(seed, ix, iy); 
    let v10 = lattice_value(seed, ix +1, iy); 
    let v01 = lattice_value(seed, ix, iy + 1);
    let v11 = lattice_value(seed, ix + 1, iy + 1); 

    let top = lerp(v00, v10, tx); 
    let bottom = lerp(v01, v11, tx); 
    lerp(top, bottom, ty)
}