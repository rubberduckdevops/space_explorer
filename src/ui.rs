use macroquad::ui::{Skin, hash, root_ui, widgets};
use macroquad::{prelude::*, text};

use crate::chunk::World;
use crate::common::{draw_bottom_left, draw_bottom_right, draw_centered};

/// A clickable button. Draws itself and returns true on the frame it is clicked.
/// Call in screen space (after set_default_camera()).
pub fn button(label: &str, rect: Rect) -> bool {
    let (mx, my) = mouse_position();
    let hovered = rect.contains(vec2(mx, my));

    let bg = if hovered { DARKGRAY } else { GRAY };
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, bg);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, WHITE);

    let d = measure_text(label, None, 28, 1.0);
    draw_text(
        label,
        rect.x + rect.w / 2.0 - d.width / 2.0,
        rect.y + rect.h / 2.0 + d.height / 2.0,
        28.0,
        WHITE,
    );

    hovered && is_mouse_button_pressed(MouseButton::Left)
}

/// A standard 220x50 button, horizontally centered, at a given y.
fn centered_button(y: f32) -> Rect {
    let w = 220.0;
    Rect::new(screen_width() / 2.0 - w / 2.0, y, w, 50.0)
}

/// Screen-space HUD shown while exploring. Call after set_default_camera().
pub fn exploring_hud(
    world: &World,
    ship_chunk: (i32, i32),
    mouse_world: Vec2,
    dungeon_in_range: bool,
) {
    if dungeon_in_range {
        draw_text(
            "● Dungeon in range — press E to enter",
            10.0,
            90.0,
            26.0,
            RED,
        );
    }
    draw_bottom_left(
        &[
            &format!(
                "Mouse position: ({:.0}, {:.0})",
                mouse_world.x, mouse_world.y
            ),
            &format!(
                "chunk {:?} loaded chunks: {} pending {}",
                ship_chunk,
                world.loaded.len(),
                world.pending.len(),
            ),
        ],
        20,
        WHITE,
    );
    draw_bottom_right(&["Demo - v0.0.1"], 16, WHITE);
}

/// What the pause menu is asking the game to do this frame.
pub enum PauseAction {
    Resume,
    Save,
    Quit,
}

/// Draws the pause overlay and returns an action if a button was pressed.
/// Call after set_default_camera().
pub fn pause_menu() -> Option<PauseAction> {
    // dim the frame behind the menu
    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        screen_height(),
        Color::new(0.0, 0.0, 0.0, 0.6),
    );
    draw_centered("PAUSED", 200.0, 56, WHITE);

    if button("Resume", centered_button(300.0)) {
        return Some(PauseAction::Resume);
    }
    if button("Save", centered_button(370.0)) {
        return Some(PauseAction::Save);
    }
    if button("Quit", centered_button(440.0)) {
        return Some(PauseAction::Quit);
    }
    None
}

pub enum StartMenuAction {
    Start,
}

pub fn start_menu(texture: &Texture2D) -> Option<StartMenuAction> {
    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        screen_height(),
        Color::new(0.0, 0.0, 0.0, 0.6),
    );
    let image_width = texture.width() / 2.0;
    let image_height = texture.height() / 2.0;
    draw_texture(
        texture,
        screen_width() / 2.0 - image_width,
        screen_height() / 2.0 - image_height,
        WHITE,
    );
    draw_centered("Space Explorer - v0.0.1", 50.0, 56, WHITE);

    if button("START", centered_button(screen_height() - 100.0)) {
        return Some(StartMenuAction::Start);
    }
    None
}

/// Builds the skin used for menu-style `macroquad::ui` widgets.
///
/// Colors + font size + spacing only — no image assets required. Build this
/// ONCE (it allocates), store it, and push it around the widgets you want
/// styled. Add `.background(Image)` + `.background_margin(RectOffset)` later
/// for 9-slice panel art.
pub fn build_menu_skin() -> Skin {
    let window_style = root_ui()
        .style_builder()
        .color(Color::from_rgba(18, 22, 34, 235))
        .build();

    let window_titlebar_style = root_ui()
        .style_builder()
        .color(Color::from_rgba(30, 38, 60, 255))
        .text_color(Color::from_rgba(150, 190, 255, 255))
        .font_size(24)
        .build();

    let label_style = root_ui()
        .style_builder()
        .text_color(Color::from_rgba(200, 210, 230, 255))
        .font_size(22)
        .build();

    let editbox_style = root_ui()
        .style_builder()
        .color(Color::from_rgba(10, 12, 20, 255))
        .text_color(DARKGREEN) // terminal-green input
        .font_size(24)
        .margin(RectOffset::new(8.0, 8.0, 6.0, 6.0))
        .build();

    let button_style = root_ui()
        .style_builder()
        .color(Color::from_rgba(40, 60, 110, 255))
        .color_hovered(Color::from_rgba(60, 90, 160, 255))
        .color_clicked(Color::from_rgba(28, 42, 80, 255))
        .text_color(WHITE)
        .font_size(24)
        .margin(RectOffset::new(16.0, 16.0, 8.0, 8.0))
        .build();

    // Fill everything we didn't override from the stock skin.
    Skin {
        window_style,
        window_titlebar_style,
        label_style,
        editbox_style,
        button_style,
        ..root_ui().default_skin()
    }
}

/// What the name-entry screen is asking the game to do.
pub enum NameEntryAction {
    Confirm,
}

/// Draws the username entry window. `username` is owned by the caller (so its
/// text survives between frames) and edited in place by the editbox widget.
/// `skin` is pushed for the duration of the window, so this screen looks like
/// `build_menu_skin()` regardless of what skin is otherwise active.
/// Returns Some(Confirm) on the frame the user confirms a non-empty name.
pub fn name_entry(username: &mut String, skin: &Skin) -> Option<NameEntryAction> {
    root_ui().push_skin(skin);

    let size = vec2(320.0, 160.0);
    let pos = vec2(
        screen_width() / 2.0 - size.x / 2.0,
        screen_height() / 2.0 - size.y / 2.0,
    );

    // The closure can't `return` out of the function, so it records intent in a
    // variable we read after the window is drawn.
    let mut action = None;
    widgets::Window::new(hash!(), pos, size)
        .label("Enter Name")
        .ui(&mut root_ui(), |ui| {
            ui.label(None, "Username:");
            ui.input_text(hash!(), "", username);
            if ui.button(None, "Confirm") && !username.is_empty() {
                action = Some(NameEntryAction::Confirm);
            }
        });

    root_ui().pop_skin();
    action
}
