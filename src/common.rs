use macroquad::prelude::*;

pub struct StarLayer {
    pub stars: Vec<Vec2>,
    pub tile: f32,
    pub parallax: f32,
    pub size: f32,
    pub color: Color,
}

impl StarLayer {
    pub fn new(count: usize, tile: f32, parallax: f32, size: f32, color: Color) -> StarLayer {
        let mut stars = Vec::new();
        for _ in 0..count {
            stars.push(vec2(rand::gen_range(0.0, tile), rand::gen_range(0.0, tile)));
        }
        StarLayer {
            stars,
            tile,
            parallax,
            size,
            color,
        }
    }

    pub fn draw(&self, cam_x: f32, cam_y: f32) {
        // haw far this layer has scrolled, wrapped into [0, tile]
        let offset_x = (cam_x * self.parallax).rem_euclid(self.tile);
        let offset_y = (cam_y * self.parallax).rem_euclid(self.tile);

        // stamp the tile across the screen, starting opne tile before the edge
        let mut tile_x = -offset_x;
        while tile_x < screen_width() {
            let mut tile_y = -offset_y;
            while tile_y < screen_height() {
                for pos in &self.stars {
                    draw_circle(tile_x + pos.x, tile_y + pos.y, self.size, self.color);
                }
                tile_y += self.tile;
            }
            tile_x += self.tile;
        }
    }
}
