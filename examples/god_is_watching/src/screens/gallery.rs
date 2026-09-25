use novn::prelude::*;
use novn::raylib::prelude::*;
use novn::ui;
use novn::ui::button::ButtonStyle;

use crate::gallery::{TABS, Tab, found};
use crate::style;

const COLUMNS: usize = 4;
const CELL: Vector2 = Vector2::new(240.0, 150.0);
const GAP: f32 = 16.0;
const TRACK_ROW: Vector2 = Vector2::new(520.0, 46.0);

pub struct GalleryScreen {
    tab: usize,
    heading: TextStyle,
    caption: TextStyle,
    count: TextStyle,
    locked: TextStyle,
    tab_button: ButtonStyle,
    track: ButtonStyle,
    back: ButtonStyle,
    frame: PanelStyle,
    cell: PanelStyle,
}

impl GalleryScreen {
    pub fn new() -> Self {
        Self {
            tab: 0,
            heading: style::heading(40.0),
            caption: style::label(16.0).color(style::TEXT),
            count: style::section(16.0),
            locked: style::section(22.0),
            tab_button: style::menu_link(ButtonStyle::default().size(150.0, 34.0)),
            track: style::button(ButtonStyle::default().size(TRACK_ROW.x, TRACK_ROW.y)),
            back: style::button(ButtonStyle::default().size(200.0, 44.0)),
            frame: style::frame(PanelStyle::default()),
            cell: style::inset(PanelStyle::default()),
        }
    }

    fn tab_rects(&self, screen: Vector2) -> Vec<Rectangle> {
        let width = self.tab_button.width;
        let total = width * TABS.len() as f32 + GAP * (TABS.len() - 1) as f32;
        (0..TABS.len())
            .map(|index| {
                Rectangle::new(
                    (screen.x - total) / 2.0 + index as f32 * (width + GAP),
                    104.0,
                    width,
                    self.tab_button.height,
                )
            })
            .collect()
    }

    fn cell_rects(&self, tab: &Tab, screen: Vector2) -> Vec<Rectangle> {
        let columns = COLUMNS.min(tab.entries.len().max(1));
        let total = CELL.x * columns as f32 + GAP * (columns - 1) as f32;
        tab.entries
            .iter()
            .enumerate()
            .map(|(index, _)| {
                let (row, column) = (index / columns, index % columns);
                Rectangle::new(
                    (screen.x - total) / 2.0 + column as f32 * (CELL.x + GAP),
                    168.0 + row as f32 * (CELL.y + 34.0),
                    CELL.x,
                    CELL.y,
                )
            })
            .collect()
    }

    fn track_rects(&self, tab: &Tab, screen: Vector2) -> Vec<Rectangle> {
        tab.entries
            .iter()
            .enumerate()
            .map(|(index, _)| {
                Rectangle::new(
                    (screen.x - TRACK_ROW.x) / 2.0,
                    172.0 + index as f32 * (TRACK_ROW.y + 12.0),
                    TRACK_ROW.x,
                    TRACK_ROW.y,
                )
            })
            .collect()
    }

    fn back_rect(&self, screen: Vector2) -> Rectangle {
        Rectangle::new(
            (screen.x - self.back.width) / 2.0,
            screen.y - self.back.height - 32.0,
            self.back.width,
            self.back.height,
        )
    }
}

fn source(picture: &Texture2D, crop: Option<[f32; 4]>) -> Rectangle {
    let (width, height) = (picture.width as f32, picture.height as f32);
    match crop {
        Some([x, y, w, h]) => Rectangle::new(x * width, y * height, w * width, h * height),
        None => Rectangle::new(0.0, 0.0, width, height),
    }
}

fn fit(source: Rectangle, cell: Rectangle) -> Rectangle {
    let scale = (cell.width / source.width).min(cell.height / source.height);
    let (width, height) = (source.width * scale, source.height * scale);
    Rectangle::new(
        cell.x + (cell.width - width) / 2.0,
        cell.y + (cell.height - height) / 2.0,
        width,
        height,
    )
}

