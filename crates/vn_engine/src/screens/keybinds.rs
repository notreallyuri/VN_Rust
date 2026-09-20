use std::rc::Rc;

use raylib::prelude::*;

use crate::PanelStyle;
use crate::screens::PlayingConfig;
use crate::ui::{self, ButtonStyle, TextStyle};
use crate::{
    DrawContext, Focus, FontRole, GameContext, NavigationConfig, Overlay, OverlayAction,
    RollbackConfig,
};

pub const KEYBINDS_OVERLAY: &str = "keybinds";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyRow {
    pub keys: String,
    pub gamepad: String,
    pub action: String,
}

impl KeyRow {
    pub fn new(
        keys: impl Into<String>,
        gamepad: impl Into<String>,
        action: impl Into<String>,
    ) -> Self {
        Self {
            keys: keys.into(),
            gamepad: gamepad.into(),
            action: action.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeySection {
    pub title: String,
    pub rows: Vec<KeyRow>,
}

impl KeySection {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            rows: Vec::new(),
        }
    }

    pub fn row(
        mut self,
        keys: impl Into<String>,
        gamepad: impl Into<String>,
        action: impl Into<String>,
    ) -> Self {
        self.rows.push(KeyRow::new(keys, gamepad, action));
        self
    }
}

pub fn key_name(key: KeyboardKey) -> String {
    use KeyboardKey::*;
    let special = match key {
        KEY_LEFT_CONTROL | KEY_RIGHT_CONTROL => "Ctrl",
        KEY_LEFT_SHIFT | KEY_RIGHT_SHIFT => "Shift",
        KEY_LEFT_ALT | KEY_RIGHT_ALT => "Alt",
        KEY_ESCAPE => "Esc",
        KEY_ENTER => "Enter",
        KEY_KP_ENTER => "Keypad Enter",
        KEY_PAGE_UP => "Page Up",
        KEY_PAGE_DOWN => "Page Down",
        KEY_BACKSPACE => "Backspace",
        KEY_DELETE => "Delete",
        _ => "",
    };
    if !special.is_empty() {
        return special.to_string();
    }

    let raw = format!("{:?}", key);
    let name = raw.strip_prefix("KEY_").unwrap_or(&raw);
    let function_key =
        name.starts_with('F') && name.len() > 1 && name[1..].chars().all(|c| c.is_ascii_digit());
    if name.chars().count() == 1 || function_key {
        return name.to_string();
    }
    name.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_string() + &chars.as_str().to_lowercase(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn key_list(keys: &[KeyboardKey]) -> String {
    use KeyboardKey::*;
    let arrows = [KEY_UP, KEY_DOWN, KEY_LEFT, KEY_RIGHT];
    let all_arrows = arrows.iter().all(|arrow| keys.contains(arrow));

    let mut names: Vec<String> = Vec::new();
    for &key in keys {
        let name = if all_arrows && arrows.contains(&key) {
            "Arrow keys".to_string()
        } else {
            key_name(key)
        };
        if !names.contains(&name) {
            names.push(name);
        }
    }
    names.join(", ")
}

fn with_mouse(keys: String, mouse: Option<&str>) -> String {
    match (keys.is_empty(), mouse) {
        (_, None) => keys,
        (true, Some(mouse)) => mouse.to_string(),
        (false, Some(mouse)) => format!("{}, {}", keys, mouse),
    }
}

pub fn default_keybinds(
    playing: &PlayingConfig,
    rollback: &RollbackConfig,
    navigation: &NavigationConfig,
) -> Vec<KeySection> {
    let keys = &playing.keys;
    let one = |key: Option<KeyboardKey>| key.map(key_name).unwrap_or_default();
    let pad = |name: &'static str| if navigation.gamepad { name } else { "" };
    let wheel = rollback.mouse_wheel;

    let mut story = KeySection::new("In the story").row(
        with_mouse(key_list(&playing.advance_keys), Some("click")),
        pad("A"),
        "Next line",
    );
    if rollback.enabled {
        story = story
            .row(
                with_mouse(key_list(&rollback.back_keys), wheel.then_some("wheel up")),
                pad("LB"),
                "Roll back",
            )
            .row(
                with_mouse(
                    key_list(&rollback.forward_keys),
                    wheel.then_some("wheel down"),
                ),
                pad("RB"),
                "Roll forward",
            );
    }
    story = story
        .row(
            with_mouse(
                key_list(&keys.hide),
                keys.middle_click_hides.then_some("middle click"),
            ),
            pad("Select"),
            "Hide the text",
        )
        .row(
            key_list(&keys.skip_hold),
            pad("RT (hold)"),
            "Skip while held",
        )
        .row(key_list(&keys.skip_toggle), "", "Skip on / off")
        .row(key_list(&keys.auto), "", "Auto on / off")
        .row(key_list(&keys.log), pad("Y"), "Log")
        .row(
            with_mouse(
                one(playing.pause_key),
                keys.right_click_pauses.then_some("right click"),
            ),
            pad("Start"),
            "Pause menu",
        )
        .row(one(playing.quick_save_key), "", "Quick save")
        .row(one(playing.quick_load_key), "", "Quick load")
        .row(key_list(&keys.screenshot), "", "Screenshot")
        .row(key_list(&keys.fullscreen), "", "Fullscreen on / off");
    story
        .rows
        .retain(|row| !row.keys.is_empty() || !row.gamepad.is_empty());

    let mut arrows: Vec<KeyboardKey> = [
        &navigation.up_keys,
        &navigation.down_keys,
        &navigation.left_keys,
        &navigation.right_keys,
    ]
    .into_iter()
    .flatten()
    .copied()
    .collect();
    arrows.push(KeyboardKey::KEY_TAB);
    let mut back = vec![KeyboardKey::KEY_ESCAPE, KeyboardKey::KEY_BACKSPACE];
    back.extend(navigation.back_keys.iter().copied());
    let mut menus = KeySection::new("Menus")
        .row(
            key_list(&arrows),
            pad("D-pad, left stick"),
            "Move between buttons",
        )
        .row(
            key_list(&navigation.accept_keys),
            pad("A"),
            "Press the button",
        )
        .row(key_list(&back), pad("B"), "Back")
        .row(key_list(&navigation.alt_keys), pad("X"), "Delete a save");
    if !navigation.enabled {
        menus.rows.clear();
    }

    let mut sections = vec![story];
    if !menus.rows.is_empty() {
        sections.push(menus);
    }
    sections
}

#[derive(Clone, Debug)]
pub struct KeybindsConfig {
    pub title: String,
    pub title_text: TextStyle,
    pub section_text: TextStyle,
    pub key_text: TextStyle,
    pub action_text: TextStyle,
    pub header_text: TextStyle,
    pub keys_header: String,
    pub gamepad_header: String,
    pub open_keys: Vec<KeyboardKey>,
    pub close_keys: Vec<KeyboardKey>,
    pub panel_width: f32,
    pub panel: PanelStyle,
    pub backdrop: Color,
    pub back_button: ButtonStyle,
    pub back_label: String,
    pub sections: Option<Vec<KeySection>>,
    pub extra: Vec<KeySection>,
}

impl Default for KeybindsConfig {
    fn default() -> Self {
        Self {
            title: "Controls".to_string(),
            title_text: TextStyle::new(FontRole::Title, 38.0, Color::RAYWHITE),
            section_text: TextStyle::new(FontRole::Menu, 20.0, Color::GOLD),
            key_text: TextStyle::new(FontRole::Menu, 16.0, Color::RAYWHITE),
            action_text: TextStyle::new(FontRole::Menu, 16.0, Color::LIGHTGRAY),
            header_text: TextStyle::new(FontRole::Menu, 13.0, Color::GRAY),
            keys_header: "KEYBOARD / MOUSE".to_string(),
            gamepad_header: "GAMEPAD".to_string(),
            open_keys: vec![KeyboardKey::KEY_F1],
            close_keys: vec![
                KeyboardKey::KEY_F1,
                KeyboardKey::KEY_ESCAPE,
                KeyboardKey::KEY_BACKSPACE,
            ],
            panel_width: 1160.0,
            panel: PanelStyle::new(Color::new(14, 14, 22, 240)).roundness(0.03),
            backdrop: Color::new(0, 0, 0, 170),
            back_button: ButtonStyle::default().size(200.0, 44.0),
            back_label: "Back".to_string(),
            sections: None,
            extra: Vec::new(),
        }
    }
}

impl KeybindsConfig {
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn title_text(mut self, style: TextStyle) -> Self {
        self.title_text = style;
        self
    }

    pub fn section_text(mut self, style: TextStyle) -> Self {
        self.section_text = style;
        self
    }

    pub fn key_text(mut self, style: TextStyle) -> Self {
        self.key_text = style;
        self
    }

    pub fn action_text(mut self, style: TextStyle) -> Self {
        self.action_text = style;
        self
    }

    pub fn header_text(mut self, style: TextStyle) -> Self {
        self.header_text = style;
        self
    }

    pub fn open_keys(mut self, keys: impl IntoIterator<Item = KeyboardKey>) -> Self {
        self.open_keys = keys.into_iter().collect();
        self
    }

    pub fn close_keys(mut self, keys: impl IntoIterator<Item = KeyboardKey>) -> Self {
        self.close_keys = keys.into_iter().collect();
        self
    }

    pub fn panel_color(mut self, color: Color) -> Self {
        self.panel.color = color;
        self
    }

    pub fn panel(mut self, style: impl FnOnce(PanelStyle) -> PanelStyle) -> Self {
        self.panel = style(self.panel);
        self
    }

    pub fn backdrop(mut self, color: Color) -> Self {
        self.backdrop = color;
        self
    }

    pub fn back_button(mut self, style: impl FnOnce(ButtonStyle) -> ButtonStyle) -> Self {
        self.back_button = style(self.back_button);
        self
    }

    pub fn back_label(mut self, text: impl Into<String>) -> Self {
        self.back_label = text.into();
        self
    }

    pub fn sections(mut self, sections: impl IntoIterator<Item = KeySection>) -> Self {
        self.sections = Some(sections.into_iter().collect());
        self
    }

    pub fn section(mut self, section: KeySection) -> Self {
        self.extra.push(section);
        self
    }
}

pub struct KeybindsOverlay {
    config: Rc<KeybindsConfig>,
    sections: Vec<KeySection>,
    focus: Focus,
}

impl KeybindsOverlay {
    pub fn new(config: Rc<KeybindsConfig>, defaults: Vec<KeySection>) -> Self {
        let mut sections = config.sections.clone().unwrap_or(defaults);
        let anywhere =
            KeySection::new("Anywhere").row(key_list(&config.open_keys), "", "These controls");
        sections.extend(config.extra.iter().cloned());
        if !config.open_keys.is_empty() {
            sections.push(anywhere);
        }
        Self {
            config,
            sections,
            focus: Focus::default(),
        }
    }

    pub fn sections(&self) -> &[KeySection] {
        &self.sections
    }

    fn panel_rect(&self, screen: Vector2) -> Rectangle {
        let width = self.config.panel_width.min(screen.x - 32.0);
        Rectangle::new((screen.x - width) / 2.0, 16.0, width, screen.y - 32.0)
    }

    fn back_rect(&self, screen: Vector2) -> Rectangle {
        let panel = self.panel_rect(screen);
        let style = &self.config.back_button;
        Rectangle::new(
            (screen.x - style.width) / 2.0,
            panel.y + panel.height - style.height - 16.0,
            style.width,
            style.height,
        )
    }
}

impl Overlay for KeybindsOverlay {
    fn update(&mut self, mut ctx: GameContext) -> OverlayAction {
        let config = Rc::clone(&self.config);
        let back = self.back_rect(ui::screen_size(ctx.rl));
        let key = config.close_keys.iter().any(|&k| ctx.rl.is_key_pressed(k));
        let right_click = ctx
            .rl
            .is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_RIGHT);
        let accepted = self.focus.update(&ctx.nav, &[back], &[], None);

        if ui::button_clicked(&mut ctx, back, &config.back_button)
            || key
            || right_click
            || ctx.nav.back
            || accepted.is_some()
        {
            OverlayAction::Close
        } else {
            OverlayAction::Stay
        }
    }

    fn draw(&self, d: &mut RaylibDrawHandle, ctx: &DrawContext) {
        let config = &self.config;
        let fonts = ctx.fonts();
        let screen = ui::screen_size(d);
        let panel = self.panel_rect(screen);

        d.draw_rectangle(0, 0, screen.x as i32, screen.y as i32, config.backdrop);
        config.panel.draw(d, panel);
        ui::draw_text_centered(
            d,
            fonts,
            ctx.label(&config.title),
            Vector2::new(
                screen.x / 2.0,
                panel.y + 18.0 + config.title_text.size / 2.0,
            ),
            &config.title_text,
        );

        let top = panel.y + config.title_text.size + 40.0;
        let gap = 36.0;
        let column_width = (panel.width - 64.0 - gap) / 2.0;
        let keys_width = column_width * 0.4;
        let pad_width = column_width * 0.22;
        let row_height = config.key_text.size * 1.55;

        let (first, rest) = self.sections.split_at(self.sections.len().min(1));
        let columns = [first, rest];
        for (column, sections) in columns.iter().enumerate() {
            let x = panel.x + 32.0 + column as f32 * (column_width + gap);
            let mut y = top;
            for section in sections.iter() {
                ui::draw_text(
                    d,
                    fonts,
                    ctx.label(&section.title),
                    Vector2::new(x, y),
                    &config.section_text,
                );
                y += config.section_text.size * 1.4;
                ui::draw_text(
                    d,
                    fonts,
                    &config.keys_header,
                    Vector2::new(x, y),
                    &config.header_text,
                );
                ui::draw_text(
                    d,
                    fonts,
                    &config.gamepad_header,
                    Vector2::new(x + keys_width, y),
                    &config.header_text,
                );
                y += config.header_text.size * 1.6;

                for row in &section.rows {
                    let keys = ui::fit_text(fonts, &config.key_text, &row.keys, keys_width - 12.0);
                    let gamepad =
                        ui::fit_text(fonts, &config.key_text, &row.gamepad, pad_width - 12.0);
                    let action = ui::fit_text(
                        fonts,
                        &config.action_text,
                        ctx.label(&row.action),
                        column_width - keys_width - pad_width,
                    );
                    ui::draw_text(d, fonts, &keys, Vector2::new(x, y), &config.key_text);
                    ui::draw_text(
                        d,
                        fonts,
                        &gamepad,
                        Vector2::new(x + keys_width, y),
                        &config.key_text,
                    );
                    ui::draw_text(
                        d,
                        fonts,
                        &action,
                        Vector2::new(x + keys_width + pad_width, y),
                        &config.action_text,
                    );
                    y += row_height;
                }
                y += 18.0;
            }
        }

        ui::Button::new(ctx.label(&config.back_label), &config.back_button)
            .focused(ctx.shows_focus(&self.focus, 0))
            .draw(d, ctx, self.back_rect(screen));
    }
}
