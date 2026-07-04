mod chunk;
mod common;
mod player;

use crate::{
    chunk::{
        Chunk, ObjectKind, SeedRng, SpaceObject, World, generate_chunk, hash_seed_string,
        value_noise,
    },
    common::{
        StarLayer, draw_bottom_left, draw_centered,
        save::{load_game, save_game},
    },
    player::{Player, PlayerShip},
};
use macroquad::prelude::*;

enum GameState {
    Exploring,
    InCombat {
        dungeon_id: u64,
        combat: CombatInstance,
    },
}

// Chunk Stuff
const CHUNK_SIZE: f32 = 512.0;
const WORLD_SEED_STRING: &str = "hello";
const LOAD_RADIUS: i32 = 2;
const CHUNK_DEBUG: bool = true;
const INTERACT_RANGE: f32 = 70.0;
const ARENA_WIDTH: f32 = 800.0;
const ARENA_HEIGHT: f32 = 600.0;
const COMBAT_DURATION: f32 = 20.0;

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

fn draw_world(world: &World, ship: &Player) {
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

fn nearest_dungeon(world: &World, player: &Player) -> Option<SpaceObject> {
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

enum CombatOutcome {
    Win { score: u32 },
    Lost,
}

struct CombatInstance {
    player: PlayerShip,
    score: u32,
    spawn_timer: f32,
    spawn_interval: f32,
    time_left: f32,
    rng: SeedRng, // seeded from the dungeon id → reproducible waves
    outcome: Option<CombatOutcome>,
}

impl CombatInstance {
    fn new(dungeon_id: u64, player: PlayerShip) -> CombatInstance {
        let mut rng = SeedRng::new(dungeon_id);
        let spawn_interval = rng.range_f32(0.35, 0.65);
        CombatInstance {
            player: player,
            score: 0,
            spawn_timer: 0.0,
            spawn_interval,
            time_left: COMBAT_DURATION,
            rng,
            outcome: None,
        }
    }

    fn update(&mut self, delta: f32) {
        self.time_left -= delta;
        if self.time_left <= 0.0 {
            self.outcome = Some(CombatOutcome::Win { score: 20 });
            return;
        }
        if is_key_pressed(KeyCode::Q) {
            self.outcome = Some(CombatOutcome::Win { score: 20 });
        }
        if is_key_pressed(KeyCode::L) {
            self.outcome = Some(CombatOutcome::Lost);
        }
    }
    fn draw(&self) {
        // arena camera: map the fixed arena rect onto the whole window
        set_camera(&Camera2D::from_display_rect(Rect::new(
            0.0,
            0.0,
            ARENA_WIDTH,
            ARENA_HEIGHT,
        )));
        draw_rectangle_lines(0.0, 0.0, ARENA_WIDTH, ARENA_HEIGHT, 4.0, DARKGRAY);
        self.player.draw();
        set_default_camera();
        draw_text("IN DUNGEON (placeholder)", 40.0, 200.0, 50.0, YELLOW);
        draw_text(
            &format!("dungeon id: {}", self.rng.state),
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
        // HUD in screen space

        draw_text(&format!("Score: {}", self.score), 12.0, 32.0, 30.0, WHITE);
        draw_text(
            &format!("Survive: {:.1}s", self.time_left.max(0.0)),
            12.0,
            62.0,
            28.0,
            GOLD,
        );
    }
}

const NEBULA_SCALE: f32 = 1400.0;
fn draw_nebula(world: &World, player: &Player, seed: u64) {
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

#[macroquad::main("Space Explorer")]
async fn main() {
    // Initialize
    log::info!("Initalizing Systems");
    log::info!("Loading Previous Save");
    let save = load_game();
    let seed_string = save
        .as_ref()
        .map(|s| s.seed_string.clone())
        .unwrap_or_else(|| WORLD_SEED_STRING.to_string());

    log::info!("World Seed String: {}", seed_string);
    let world_seed = hash_seed_string(&seed_string);
    log::info!("World Seed Hash: {}", &world_seed);
    let mut world = World::new(world_seed);

    let mut player = save
        .as_ref()
        .map(|s| Player::load_player(s.player.clone()))
        .unwrap_or_else(|| Player::new());

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

        match &mut game_state {
            GameState::Exploring => {
                // Player Movement
                if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
                    player.ship.pos.x += player.ship.speed * delta;
                };
                if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
                    player.ship.pos.x -= player.ship.speed * delta;
                };
                if is_key_down(KeyCode::Down) || is_key_down(KeyCode::S) {
                    player.ship.pos.y -= player.ship.speed * delta;
                };
                if is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) {
                    player.ship.pos.y += player.ship.speed * delta;
                };
                if is_key_pressed(KeyCode::LeftShift) || is_key_released(KeyCode::LeftShift) {
                    player.ship.togle_hyperdrive();
                }
                if is_key_pressed(KeyCode::P) {
                    save_game(&seed_string, &player);
                }
                let ship_chunk = world_to_chunk(player.ship.pos.x, player.ship.pos.y);
                world.stream_around(ship_chunk, LOAD_RADIUS);
                // Background Stars Rendering
                // for layer in &layers {
                //     layer.draw(player.ship.pos.x, player.ship.pos.y);
                // }

                // --- WORLD SPACE: activate the follow-camera, then draw world things ---
                let cam = Camera2D::from_display_rect(Rect::new(
                    player.ship.pos.x - screen_width() / 2.0,
                    player.ship.pos.y - screen_height() / 2.0,
                    screen_width(),
                    screen_height(),
                ));
                set_camera(&cam);
                draw_nebula(&world, &player, world_seed);
                let mouse_world = cam.screen_to_world(mouse_position().into());

                draw_world(&world, &player);

                //Player Drawing
                player.ship.draw();

                let target = nearest_dungeon(&world, &player);
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
                        next_state = Some(GameState::InCombat {
                            dungeon_id: d.id,
                            combat: CombatInstance::new(d.id, player.ship.clone()),
                        });
                    }
                }
                draw_bottom_left(
                    &[
                        &format!(
                            "Mouse position: ({:.0}, {:.0})",
                            mouse_world.x, mouse_world.y
                        ),
                        &format!(
                            "chunk {:?}   loaded chunks: {}",
                            ship_chunk,
                            world.loaded.len()
                        ),
                    ],
                    20,
                    WHITE,
                );

                player.draw_player_stats();
            }
            GameState::InCombat { dungeon_id, combat } => {
                combat.update(delta);
                combat.draw();
                if let Some(outcome) = &combat.outcome {
                    set_default_camera();
                    match outcome {
                        CombatOutcome::Win { score } => {
                            draw_centered("DUNGEON CLEARED", 240.0, 56, GREEN);
                            draw_centered(&format!("+{score} score"), 300.0, 34, WHITE);
                        }
                        CombatOutcome::Lost => {
                            draw_centered("YOU DIED", 240.0, 56, RED);
                        }
                    }
                    draw_centered("Press SPACE to return to space", 360.0, 26, GRAY);

                    if is_key_pressed(KeyCode::Space) {
                        if let CombatOutcome::Win { score } = outcome {
                            player.credits += score;
                            player.clear_dungeon(*dungeon_id);
                        }
                        let _ = dungeon_id;
                        next_state = Some(GameState::Exploring);
                    }
                }
            }
        }

        if let Some(state) = next_state.take() {
            game_state = state;
        }

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
