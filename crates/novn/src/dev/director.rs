use std::cell::RefCell;
use std::collections::BTreeSet;

use novn_script::{Severity, compile_source};
use raylib::prelude::*;

use crate::context::{DrawContext, GameContext};
use crate::data::assets::Assets;
use crate::dev::scene_jump::{centred, followed};
use crate::frame::viewport;
use crate::game::audio::AUDIO_EXTENSIONS;
use crate::overlay::{Overlay, OverlayAction};
use crate::ui;
use crate::ui::TextStyle;
use crate::ui::fonts::FontRole;

pub const DIRECTOR_OVERLAY: &str = "dev.director";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Show,
    Background,
    Music,
    Sound,
    Voice,
}

impl Kind {
    pub const ALL: [Kind; 5] = [
        Kind::Show,
        Kind::Background,
        Kind::Music,
        Kind::Sound,
        Kind::Voice,
    ];

    pub fn keyword(self) -> &'static str {
        match self {
            Kind::Show => "show",
            Kind::Background => "background",
            Kind::Music => "music",
            Kind::Sound => "sound",
            Kind::Voice => "voice",
        }
    }

    pub fn folder(self) -> &'static str {
        match self {
            Kind::Show => "characters",
            Kind::Background => "backgrounds",
            Kind::Music => "music",
            Kind::Sound => "sounds",
            Kind::Voice => "voice",
        }
    }

    fn extensions(self) -> &'static [&'static str] {
        match self {
            Kind::Show | Kind::Background => &["png"],
            Kind::Music | Kind::Sound | Kind::Voice => &AUDIO_EXTENSIONS,
        }
    }
}

pub fn candidates(assets: &Assets, kind: Kind) -> Vec<String> {
    let Ok(files) = assets.files_under(kind.folder()) else {
        return Vec::new();
    };
    let prefix = format!("{}/", kind.folder());
    let mut found = BTreeSet::new();
    for file in files {
        let Some(rest) = file.strip_prefix(&prefix) else {
            continue;
        };
        let Some((stem, extension)) = rest.rsplit_once('.') else {
            continue;
        };
        if !kind.extensions().contains(&extension) {
            continue;
        }
        let name = match kind {
            Kind::Show => match stem.split_once('/') {
                Some((character, image)) if !image.contains('/') => {
                    format!("{character} {image}")
                }
                _ => continue,
            },
            _ if stem.contains('/') => continue,
            _ => stem.to_string(),
        };
        if name.split(' ').all(novn_script::is_identifier) {
            found.insert(name);
        }
    }
    found.into_iter().collect()
}

pub fn command_line(kind: Kind, choice: &str) -> String {
    format!("{} {}", kind.keyword(), choice)
}

pub fn insert_above(source: &str, line: usize, text: &str) -> Option<String> {
    if line == 0 {
        return None;
    }
    let newline = if source.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let mut lines: Vec<&str> = source.split(newline).collect();
    let target = lines.get(line - 1)?;
    let indent: String = target
        .chars()
        .take_while(|c| *c == ' ' || *c == '\t')
        .collect();
    let inserted = format!("{indent}{text}");
    lines.insert(line - 1, &inserted);
    Some(lines.join(newline))
}

pub fn errors(source: &str) -> usize {
    compile_source(source)
        .diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .count()
}

pub fn safe(before: &str, after: &str) -> bool {
    errors(after) <= errors(before)
}

