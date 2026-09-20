use std::collections::HashMap;
use std::rc::Rc;

use raylib::prelude::*;

use super::actions::{delete_slot, load_from, save_to, slot_label};
use super::{SaveMenuConfig, SaveMenuMode};
use crate::data::saves::SaveError;
use crate::data::saves::{AUTO_SLOT, QUICK_SLOT, SlotInfo, now, time_ago};
use crate::screens::Confirm;
use crate::ui;
use crate::{Action, DrawContext, Focus, GameContext, ScreenState};

pub(super) enum Outcome {
    Stay,
    Back,
    Loaded,
}

pub(super) struct SaveMenu {
    pub(super) config: Rc<SaveMenuConfig>,
    mode: SaveMenuMode,
    in_game: bool,
    slots: Option<Vec<SlotInfo>>,
    thumbnails: HashMap<String, Texture2D>,
    seen_generation: u64,
    focus: Focus,
}

impl SaveMenu {
    pub(super) fn new(config: Rc<SaveMenuConfig>, mode: SaveMenuMode, in_game: bool) -> Self {
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

    fn panel_rect(&self, screen: Vector2) -> Rectangle {
        let slots = self.slot_rects(screen);
        let half = self.config.slot_width / 2.0;
        let left = slots
            .iter()
            .map(|r| r.x)
            .fold(screen.x / 2.0 - half, f32::min);
        let right = slots
            .iter()
            .map(|r| r.x + r.width)
            .fold(screen.x / 2.0 + half, f32::max);
        let padding = self.config.panel_padding;
        Rectangle::new(
            left - padding,
            24.0,
            right - left + padding * 2.0,
            screen.y - 48.0,
        )
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

impl SaveMenu {
    fn occupied(&self, index: usize) -> bool {
        self.slots
            .as_ref()
            .and_then(|slots| slots.get(index))
            .is_some_and(|info| !info.is_empty())
    }

    pub(super) fn update(&mut self, ctx: &mut GameContext) -> Outcome {
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

    pub(super) fn draw(&self, d: &mut RaylibDrawHandle, ctx: &DrawContext) {
        let config = &self.config;
        let screen = ui::screen_size(d);
        let fonts = ctx.fonts();
        let now = now();

        if let Some(panel) = &config.panel {
            panel.draw(d, self.panel_rect(screen));
        }

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
            if hovered || ctx.shows_focus(&self.focus, index) {
                config.slot_hover_panel().draw(d, rect);
            } else {
                config.slot_panel.draw(d, rect);
            }

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
