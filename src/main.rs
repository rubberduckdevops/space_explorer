mod common;
mod seed;
mod chunk;

use crate::{common::StarLayer, seed::hash_seed_string, chunk::{generate_chunk, ObjectKind}};
use macroquad::prelude::*;

struct PlayerShip {
    pos: Vec2,
    speed: f32,
}

// Chunk Stuff
const CHUNK_SIZE: f32 = 512.0;
const WORLD_SEED_STRING: &str = "hello";

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

#[macroquad::main("Space Explorer")]
async fn main() {
    // Initialize
    log::info!("Initalizing Systems");
    log::info!("World Seed String: {}", WORLD_SEED_STRING);
    let world_seed = hash_seed_string(WORLD_SEED_STRING);
    log::info!("World Seed Hash: {}", &world_seed);

    let mut ship = PlayerShip {
        pos: vec2(0.0, 0.0),
        speed: 300.0,
    };

    let layers = [
        StarLayer::new(40, 600.0, 0.15, 1.0, Color::new(0.6, 0.6, 0.7, 1.0)),
        StarLayer::new(30, 500.0, 0.30, 1.5, Color::new(0.8, 0.8, 0.9, 1.0)),
        StarLayer::new(20, 400.0, 0.55, 2.0, WHITE),
    ];

    // Start Game loop here
    log::info!("Starting Primary Loop");
    loop {
        clear_background(BLACK);
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

        // Spawn Position
        draw_line(-20.0, 0.0, 20.0, 0.0, 1.0, DARKGRAY);
        draw_line(0.0, -20.0, 0.0, 20.0, 1.0, DARKGRAY);


        // Chunks — borders in WORLD space (shapes ignore the camera's flipped Y)
        let ((min_cx, min_cy), (max_cx, max_cy)) = visible_chunk_range(ship.pos.x, ship.pos.y);
        let (ship_cx, ship_cy) = world_to_chunk(ship.pos.x, ship.pos.y);
        for cy in min_cy..=max_cy {
            for cx in min_cx..=max_cx {
                let chunk = generate_chunk(world_seed, cx, cy);
                for obj in &chunk.objects {
                    draw_circle(obj.x, obj.y, obj.radius, object_color(obj.kind));
                }
                // Debug Chunk Lines
                let wx = cx as f32 * CHUNK_SIZE;
                let wy = cy as f32 * CHUNK_SIZE;
                let color = if (cx, cy) == (ship_cx, ship_cy) {
                    GREEN
                } else {
                    DARKGREEN
                };
                draw_rectangle_lines(wx, wy, CHUNK_SIZE, CHUNK_SIZE, 2.0, color);
            }
        }

        //Player Drawing

        draw_circle(ship.pos.x, ship.pos.y, 16.0, YELLOW);

        set_default_camera();

        // Chunk labels — drawn in SCREEN space so the world camera's flipped Y
        // doesn't render the text upside-down. Convert each chunk corner to screen.
        for cy in min_cy..=max_cy {
            for cx in min_cx..=max_cx {
                let wx = cx as f32 * CHUNK_SIZE;
                let wy = cy as f32 * CHUNK_SIZE;
                // project the chunk's world corner into screen pixels (1:1 zoom here)
                let corner = cam.world_to_screen(vec2(wx, wy));
                let color = if (cx, cy) == (ship_cx, ship_cy) {
                    GREEN
                } else {
                    DARKGREEN
                };
                draw_text(&format!("({},{})", cx, cy), corner.x + 12.0, corner.y + 34.0, 26.0, color);
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
            &format!("world ({:.0}, {:.0})   chunk ({}, {})", ship.pos.x, ship.pos.y, ship_cx, ship_cy),
            10.0, 30.0, 26.0, WHITE,
        );
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