use macroquad::prelude::*;

// #[allow(dead_code)]
// pub struct StarLayer {
//     pub stars: Vec<Vec2>,
//     pub tile: f32,
//     pub parallax: f32,
//     pub size: f32,
//     pub color: Color,
// }

// #[allow(dead_code)]
// impl StarLayer {
//     pub fn new(count: usize, tile: f32, parallax: f32, size: f32, color: Color) -> StarLayer {
//         let mut stars = Vec::new();
//         for _ in 0..count {
//             stars.push(vec2(rand::gen_range(0.0, tile), rand::gen_range(0.0, tile)));
//         }
//         StarLayer {
//             stars,
//             tile,
//             parallax,
//             size,
//             color,
//         }
//     }

//     pub fn draw(&self, cam_x: f32, cam_y: f32) {
//         // haw far this layer has scrolled, wrapped into [0, tile]
//         let offset_x = (cam_x * self.parallax).rem_euclid(self.tile);
//         let offset_y = (cam_y * self.parallax).rem_euclid(self.tile);

//         // stamp the tile across the screen, starting opne tile before the edge
//         let mut tile_x = -offset_x;
//         while tile_x < screen_width() {
//             let mut tile_y = -offset_y;
//             while tile_y < screen_height() {
//                 for pos in &self.stars {
//                     draw_circle(tile_x + pos.x, tile_y + pos.y, self.size, self.color);
//                 }
//                 tile_y += self.tile;
//             }
//             tile_x += self.tile;
//         }
//     }
// }

pub fn draw_centered(text: &str, y: f32, font_size: u16, color: Color) {
    let d = measure_text(text, None, font_size, 1.0);
    draw_text(
        text,
        screen_width() / 2.0 - d.width / 2.0,
        y,
        font_size as f32,
        color,
    );
}

const MARGIN: f32 = 12.0;
const LINE_SPACING: f32 = 4.0;

// draw lines of text stacked upward from the bottom-left corner
pub fn draw_bottom_left(lines: &[&str], font_size: u16, color: Color) {
    let line_height = font_size as f32 + LINE_SPACING;
    // last line sits at the bottom, earlier lines stack above it
    for (i, line) in lines.iter().rev().enumerate() {
        let y = screen_height() - MARGIN - line_height * i as f32;
        draw_text(line, MARGIN, y, font_size as f32, color);
    }
}

// draw lines of text stacked upward from the bottom-right corner
pub fn draw_bottom_right(lines: &[&str], font_size: u16, color: Color) {
    let line_height = font_size as f32 + LINE_SPACING;
    // last line sits at the bottom, earlier lines stack above it
    for (i, line) in lines.iter().rev().enumerate() {
        let d = measure_text(line, None, font_size, 1.0);
        let y = screen_height() - MARGIN - line_height * i as f32;
        draw_text(
            line,
            screen_width() - MARGIN - d.width,
            y,
            font_size as f32,
            color,
        );
    }
}

pub mod save {
    use std::collections::HashSet;

    use serde::{Deserialize, Serialize};

    use crate::player::Player;
    const SAVE_PATH: &str = "explorer_save.json";

    #[derive(Serialize, Deserialize, Clone)]
    pub struct PlayerSaveData {
        pub ship_pos_x: f32,
        pub ship_pos_y: f32,
        pub credits: u32,
        pub speed_level: u32,
        pub cleared_dungeons: HashSet<u64>,
    }

    #[derive(Serialize, Deserialize)]
    pub struct SaveData {
        pub seed_string: String,
        pub player: PlayerSaveData,
    }

    pub fn save_game(seed_string: &str, player_data: &Player) {
        log::info!("Saving Data...");
        let data = SaveData {
            seed_string: seed_string.to_string(),
            player: player_data.save_player(),
        };
        match serde_json::to_string_pretty(&data) {
            Ok(json) => {
                if let Err(e) = std::fs::write(SAVE_PATH, json) {
                    log::error!("Could not write save: {e}");
                    eprintln!("could not write save: {e}")
                }
            }
            Err(e) => {
                log::error!("Could not seralize save: {e}");
                eprintln!("Could not seralize save: {e}");
            }
        }
    }
    pub fn load_game() -> Option<SaveData> {
        let text = std::fs::read_to_string(SAVE_PATH).ok()?;
        serde_json::from_str(&text).ok()
    }
}
