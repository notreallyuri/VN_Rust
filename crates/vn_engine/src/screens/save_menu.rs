use std::collections::HashMap;
use std::rc::Rc;

use raylib::prelude::*;

use crate::saves::{AUTO_SLOT, QUICK_SLOT, SaveError, SlotInfo, now, time_ago};
use crate::screens::Confirm;
use crate::ui::{self, Background, ButtonStyle, TextStyle};
use crate::{
    Action, Anchor, DrawContext, Focus, FontRole, GameContext, Layout, Overlay, OverlayAction,
    Screen, ScreenState,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SaveMenuMode {
    Save,
    Load,
}

#[derive(Clone, Debug)]
pub struct SaveMenuConfig {
    pub slots: usize,
    pub save_title: String,
    pub load_title: String,
    pub title_text: TextStyle,
    pub slot_width: f32,
    pub slot_height: f32,
    pub slot_layout: Layout,
    pub slot_color: Color,
    pub slot_hover_color: Color,
    pub slot_title_text: TextStyle,
    pub slot_summary_text: TextStyle,
    pub error_text: TextStyle,
    pub empty_label: String,
    pub back_button: ButtonStyle,
    pub back_label: String,
    pub back_keys: Vec<KeyboardKey>,
    pub confirm_overwrite: bool,
    pub confirm_load_in_game: bool,
    pub thumbnails: bool,
    pub thumbnail_color: Color,
    pub allow_delete: bool,
    pub confirm_delete: bool,
    pub delete_button: ButtonStyle,
    pub delete_label: String,
    pub delete_tooltip: Option<String>,
    pub delete_keys: Vec<KeyboardKey>,
    pub background: Option<Background>,
    pub backdrop: Color,
}

impl Default for SaveMenuConfig {
    fn default() -> Self {
        Self {
            slots: 6,
            save_title: "Save Game".to_string(),
            load_title: "Load Game".to_string(),
            title_text: TextStyle::new(FontRole::Title, 44.0, Color::RAYWHITE),
            slot_width: 760.0,
            slot_height: 64.0,
            slot_layout: Layout::default().anchor(Anchor::Top).spacing(10.0),
            slot_color: Color::new(30, 30, 45, 230),
            slot_hover_color: Color::new(50, 50, 72, 240),
            slot_title_text: TextStyle::new(FontRole::Menu, 20.0, Color::RAYWHITE),
            slot_summary_text: TextStyle::new(FontRole::Dialogue, 17.0, Color::LIGHTGRAY),
            error_text: TextStyle::new(FontRole::Menu, 17.0, Color::new(230, 110, 110, 255)),
            empty_label: "Empty".to_string(),
            back_button: ButtonStyle::default().size(200.0, 46.0),
            back_label: "Back".to_string(),
            back_keys: vec![KeyboardKey::KEY_ESCAPE, KeyboardKey::KEY_BACKSPACE],
            confirm_overwrite: true,
            confirm_load_in_game: true,
            thumbnails: true,
            thumbnail_color: Color::new(10, 10, 16, 255),
            allow_delete: true,
            confirm_delete: true,
            delete_button: ButtonStyle::default()
                .size(76.0, 26.0)
                .color(Color::new(90, 40, 40, 230))
                .font_size(15.0),
            delete_label: "Delete".to_string(),
            delete_tooltip: Some("Delete this save for good (Delete key)".to_string()),
            delete_keys: vec![KeyboardKey::KEY_DELETE],
            background: None,
            backdrop: Color::new(8, 8, 14, 235),
        }
    }
}

impl SaveMenuConfig {
    pub fn slots(mut self, count: usize) -> Self {
        self.slots = count.max(1);
        self
    }

    pub fn save_title(mut self, text: impl Into<String>) -> Self {
        self.save_title = text.into();
        self
    }

    pub fn load_title(mut self, text: impl Into<String>) -> Self {
        self.load_title = text.into();
        self
    }

    pub fn title_text(mut self, style: TextStyle) -> Self {
        self.title_text = style;
        self
    }

    pub fn slot_size(mut self, width: f32, height: f32) -> Self {
        self.slot_width = width;
        self.slot_height = height;
        self
    }

    pub fn slot_spacing(mut self, spacing: f32) -> Self {
        self.slot_layout = self.slot_layout.spacing(spacing);
        self
    }

    pub fn slot_layout(mut self, layout: impl FnOnce(Layout) -> Layout) -> Self {
        self.slot_layout = layout(self.slot_layout);
        self
    }

    pub fn slot_color(mut self, color: Color) -> Self {
        self.slot_color = color;
        self.slot_hover_color = ui::lighten(color);
        self
    }

    pub fn slot_title_text(mut self, style: TextStyle) -> Self {
        self.slot_title_text = style;
        self
    }

    pub fn slot_summary_text(mut self, style: TextStyle) -> Self {
        self.slot_summary_text = style;
        self
    }

    pub fn error_text(mut self, style: TextStyle) -> Self {
        self.error_text = style;
        self
    }

    pub fn empty_label(mut self, text: impl Into<String>) -> Self {
        self.empty_label = text.into();
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

    pub fn back_keys(mut self, keys: impl IntoIterator<Item = KeyboardKey>) -> Self {
        self.back_keys = keys.into_iter().collect();
        self
    }

    pub fn confirm_overwrite(mut self, confirm: bool) -> Self {
        self.confirm_overwrite = confirm;
        self
    }

    pub fn confirm_load_in_game(mut self, confirm: bool) -> Self {
        self.confirm_load_in_game = confirm;
        self
    }

    pub fn thumbnails(mut self, show: bool) -> Self {
        self.thumbnails = show;
        self
    }

    pub fn thumbnail_color(mut self, color: Color) -> Self {
        self.thumbnail_color = color;
        self
    }

    pub fn allow_delete(mut self, allow: bool) -> Self {
        self.allow_delete = allow;
        self
    }

    pub fn confirm_delete(mut self, confirm: bool) -> Self {
        self.confirm_delete = confirm;
        self
    }

    pub fn delete_button(mut self, style: impl FnOnce(ButtonStyle) -> ButtonStyle) -> Self {
        self.delete_button = style(self.delete_button);
        self
    }

    pub fn delete_label(mut self, text: impl Into<String>) -> Self {
        self.delete_label = text.into();
        self
    }

    pub fn delete_tooltip(mut self, text: Option<&str>) -> Self {
        self.delete_tooltip = text.map(str::to_string);
        self
    }

    pub fn delete_keys(mut self, keys: impl IntoIterator<Item = KeyboardKey>) -> Self {
        self.delete_keys = keys.into_iter().collect();
        self
    }

    pub fn backdrop(mut self, color: Color) -> Self {
        self.backdrop = color;
        self
    }

    fn thumbnail_rect(&self, slot: Rectangle) -> Option<Rectangle> {
        if !self.thumbnails {
            return None;
        }
        let height = slot.height - 12.0;
        Some(Rectangle::new(
            slot.x + 6.0,
            slot.y + 6.0,
            height * 16.0 / 9.0,
            height,
        ))
    }

    fn delete_rect(&self, slot: Rectangle) -> Rectangle {
        let style = &self.delete_button;
        Rectangle::new(
            slot.x + slot.width - style.width - 8.0,
            slot.y + 8.0,
            style.width,
            style.height,
        )
    }

    pub fn background(mut self, background: Background) -> Self {
        self.background = Some(background);
        self
    }
}

enum Outcome {
    Stay,
    Back,
    Loaded,
}

struct SaveMenu {
    config: Rc<SaveMenuConfig>,
    mode: SaveMenuMode,
    in_game: bool,
    slots: Option<Vec<SlotInfo>>,
    thumbnails: HashMap<String, Texture2D>,
    seen_generation: u64,
    focus: Focus,
}

impl SaveMenu {
    fn new(config: Rc<SaveMenuConfig>, mode: SaveMenuMode, in_game: bool) -> Self {
        Self {
            config,
            mode,
            in_game,
            slots: None,
            thumbnails: HashMap::new(),
            seen_generation: 0,
            focus: Focus::default(),
        }
    }

    fn slot_ids(&self, ctx: &GameContext) -> Vec<String> {
        let mut ids: Vec<String> = (1..=self.config.slots).map(|n| n.to_string()).collect();

        if self.mode == SaveMenuMode::Load {
            let exists = |slot| ctx.saves.path(slot).is_ok_and(|path| path.exists());
            for slot in [QUICK_SLOT, AUTO_SLOT] {
                if exists(slot) {
                    ids.insert(0, slot.to_string());
                }
            }
        }

        ids
    }

    fn refresh(&mut self, ctx: &mut GameContext) {
        let slots: Vec<SlotInfo> = self
            .slot_ids(ctx)
            .iter()
            .map(|id| ctx.saves.slot(id))
            .collect();

        self.thumbnails.clear();
        if self.config.thumbnails {
            for info in slots.iter().filter(|info| info.save.is_ok()) {
                let Ok(path) = ctx.saves.thumbnail_path(&info.slot) else {
                    continue;
                };
                if path.exists()
                    && let Ok(texture) = ctx.rl.load_texture(ctx.thread, &path.to_string_lossy())
                {
                    texture.set_texture_filter(ctx.thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
                    self.thumbnails.insert(info.slot.clone(), texture);
                }
            }
        }

        self.slots = Some(slots);
        self.seen_generation = ctx.saves.generation();
    }

    fn delete(&self, ctx: &mut GameContext, index: usize) {
        let Some(info) = self.slots.as_ref().and_then(|slots| slots.get(index)) else {
            return;
        };
        if info.is_empty() {
            return;
        }

        let slot = info.slot.clone();
        let label = slot_label(&slot);
        if self.config.confirm_delete {
            ctx.confirm(
                Confirm::new(
                    format!("Delete {}? This can't be undone.", label),
                    Action::custom(move |ctx| {
                        delete_slot(ctx, &slot);
                        None
                    }),
                )
                .confirm_label("Delete"),
            );
        } else {
            delete_slot(ctx, &slot);
        }
    }

    fn slot_rects(&self, screen: Vector2) -> Vec<Rectangle> {
        const TOP: f32 = 130.0;
        const MESSAGE_SPACE: f32 = 24.0;

        let count = self.slots.as_ref().map_or(0, Vec::len);
        let config = &self.config;
        let layout = &config.slot_layout;

        let bottom = self.back_rect(screen).y - MESSAGE_SPACE;
        let area = Rectangle::new(0.0, TOP, screen.x, (bottom - TOP).max(0.0));
        let rows = layout.rows(count).max(1);
        let gaps = (rows - 1) as f32 * layout.spacing.y;
        let fitting = (area.height - gaps) / rows as f32;
        let size = Vector2::new(config.slot_width, config.slot_height.min(fitting));

        layout.place(area, &vec![size; count])
    }

    fn back_rect(&self, screen: Vector2) -> Rectangle {
        let style = &self.config.back_button;
        Rectangle::new(
            (screen.x - style.width) / 2.0,
            screen.y - style.height - 40.0,
            style.width,
            style.height,
        )
    }

    fn pick(&mut self, ctx: &mut GameContext, index: usize) -> Outcome {
        let Some(info) = self.slots.as_ref().and_then(|slots| slots.get(index)) else {
            return Outcome::Stay;
        };
        let slot = info.slot.clone();
        let label = slot_label(&slot);
        let occupied = info.save.is_ok();

        match self.mode {
            SaveMenuMode::Save if occupied && self.config.confirm_overwrite => {
                ctx.confirm(
                    Confirm::new(
                        format!("Overwrite {}?", label),
                        Action::custom(move |ctx| {
                            save_to(ctx, &slot);
                            None
                        }),
                    )
                    .confirm_label("Overwrite"),
                );
                Outcome::Stay
            }
            SaveMenuMode::Save => {
                save_to(ctx, &slot);
                Outcome::Stay
            }
            SaveMenuMode::Load if !occupied => {
                if let Err(e) = &info.save {
                    ctx.notify_error(e.player_message());
                }
                Outcome::Stay
            }
            SaveMenuMode::Load if self.in_game && self.config.confirm_load_in_game => {
                ctx.confirm(
                    Confirm::new(
                        format!("Load {}? Unsaved progress will be lost.", label),
                        Action::custom(move |ctx| {
                            load_from(ctx, &slot).then_some(ScreenState::Playing)
                        }),
                    )
                    .confirm_label("Load"),
                );
                Outcome::Stay
            }
            SaveMenuMode::Load => {
                if load_from(ctx, &slot) {
                    Outcome::Loaded
                } else {
                    Outcome::Stay
                }
            }
        }
    }
}

fn save_to(ctx: &mut GameContext, slot: &str) {
    match ctx.save(slot) {
        Ok(()) => ctx.notify(format!("Saved to {}", slot_label(slot))),
        Err(e) => {
            eprintln!("⚠️ Save failed ({}): {}", slot, e);
            ctx.notify_error(format!("Save failed: {}", e.player_message()));
        }
    }
}

fn load_from(ctx: &mut GameContext, slot: &str) -> bool {
    match ctx.load(slot) {
        Ok(_) => {
            ctx.notify(format!("Loaded {}", slot_label(slot)));
            true
        }
        Err(e) => {
            eprintln!("⚠️ Load failed ({}): {}", slot, e);
            ctx.notify_error(format!("Load failed: {}", e.player_message()));
            false
        }
    }
}

fn delete_slot(ctx: &mut GameContext, slot: &str) {
    match ctx.saves.delete(slot) {
        Ok(()) => ctx.notify(format!("Deleted {}", slot_label(slot))),
        Err(e) => {
            eprintln!("⚠️ Delete failed ({}): {}", slot, e);
            ctx.notify_error(format!("Delete failed: {}", e.player_message()));
        }
    }
}

fn slot_label(slot: &str) -> String {
    match slot {
        QUICK_SLOT => "Quick save".to_string(),
        AUTO_SLOT => "Autosave".to_string(),
        _ => format!("Slot {}", slot),
    }
}

impl SaveMenu {
    fn occupied(&self, index: usize) -> bool {
        self.slots
            .as_ref()
            .and_then(|slots| slots.get(index))
            .is_some_and(|info| !info.is_empty())
    }

    fn update(&mut self, ctx: &mut GameContext) -> Outcome {
        if self.slots.is_none() || self.seen_generation != ctx.saves.generation() {
            self.refresh(ctx);
        }

        let screen = ui::screen_size(ctx.rl);
        let back_key = self
            .config
            .back_keys
            .iter()
            .any(|&key| ctx.rl.is_key_pressed(key));
        let config = Rc::clone(&self.config);
        let back = self.back_rect(screen);
        let back_clicked = ui::button_clicked(ctx, back, &config.back_button);
        if back_key || back_clicked || ctx.nav.back {
            return Outcome::Back;
        }

        let rects = self.slot_rects(screen);
        let hovered = rects.iter().position(|rect| ui::is_hovered(ctx.rl, *rect));

        let mut targets = rects.clone();
        targets.push(back);
        let pointed = hovered.or_else(|| {
            ui::button_hovered(ctx.rl, back, &config.back_button).then_some(rects.len())
        });
        let accepted = self.focus.update(&ctx.nav, &targets, &[], pointed);
        match accepted {
            Some(index) if index == rects.len() => return Outcome::Back,
            Some(index) => return self.pick(ctx, index),
            None => {}
        }

        if config.allow_delete
            && ctx.nav.alt
            && let Some(index) = self.focus.index().filter(|&i| self.occupied(i))
        {
            self.delete(ctx, index);
            return Outcome::Stay;
        }

        if self.config.allow_delete
            && let Some(index) = hovered
            && self.occupied(index)
        {
            let delete = self.config.delete_rect(rects[index]);
            if let Some(text) = &self.config.delete_tooltip {
                ctx.tooltip(delete, text.clone());
            }
            let key = self
                .config
                .delete_keys
                .iter()
                .any(|&key| ctx.rl.is_key_pressed(key));
            if ui::button_clicked(ctx, delete, &config.delete_button) || key {
                self.delete(ctx, index);
                return Outcome::Stay;
            }
        }

        match rects.iter().position(|rect| ui::is_clicked(ctx.rl, *rect)) {
            Some(index) => self.pick(ctx, index),
            None => Outcome::Stay,
        }
    }

    fn draw(&self, d: &mut RaylibDrawHandle, ctx: &DrawContext) {
        let config = &self.config;
        let screen = ui::screen_size(d);
        let fonts = ctx.fonts();
        let now = now();

        let title = match self.mode {
            SaveMenuMode::Save => &config.save_title,
            SaveMenuMode::Load => &config.load_title,
        };
        ui::draw_text_centered(
            d,
            fonts,
            title,
            Vector2::new(screen.x / 2.0, 70.0),
            &config.title_text,
        );

        let slots = self.slots.as_deref().unwrap_or_default();
        for (index, (info, rect)) in slots.iter().zip(self.slot_rects(screen)).enumerate() {
            let hovered = ctx.pointer_over(d, rect);
            let color = if hovered || ctx.shows_focus(&self.focus, index) {
                config.slot_hover_color
            } else {
                config.slot_color
            };
            d.draw_rectangle_rec(rect, color);

            let mut x = rect.x + 18.0;
            if let Some(frame) = config.thumbnail_rect(rect) {
                d.draw_rectangle_rec(frame, config.thumbnail_color);
                if let Some(texture) = self.thumbnails.get(&info.slot) {
                    ui::draw_texture_cover(d, texture, frame);
                }
                x = frame.x + frame.width + 14.0;
            }

            let focused = ctx.shows_focus(&self.focus, index);
            let deletable = config.allow_delete && (hovered || focused) && self.occupied(index);
            let delete = config.delete_rect(rect);
            let right = if deletable {
                delete.x - 10.0
            } else {
                rect.x + rect.width - 12.0
            };

            let (heading, detail, detail_style) = match &info.save {
                Ok(file) => (
                    format!(
                        "{}  ·  {}",
                        slot_label(&info.slot),
                        time_ago(file.saved_at, now)
                    ),
                    file.summary.clone(),
                    &config.slot_summary_text,
                ),
                Err(SaveError::Empty { .. }) => (
                    slot_label(&info.slot),
                    config.empty_label.clone(),
                    &config.slot_summary_text,
                ),
                Err(e) => (
                    slot_label(&info.slot),
                    e.player_message(),
                    &config.error_text,
                ),
            };

            let width = (right - x).max(0.0);
            let heading = ui::fit_text(fonts, &config.slot_title_text, &heading, width);
            let lines = fonts.wrap(detail_style.font, &detail, detail_style.size, width);
            let title_bottom = rect.y + 12.0 + config.slot_title_text.size;
            let line_height = detail_style.size * 1.25;
            let room =
                (((rect.y + rect.height - 6.0 - title_bottom) / line_height) as usize).max(1);

            ui::draw_text(
                d,
                fonts,
                &heading,
                Vector2::new(x, rect.y + 10.0),
                &config.slot_title_text,
            );
            for (i, line) in lines.iter().take(room).enumerate() {
                let line = if i + 1 == room && lines.len() > room {
                    ui::fit_text(fonts, detail_style, &format!("{}…", line), width)
                } else {
                    line.clone()
                };
                ui::draw_text(
                    d,
                    fonts,
                    &line,
                    Vector2::new(x, title_bottom + i as f32 * line_height),
                    detail_style,
                );
            }

            if deletable {
                ui::draw_button(d, ctx, delete, &config.delete_label, &config.delete_button);
            }
        }

        let back_index = slots.len();
        ui::Button::new(&config.back_label, &config.back_button)
            .focused(ctx.shows_focus(&self.focus, back_index))
            .draw(d, ctx, self.back_rect(screen));
    }
}

pub struct SaveMenuScreen {
    menu: SaveMenu,
}

impl SaveMenuScreen {
    pub fn new(config: Rc<SaveMenuConfig>, mode: SaveMenuMode) -> Self {
        Self {
            menu: SaveMenu::new(config, mode, false),
        }
    }
}

impl Screen for SaveMenuScreen {
    fn update(&mut self, mut ctx: GameContext) -> Option<ScreenState> {
        ui::load_background(&mut ctx, self.menu.config.background.as_ref());

        match self.menu.update(&mut ctx) {
            Outcome::Stay => None,
            Outcome::Loaded => Some(ScreenState::Playing),
            Outcome::Back => Some(match ctx.previous {
                Some(state) if !matches!(state, ScreenState::Save | ScreenState::Load) => {
                    state.clone()
                }
                _ => ScreenState::MainMenu,
            }),
        }
    }

    fn draw(&self, d: &mut RaylibDrawHandle, ctx: &DrawContext) {
        ui::draw_background(d, ctx.resources, self.menu.config.background.as_ref());
        self.menu.draw(d, ctx);
    }
}

pub struct SaveMenuOverlay {
    menu: SaveMenu,
}

impl SaveMenuOverlay {
    pub fn new(config: Rc<SaveMenuConfig>, mode: SaveMenuMode) -> Self {
        Self {
            menu: SaveMenu::new(config, mode, true),
        }
    }
}

impl Overlay for SaveMenuOverlay {
    fn update(&mut self, mut ctx: GameContext) -> OverlayAction {
        match self.menu.update(&mut ctx) {
            Outcome::Stay => OverlayAction::Stay,
            Outcome::Back => OverlayAction::Close,
            Outcome::Loaded => OverlayAction::Goto(ScreenState::Playing),
        }
    }

    fn draw(&self, d: &mut RaylibDrawHandle, ctx: &DrawContext) {
        let screen = ui::screen_size(d);
        d.draw_rectangle(
            0,
            0,
            screen.x as i32,
            screen.y as i32,
            self.menu.config.backdrop,
        );
        self.menu.draw(d, ctx);
    }
}
