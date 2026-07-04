mod chunk;
mod common;
mod seed;

use crate::{
    chunk::{Chunk, ObjectKind, SpaceObject, generate_chunk},
    common::StarLayer,
    seed::{World, hash_seed_string},
};
use macroquad::prelude::*;

struct PlayerShip {
    pos: Vec2,
    speed: f32,
    hyper_drive: bool,
}

enum GameState {
    Exploring,
    InCombat { dungeon_id: u64 },
}

// Chunk Stuff
const CHUNK_SIZE: f32 = 512.0;
const WORLD_SEED_STRING: &str = "hello";
const LOAD_RADIUS: i32 = 2;
const CHUNK_DEBUG: bool = true;
const INTERACT_RANGE: f32 = 70.0;

fn world_to_chunk(world_x: f32, world_y: f32) -> (i32, i32) {
    let cx = (world_x / CHUNK_SIZE).floor() as i32;
    let cy = (world_y / CHUNK_SIZE).floor() as i32;
    (cx, cy)
}
fn chunk_to_world(cx: i32, cy: i32) -> (f32, f32) {
    (cx as f32 * CHUNK_SIZE, cy as f32 * CHUNK_SIZE)
}

fn world_to_local(world_x: f32, world_y: f32) -> (f32, f32) {
    (
        world_x.rem_euclid(CHUNK_SIZE),
        world_y.rem_euclid(CHUNK_SIZE),
    )
}

fn visible_chunk_range(ship_x: f32, ship_y: f32) -> ((i32, i32), (i32, i32)) {
    let half_w = screen_width() / 2.0;
    let half_h = screen_height() / 2.0;
    let min = world_to_chunk(ship_x - half_w, ship_y - half_h); // top-left
    let max = world_to_chunk(ship_x + half_w, ship_y + half_h); // bottom-right

    (min, max)
}

fn draw_world(world: &World, ship: &PlayerShip) {
    let half_w = screen_width() / 2.0;
    let half_h = screen_height() / 2.0;
    let (min_cx, min_cy) = world_to_chunk(ship.pos.x - half_w, ship.pos.y - half_h);
    let (max_cx, max_cy) = world_to_chunk(ship.pos.x + half_w, ship.pos.y + half_h);
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
                    draw_circle(obj.x, obj.y, obj.radius, object_color(obj.kind));
                }
            }
        }
    }
}

fn nearest_dungeon(world: &World, ship: &PlayerShip) -> Option<SpaceObject> {
    let mut best: Option<SpaceObject> = None;
    let mut best_dist = INTERACT_RANGE;

    for chunk in world.loaded.values() {
        for obj in &chunk.objects {
            if obj.kind != ObjectKind::Dungeon {
                continue;
            }
            let dx = obj.x - ship.pos.x;
            let dy = obj.y - ship.pos.y;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist < best_dist {
                best_dist = dist;
                best = Some(*obj);
            }
        }
    }
    best
}