impl Screen for GalleryScreen {
    fn update(&mut self, mut ctx: GameContext) -> Option<ScreenState> {
        ui::load_background(
            &mut ctx,
            Some(&style::background(style::ARCHIVE_BACKGROUND)),
        );
        let screen = ui::screen_size(ctx.rl);

        for (index, rect) in self.tab_rects(screen).into_iter().enumerate() {
            if ui::button::button_clicked(&mut ctx, rect, &self.tab_button) {
                self.tab = index;
            }
        }

        let tab = &TABS[self.tab];
        if tab.entries.iter().all(|entry| entry.track().is_none()) {
            for (entry, rect) in tab.entries.iter().zip(self.cell_rects(tab, screen)) {
                if ctx.has_seen(&entry.key()) {
                    ctx.tooltip(rect, entry.title.to_string());
                }
            }
        } else {
            for (entry, rect) in tab.entries.iter().zip(self.track_rects(tab, screen)) {
                let playable = ctx.has_seen(&entry.key());
                if playable && ui::button::button_clicked(&mut ctx, rect, &self.track) {
                    ctx.play_music(entry.track());
                }
            }
        }

        let back = self.back_rect(screen);
        let key = [KeyboardKey::KEY_ESCAPE, KeyboardKey::KEY_BACKSPACE]
            .into_iter()
            .any(|key| ctx.rl.is_key_pressed(key));
        (ui::button::button_clicked(&mut ctx, back, &self.back) || key || ctx.nav.back)
            .then_some(ScreenState::MainMenu)
    }

    fn draw(&self, d: &mut RaylibDrawHandle, ctx: &DrawContext) {
        let screen = ui::screen_size(d);
        let fonts = ctx.fonts();

        ui::draw_background(
            d,
            ctx.resources,
            Some(&style::background(style::ARCHIVE_BACKGROUND)),
        );
        d.draw_rectangle(0, 0, screen.x as i32, screen.y as i32, style::BACKDROP);
        self.frame.draw(
            d,
            Rectangle::new(24.0, 24.0, screen.x - 48.0, screen.y - 48.0),
        );

        let tab = &TABS[self.tab];
        ui::draw_text_centered(
            d,
            fonts,
            "Gallery",
            Vector2::new(screen.x / 2.0, 64.0),
            &self.heading,
        );

        for (index, rect) in self.tab_rects(screen).into_iter().enumerate() {
            let style = if index == self.tab {
                self.tab_button.clone().text_color(style::PARCHMENT)
            } else {
                self.tab_button.clone()
            };
            ui::button::Button::new(TABS[index].name, &style).draw(d, ctx, rect);
        }

        let found = found(tab, |key| ctx.has_seen(key));
        ui::draw_text_centered(
            d,
            fonts,
            &format!("{} OF {} FOUND", found, tab.entries.len()),
            Vector2::new(screen.x / 2.0, 146.0),
            &self.count,
        );

        if tab.entries.iter().all(|entry| entry.track().is_none()) {
            for (entry, rect) in tab.entries.iter().zip(self.cell_rects(tab, screen)) {
                let seen = ctx.has_seen(&entry.key());
                self.cell.draw(d, rect);
                match entry.picture().filter(|_| seen) {
                    Some(path) => {
                        if let Some(picture) = ctx.resources.texture(&path) {
                            let from = source(picture, entry.crop());
                            let into = fit(from, rect);
                            d.draw_texture_pro(
                                picture,
                                from,
                                into,
                                Vector2::zero(),
                                0.0,
                                Color::WHITE,
                            );
                        }
                    }
                    None => ui::draw_text_centered(
                        d,
                        fonts,
                        "???",
                        Vector2::new(rect.x + rect.width / 2.0, rect.y + rect.height / 2.0 - 11.0),
                        &self.locked,
                    ),
                }
                let title = if seen { entry.title } else { "Not yet seen" };
                ui::draw_text_centered(
                    d,
                    fonts,
                    title,
                    Vector2::new(rect.x + rect.width / 2.0, rect.y + rect.height + 6.0),
                    &self.caption,
                );
            }
        } else {
            for (entry, rect) in tab.entries.iter().zip(self.track_rects(tab, screen)) {
                let seen = ctx.has_seen(&entry.key());
                if seen {
                    ui::button::Button::new(entry.title, &self.track).draw(d, ctx, rect);
                } else {
                    self.cell.draw(d, rect);
                    ui::draw_text_centered(
                        d,
                        fonts,
                        "???",
                        Vector2::new(rect.x + rect.width / 2.0, rect.y + 12.0),
                        &self.locked,
                    );
                }
            }
        }

        ui::button::Button::new("Back", &self.back)
            .focused(ctx.interactive && ctx.focus_visible)
            .draw(d, ctx, self.back_rect(screen));
    }
}
