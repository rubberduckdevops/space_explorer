use crate::chunk::SeedRng;
use crate::player::PlayerShip;
use macroquad::prelude::*;

const ARENA_WIDTH: f32 = 800.0;
const ARENA_HEIGHT: f32 = 600.0;
const COMBAT_DURATION: f32 = 20.0;

pub enum CombatOutcome {
    Win { score: u32 },
    Lost,
}

pub struct CombatInstance {
    player: PlayerShip,
    score: u32,
    #[allow(dead_code)]
    spawn_timer: f32,
    #[allow(dead_code)]
    spawn_interval: f32,
    time_left: f32,
    rng: SeedRng, // seeded from the dungeon id → reproducible waves
    pub outcome: Option<CombatOutcome>,
}

impl CombatInstance {
    pub fn new(dungeon_id: u64, player: PlayerShip) -> CombatInstance {
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

    pub fn update(&mut self, delta: f32) {
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
    pub fn draw(&self) {
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
