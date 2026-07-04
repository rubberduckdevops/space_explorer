mod chunk;
mod combat;
mod common;
mod player;
mod ui;

use crate::{
    chunk::{
        LOAD_RADIUS, World, draw_nebula, draw_world, hash_seed_string, nearest_dungeon,
        world_to_chunk,
    },
    combat::{CombatInstance, CombatOutcome},
    common::{
        draw_centered,
        save::{load_game, save_game},
    },
    player::Player,
    ui::PauseAction,
};
use macroquad::prelude::*;

enum GameState {
    Start,
    Exploring,
    Paused,
    InCombat {
        dungeon_id: u64,
        combat: CombatInstance,
    },
}

const WORLD_SEED_STRING: &str = "hello";

#[macroquad::main("Space Explorer")]
async fn main() {
    env_logger::init();
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

    let mut username = String::new();
    let menu_skin = ui::build_menu_skin();
    // Start Game loop here
    let mut game_state = GameState::Start;
    log::info!("Starting Primary Loop");
    loop {
        clear_background(BLACK);
        let mut next_state: Option<GameState> = None;
        let delta = get_frame_time();

        match &mut game_state {
            GameState::Start => {
                set_default_camera();
                if let Some(ui::NameEntryAction::Confirm) = ui::name_entry(&mut username, &menu_skin)
                {
                    next_state = Some(GameState::Exploring);
                }
            }
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
                if is_key_pressed(KeyCode::Escape) {
                    next_state = Some(GameState::Paused);
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
                let mouse_world = cam.screen_to_world(mouse_position().into());

                // Draw Environment and Players
                draw_nebula(&world, &player, world_seed);
                draw_world(&world, &player);
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
                let dungeon_in_range = target.is_some();
                if let Some(d) = target {
                    if is_key_pressed(KeyCode::E) {
                        next_state = Some(GameState::InCombat {
                            dungeon_id: d.id,
                            combat: CombatInstance::new(d.id, player.ship.clone()),
                        });
                    }
                }
                ui::exploring_hud(&world, ship_chunk, mouse_world, dungeon_in_range);
                player.draw_player_stats();
            }
            GameState::Paused => {
                set_default_camera();
                match ui::pause_menu() {
                    Some(PauseAction::Resume) => next_state = Some(GameState::Exploring),
                    Some(PauseAction::Save) => {
                        save_game(&seed_string, &player);
                        next_state = Some(GameState::Exploring);
                    }
                    Some(PauseAction::Quit) => std::process::exit(0),
                    None => {}
                }
                if is_key_pressed(KeyCode::Escape) {
                    next_state = Some(GameState::Exploring);
                }
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