thread_local! {
    static PREVIEW: RefCell<Preview> = RefCell::new(Preview::default());
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Preview {
    pub background: Option<String>,
    pub character: Option<(String, String)>,
    pub music: Option<String>,
}

pub fn preview() -> Preview {
    PREVIEW.with(|p| p.borrow().clone())
}

pub fn background() -> Option<String> {
    PREVIEW.with(|p| p.borrow().background.clone())
}

pub fn character() -> Option<(String, String)> {
    PREVIEW.with(|p| p.borrow().character.clone())
}

pub fn music() -> Option<String> {
    PREVIEW.with(|p| p.borrow().music.clone())
}

pub fn set_preview(preview: Preview) {
    PREVIEW.with(|p| *p.borrow_mut() = preview);
}

pub fn clear_preview() {
    set_preview(Preview::default());
}

pub fn preview_for(kind: Kind, choice: &str) -> Preview {
    match kind {
        Kind::Background => Preview {
            background: Some(choice.to_string()),
            ..Preview::default()
        },
        Kind::Show => Preview {
            character: choice
                .split_once(' ')
                .map(|(id, image)| (id.to_string(), image.to_string())),
            ..Preview::default()
        },
        Kind::Music => Preview {
            music: Some(choice.to_string()),
            ..Preview::default()
        },
        Kind::Sound | Kind::Voice => Preview::default(),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Written {
    Line {
        file: String,
        line: usize,
        text: String,
    },
    NoLine,
    NotOnDisk(String),
    WouldBreak(String),
    Failed(String),
}

pub fn write_line(file: &str, line: usize, text: &str) -> Written {
    let path = std::path::Path::new(file);
    if !path.is_file() {
        return Written::NotOnDisk(file.to_string());
    }
    let before = match std::fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => return Written::Failed(error.to_string()),
    };
    let Some(after) = insert_above(&before, line, text) else {
        return Written::NoLine;
    };
    if !safe(&before, &after) {
        return Written::WouldBreak(text.to_string());
    }
    match std::fs::write(path, after) {
        Ok(()) => Written::Line {
            file: file.to_string(),
            line,
            text: text.to_string(),
        },
        Err(error) => Written::Failed(error.to_string()),
    }
}

const WIDTH: f32 = 360.0;
const MARGIN: f32 = 16.0;
const PAD: f32 = 14.0;
const ROW: f32 = 24.0;

#[derive(Default)]
pub struct DirectorOverlay {
    target: Option<Option<(String, usize)>>,
    kind: Option<Kind>,
    kind_at: usize,
    items: Vec<String>,
    item_at: usize,
    item_top: usize,
    counts: Option<[usize; 5]>,
    message: Option<(String, bool)>,
}

impl DirectorOverlay {
    pub fn new() -> Self {
        Self::default()
    }

    fn panel(screen: Vector2) -> Rectangle {
        Rectangle::new(
            screen.x - WIDTH - MARGIN,
            MARGIN,
            WIDTH,
            screen.y - MARGIN * 2.0,
        )
    }

    fn list(panel: Rectangle) -> Rectangle {
        let top = panel.y + PAD + 22.0 * 3.0;
        let bottom = panel.y + panel.height - PAD - 22.0 * 3.0;
        Rectangle::new(panel.x + PAD, top, panel.width - PAD * 2.0, bottom - top)
    }

    fn rows(list: Rectangle) -> usize {
        (list.height / ROW).floor().max(1.0) as usize
    }

    fn open(&mut self, ctx: &mut GameContext, kind: Kind, rows: usize) {
        self.kind = Some(kind);
        self.items = candidates(ctx.resources.assets(), kind);
        self.item_at = 0;
        self.item_top = centred(0, self.items.len(), rows);
        self.message = None;
        self.audition(ctx);
    }

    fn back(&mut self) {
        self.kind = None;
        self.items.clear();
        clear_preview();
    }

    fn audition(&mut self, ctx: &mut GameContext) {
        let (Some(kind), Some(choice)) = (self.kind, self.items.get(self.item_at).cloned()) else {
            clear_preview();
            return;
        };
        let preview = preview_for(kind, &choice);
        if let Some(background) = &preview.background {
            let path = crate::data::resources::background_path(background);
            ctx.resources.get_or_load(&path, ctx.rl, ctx.thread);
        }
        if let Some((id, image)) = &preview.character {
            let path = crate::data::resources::character_path(id, image);
            ctx.resources.get_or_load(&path, ctx.rl, ctx.thread);
        }
        match kind {
            Kind::Sound => ctx.audio.play_sound(&choice),
            Kind::Voice => ctx.audio.play_voice(&choice),
            _ => {}
        }
        set_preview(preview);
    }

    fn write(&mut self) -> Option<(String, bool)> {
        let (Some(kind), Some(choice)) = (self.kind, self.items.get(self.item_at)) else {
            return None;
        };
        let text = command_line(kind, choice);
        let result = match self.target.clone().flatten() {
            Some((file, line)) => write_line(&file, line, &text),
            None => Written::NoLine,
        };
        Some(match result {
            Written::Line { file, line, text } => {
                println!("Director: wrote `{text}` above line {line} of {file}");
                let name = file.rsplit(['/', '\\']).next().unwrap_or(&file).to_string();
                (format!("Wrote `{text}` above line {line} of {name}"), true)
            }
            Written::NoLine => ("There is no line on screen to write above.".into(), false),
            Written::NotOnDisk(file) => (
                format!("{file} is not a file on disk, so there is nothing to write to."),
                false,
            ),
            Written::WouldBreak(text) => (
                format!("Not written: `{text}` here would stop the story compiling."),
                false,
            ),
            Written::Failed(error) => (format!("Not written: {error}"), false),
        })
    }
}

impl Drop for DirectorOverlay {
    fn drop(&mut self) {
        clear_preview();
    }
}

impl Overlay for DirectorOverlay {
    fn update(&mut self, mut ctx: GameContext) -> OverlayAction {
        if self.target.is_none() {
            self.target = Some(
                ctx.story
                    .current_source()
                    .map(|(file, line)| (file.to_string(), line)),
            );
        }
        if self.counts.is_none() {
            let assets = ctx.resources.assets();
            self.counts = Some(Kind::ALL.map(|kind| candidates(assets, kind).len()));
        }

        let screen = ui::screen_size(ctx.rl);
        let panel = Self::panel(screen);
        let list = Self::list(panel);
        let rows = Self::rows(list);
        let mouse = viewport::mouse_position(ctx.rl);
        let clicked = ctx
            .rl
            .is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT);
        let escape = ctx.rl.is_key_pressed(KeyboardKey::KEY_ESCAPE);
        let nav = ctx.nav;

        if nav.back || escape {
            return OverlayAction::Close;
        }

        let hit = clicked
            .then(|| {
                let inside = Rectangle::new(list.x, list.y, list.width, list.height);
                inside
                    .check_collision_point_rec(mouse)
                    .then(|| ((mouse.y - list.y) / ROW) as usize)
            })
            .flatten();

        match self.kind {
            None => {
                if nav.up {
                    self.kind_at = self.kind_at.saturating_sub(1);
                }
                if nav.down && self.kind_at + 1 < Kind::ALL.len() {
                    self.kind_at += 1;
                }
                let mut open = nav.accept || nav.right;
                if let Some(row) = hit.filter(|&row| row < Kind::ALL.len()) {
                    open |= row == self.kind_at;
                    self.kind_at = row;
                }
                if open {
                    self.open(&mut ctx, Kind::ALL[self.kind_at], rows);
                }
            }
            Some(_) => {
                let before = self.item_at;
                if nav.left {
                    self.back();
                    return OverlayAction::Stay;
                }
                if nav.up {
                    self.item_at = self.item_at.saturating_sub(1);
                }
                if nav.down && self.item_at + 1 < self.items.len() {
                    self.item_at += 1;
                }
                let mut write = nav.accept;
                if let Some(row) = hit {
                    let at = self.item_top + row;
                    if at < self.items.len() {
                        write |= at == self.item_at;
                        self.item_at = at;
                    }
                }
                self.item_top = followed(self.item_top, self.item_at, self.items.len(), rows);
                if self.item_at != before {
                    self.message = None;
                    self.audition(&mut ctx);
                }
                if write && let Some((message, written)) = self.write() {
                    if written {
                        ctx.notify(message);
                        return OverlayAction::Close;
                    }
                    self.message = Some((message, false));
                }
            }
        }
        OverlayAction::Stay
    }

    fn draw(&self, d: &mut RaylibDrawHandle, ctx: &DrawContext) {
        let fonts = ctx.fonts();
        let screen = ui::screen_size(d);
        let panel = Self::panel(screen);
        if d.is_key_down(KeyboardKey::KEY_H) {
            let tab = Rectangle::new(panel.x + panel.width - 150.0, panel.y, 150.0, 26.0);
            d.draw_rectangle_rec(tab, Color::new(20, 21, 28, 200));
            ui::draw_text(
                d,
                fonts,
                "Director hidden",
                Vector2::new(tab.x + 10.0, tab.y + 5.0),
                &TextStyle::new(FontRole::Menu, 13.0, Color::new(150, 154, 168, 255)),
            );
            return;
        }
        let list = Self::list(panel);
        let rows = Self::rows(list);
        let ink = Color::new(236, 236, 240, 255);
        let dim = Color::new(150, 154, 168, 255);
        let accent = Color::new(242, 190, 92, 255);
        let title = TextStyle::new(FontRole::Menu, 18.0, ink);
        let text = TextStyle::new(FontRole::Menu, 15.0, ink);
        let muted = TextStyle::new(FontRole::Menu, 15.0, dim);
        let small = TextStyle::new(FontRole::Menu, 13.0, dim);

        d.draw_rectangle_rec(panel, Color::new(20, 21, 28, 225));
        d.draw_rectangle_lines_ex(panel, 1.0, Color::new(70, 74, 92, 255));
        let x = panel.x + PAD;
        let mut y = panel.y + PAD;
        let heading = match self.kind {
            None => "Director".to_string(),
            Some(kind) => format!("Director  /  {}", kind.keyword()),
        };
        ui::draw_text(d, fonts, &heading, Vector2::new(x, y), &title);
        y += 26.0;
        let place = match self.target.clone().flatten() {
            Some((file, line)) => {
                let name = file.rsplit(['/', '\\']).next().unwrap_or(&file).to_string();
                format!("Writes above line {line} of {name}")
            }
            None => "No line on screen to write above".to_string(),
        };
        ui::draw_text(d, fonts, &place, Vector2::new(x, y), &small);

        let band = |d: &mut RaylibDrawHandle, index: usize| {
            let top = list.y + ROW * index as f32;
            d.draw_rectangle_rec(
                Rectangle::new(list.x - 6.0, top, list.width + 12.0, ROW),
                Color::new(242, 190, 92, 40),
            );
        };

        match self.kind {
            None => {
                for (index, kind) in Kind::ALL.iter().enumerate() {
                    let top = list.y + ROW * index as f32;
                    if index == self.kind_at {
                        band(d, index);
                    }
                    let style = if index == self.kind_at { &text } else { &muted };
                    ui::draw_text(
                        d,
                        fonts,
                        kind.keyword(),
                        Vector2::new(list.x, top + 3.0),
                        style,
                    );
                    let count = self
                        .counts
                        .map(|counts| counts[index].to_string())
                        .unwrap_or_default();
                    let size = ui::measure_text(fonts, &count, &small);
                    ui::draw_text(
                        d,
                        fonts,
                        &count,
                        Vector2::new(list.x + list.width - size.x, top + 5.0),
                        &small,
                    );
                }
            }
            Some(kind) => {
                if self.items.is_empty() {
                    ui::draw_text(
                        d,
                        fonts,
                        &format!("Nothing in {}/", kind.folder()),
                        Vector2::new(list.x, list.y + 3.0),
                        &muted,
                    );
                }
                for (index, item) in self.items.iter().enumerate().skip(self.item_top).take(rows) {
                    let row = index - self.item_top;
                    if index == self.item_at {
                        band(d, row);
                    }
                    let style = if index == self.item_at { &text } else { &muted };
                    ui::draw_text(
                        d,
                        fonts,
                        item,
                        Vector2::new(list.x, list.y + ROW * row as f32 + 3.0),
                        style,
                    );
                }
            }
        }

        let mut y = panel.y + panel.height - PAD - 22.0 * 3.0 + 6.0;
        if let Some((message, good)) = &self.message {
            let colour = if *good {
                Color::new(130, 210, 160, 255)
            } else {
                Color::new(255, 150, 130, 255)
            };
            for line in fonts.wrap(FontRole::Menu, message, 13.0, panel.width - PAD * 2.0) {
                ui::draw_text(
                    d,
                    fonts,
                    &line,
                    Vector2::new(x, y),
                    &TextStyle::new(FontRole::Menu, 13.0, colour),
                );
                y += 17.0;
            }
        }
        let hint = match self.kind {
            None => "Up/Down choose   Enter opens   Esc closes",
            Some(_) => "Up/Down audition   Enter writes   Left back   hold H to peek",
        };
        ui::draw_text(
            d,
            fonts,
            hint,
            Vector2::new(x, panel.y + panel.height - PAD - 14.0),
            &TextStyle::new(FontRole::Menu, 12.0, accent),
        );
    }
}
