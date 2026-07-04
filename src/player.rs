use std::collections::HashSet;

use crate::common::{draw_centered, save::PlayerSaveData};
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug)]
pub struct PlayerShip {
    pub pos: Vec2,
    pub speed: f32,
    pub hyper_drive: bool,
}

impl PlayerShip {
    fn new() -> PlayerShip {
        PlayerShip {
            pos: vec2(0.0, 0.0),
            speed: 300.0,
            hyper_drive: false,
        }
    }

    pub fn draw(&self) {
        draw_circle(self.pos.x, self.pos.y, 16.0, YELLOW);
    }

    pub fn togle_hyperdrive(&mut self) {
        if self.hyper_drive {
            self.hyper_drive = false;
            self.speed = 300.0;
        } else {
            self.hyper_drive = true;
            self.speed = 1000.0;
        }
    }
}
#[derive(Clone, Debug)]
pub struct Player {
    pub ship: PlayerShip,
    pub credits: u32,
    pub speed_level: u32,
    cleared_dungeons: HashSet<u64>,
}

impl Player {
    pub fn new() -> Player {
        Player {
            ship: PlayerShip::new(),
            credits: 0,
            speed_level: 0,
            cleared_dungeons: HashSet::new(),
        }
    }
    pub fn load_player(saved_player: PlayerSaveData) -> Player {
        log::info!("Loading Player from save");
        let mut player_ship = PlayerShip::new();
        player_ship.pos.x = saved_player.ship_pos_x;
        player_ship.pos.y = saved_player.ship_pos_y;

        Player {
            ship: player_ship,
            credits: saved_player.credits,
            speed_level: saved_player.speed_level,
            cleared_dungeons: saved_player.cleared_dungeons,
        }
    }
    pub fn save_player(&self) -> PlayerSaveData {
        PlayerSaveData {
            ship_pos_x: self.ship.pos.x,
            ship_pos_y: self.ship.pos.y,
            credits: self.credits,
            speed_level: self.speed_level,
            cleared_dungeons: self.cleared_dungeons.clone(),
        }
    }
    pub fn clear_dungeon(&mut self, dungeon_id: u64) {
        // Maybe do some validation one day here??
        self.cleared_dungeons.insert(dungeon_id);
    }
    pub fn is_dungeon_cleared(&self, dungeon_id: &u64) -> bool {
        self.cleared_dungeons.contains(dungeon_id)
    }

    pub fn draw_player_stats(&self) {
        draw_centered(&format!("Credits: {}", self.credits), 16.0, 20, WHITE);
        draw_centered(
            &format!("HyperDrive: {}", self.ship.hyper_drive),
            32.0,
            20,
            WHITE,
        );
    }
}
