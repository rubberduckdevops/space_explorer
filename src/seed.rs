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