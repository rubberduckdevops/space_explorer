# Space Explorer

A 2D space exploration game written in Rust with [macroquad](https://macroquad.rs/).
Fly your ship through an endless, procedurally-generated starfield, drift between
asteroids and stations, and dive into dungeons for combat.

## Features

- **Infinite procedural world** — the map is generated on the fly from a single
  world seed. The same seed always produces the same universe (deterministic
  `splitmix64` / FNV hashing, no stored map data).
- **Chunk streaming** — space is divided into 512×512 chunks that load around the
  player and unload as you leave them, so the world stays memory-light no matter
  how far you travel.
- **Space objects** — each chunk is seeded with asteroids, stations, and dungeons.
- **Dungeons & combat** — fly into range of a dungeon and enter it to start a
  seeded combat encounter *(combat is currently a placeholder loop)*.
- **Parallax starfield** — layered background stars scroll at different speeds for
  a sense of depth.

## Controls

| Input | Action |
|-------|--------|
| `W` `A` `S` `D` / Arrow keys | Move the ship |
| `Left Shift` | Toggle hyperdrive (fast travel) |
| `E` | Enter a dungeon when one is in range |
| `Q` | Win and leave the current dungeon *(placeholder)* |
| `L` | Die and leave the current dungeon *(placeholder)* |
| `Space` | Return to space after a dungeon ends |

## Building & running

Requires a [Rust toolchain](https://rustup.rs/) (edition 2024).

```sh
cargo run            # debug build
cargo run --release  # optimized build
```

### Web (WebAssembly)

macroquad targets the browser via WASM. Build with:

```sh
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown
```

Then serve the generated `.wasm` alongside a macroquad HTML shell. See the
[macroquad WASM docs](https://github.com/not-fl3/macroquad#wasm) for the loader
page.

## Configuration

A few tunables live as constants at the top of `src/main.rs`:

- `WORLD_SEED_STRING` — the seed for the universe. Change it for a whole new map.
- `LOAD_RADIUS` — how many chunks around the ship stay loaded.
- `CHUNK_DEBUG` — draws chunk boundaries when `true`.

## Project layout

| File | Responsibility |
|------|----------------|
| `src/main.rs` | Game loop, input, rendering, state machine |
| `src/chunk.rs` | Chunk and space-object generation |
| `src/seed.rs` | Deterministic RNG, world seeding, chunk streaming |
| `src/common.rs` | Shared pieces (e.g. the parallax star layers) |

## License

Licensed under either of

- MIT license ([LICENSE-MIT](LICENSE-MIT))
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this project by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