#[macroquad::main("Space Explorer")]
async fn main() {
    // Initialize
    log::info!("Initalizing Systems");
    log::info!("World Seed String: {}", WORLD_SEED_STRING);
    let world_seed = hash_seed_string(WORLD_SEED_STRING);
    log::info!("World Seed Hash: {}", &world_seed);
    let mut world = World::new(world_seed);

    let mut ship = PlayerShip {
        pos: vec2(0.0, 0.0),
        speed: 300.0,
        hyper_drive: false,
    };

    let layers = [
        StarLayer::new(40, 600.0, 0.15, 1.0, Color::new(0.6, 0.6, 0.7, 1.0)),
        StarLayer::new(30, 500.0, 0.30, 1.5, Color::new(0.8, 0.8, 0.9, 1.0)),
        StarLayer::new(20, 400.0, 0.55, 2.0, WHITE),
    ];

    // Start Game loop here
    let mut game_state = GameState::Exploring;
    log::info!("Starting Primary Loop");
    loop {
        clear_background(BLACK);
        let mut next_state: Option<GameState> = None;
        let delta = get_frame_time();

        // Player Movement
        if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
            ship.pos.x += ship.speed * delta;
        };
        if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
            ship.pos.x -= ship.speed * delta;
        };
        if is_key_down(KeyCode::Down) || is_key_down(KeyCode::S) {
            ship.pos.y -= ship.speed * delta;
        };
        if is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) {
            ship.pos.y += ship.speed * delta;
        };
        if is_key_pressed(KeyCode::LeftShift) || is_key_released(KeyCode::LeftShift) {
            if ship.hyper_drive {
                ship.hyper_drive = false;
                ship.speed = 300.0;
            } else {
                ship.hyper_drive = true;
                ship.speed = 1000.0;
            }
        }

        match &game_state {
            GameState::Exploring => {
                let ship_chunk = world_to_chunk(ship.pos.x, ship.pos.y);
                world.stream_around(ship_chunk, LOAD_RADIUS);
                // Background Stars Rendering
                for layer in &layers {
                    layer.draw(ship.pos.x, ship.pos.y);
                }

                // --- WORLD SPACE: activate the follow-camera, then draw world things ---
                let cam = Camera2D::from_display_rect(Rect::new(
                    ship.pos.x - screen_width() / 2.0,
                    ship.pos.y - screen_height() / 2.0,
                    screen_width(),
                    screen_height(),
                ));
                set_camera(&cam);
                let mouse_world = cam.screen_to_world(mouse_position().into());

                draw_world(&world, &ship);

                //Player Drawing
                draw_circle(ship.pos.x, ship.pos.y, 16.0, YELLOW);

                let target = nearest_dungeon(&world, &ship);
                if let Some(d) = target {
                    draw_circle_lines(d.x, d.y, d.radius + 5.0, 3.0, YELLOW);
                }

                // UI TEXT - Stuff
                // Since set_default_camera() does some weird inverse flipping shit
                // I have to put all text AFTER so it renders correctly
                // But all assets like shapes before this
                set_default_camera();
                if let Some(d) = target {
                    draw_text(
                        "● Dungeon in range — press E to enter",
                        10.0,
                        90.0,
                        26.0,
                        RED,
                    );
                    if is_key_pressed(KeyCode::E) {
                        next_state = Some(GameState::InCombat { dungeon_id: d.id });
                    }
                }
                draw_text(
                    &format!(
                        "Mouse position: ({:.0}, {:.0})",
                        mouse_world.x, mouse_world.y
                    ),
                    10.0,
                    60.0,
                    28.0,
                    WHITE,
                );

                draw_text(
                    &format!(
                        "chunk {:?}   loaded chunks: {}",
                        ship_chunk,
                        world.loaded.len()
                    ),
                    10.0,
                    30.0,
                    26.0,
                    WHITE,
                );
                draw_text(
                    &format!("HyperDrive: {:?}", ship.hyper_drive),
                    400.0,
                    30.0,
                    16.0,
                    WHITE,
                );
            }
            GameState::InCombat { dungeon_id } => {
                draw_text("IN DUNGEON (placeholder)", 40.0, 200.0, 50.0, YELLOW);
                draw_text(
                    &format!("dungeon id: {dungeon_id}"),
                    40.0,
                    250.0,
                    26.0,
                    WHITE,
                );
                draw_text(
                    "Q = win and leave    L = die and leave",
                    40.0,
                    300.0,
                    26.0,
                    GRAY,
                );
                if is_key_pressed(KeyCode::Q) || is_key_pressed(KeyCode::L) {
                    next_state = Some(GameState::Exploring);
                }
            }
        }

        if let Some(state) = next_state.take() {
            game_state = state;
        }

        // Chunks — borders in WORLD space (shapes ignore the camera's flipped Y)
        // let ((min_cx, min_cy), (max_cx, max_cy)) = visible_chunk_range(ship.pos.x, ship.pos.y);
        // let (ship_cx, ship_cy) = world_to_chunk(ship.pos.x, ship.pos.y);
        // for cy in min_cy..=max_cy {
        //     for cx in min_cx..=max_cx {
        //         let chunk = generate_chunk(world_seed, cx, cy);
        //         for obj in &chunk.objects {
        //             draw_circle(obj.x, obj.y, obj.radius, object_color(obj.kind));
        //         }
        //         // Debug Chunk Lines
        //         let wx = cx as f32 * CHUNK_SIZE;
        //         let wy = cy as f32 * CHUNK_SIZE;
        //         let color = if (cx, cy) == (ship_cx, ship_cy) {
        //             GREEN
        //         } else {
        //             DARKGREEN
        //         };
        //         draw_rectangle_lines(wx, wy, CHUNK_SIZE, CHUNK_SIZE, 2.0, color);
        //     }
        // }

        next_frame().await;
    }
}

// Helpers
fn object_color(kind: ObjectKind) -> Color {
    match kind {
        ObjectKind::Asteroid => GRAY,
        ObjectKind::Station => SKYBLUE,
        ObjectKind::Dungeon => RED,
    }
}
