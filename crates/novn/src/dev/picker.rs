use std::cell::{Cell, RefCell};

use novn_script::Position;
use raylib::prelude::*;

use crate::context::{DrawContext, GameContext};
use crate::frame::viewport;
use crate::overlay::{Overlay, OverlayAction};
use crate::ui;
use crate::ui::TextStyle;
use crate::ui::fonts::FontRole;

pub const POSITION_PICKER_OVERLAY: &str = "dev.position_picker";

thread_local! {
    static LISTENING: Cell<bool> = const { Cell::new(false) };
    static CURRENT: RefCell<Snapshot> = RefCell::new(Snapshot::default());
    static LAST: RefCell<Snapshot> = RefCell::new(Snapshot::default());
    static DRAG: RefCell<Option<(String, f32)>> = const { RefCell::new(None) };
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Drawn {
    pub character: String,
    pub image: String,
    pub rect: Rectangle,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Snapshot {
    pub characters: Vec<Drawn>,
    pub slots: Option<[f32; 5]>,
}

pub fn listening() -> bool {
    LISTENING.with(Cell::get)
}

pub fn listen(on: bool) {
    LISTENING.with(|l| l.set(on));
    if !on {
        DRAG.with(|d| *d.borrow_mut() = None);
        CURRENT.with(|c| *c.borrow_mut() = Snapshot::default());
        LAST.with(|c| *c.borrow_mut() = Snapshot::default());
    }
}

pub fn begin_frame() {
    if listening() {
        let done = CURRENT.with(|c| std::mem::take(&mut *c.borrow_mut()));
        LAST.with(|l| *l.borrow_mut() = done);
    }
}

pub fn record_slots(slots: [f32; 5]) {
    if listening() {
        CURRENT.with(|c| c.borrow_mut().slots = Some(slots));
    }
}

pub fn record(character: &str, image: &str, rect: Rectangle) {
    if listening() {
        CURRENT.with(|c| {
            c.borrow_mut().characters.push(Drawn {
                character: character.to_string(),
                image: image.to_string(),
                rect,
            });
        });
    }
}

pub fn last() -> Snapshot {
    LAST.with(|l| l.borrow().clone())
}

pub fn current() -> Snapshot {
    CURRENT.with(|c| c.borrow().clone())
}

pub fn dragged_x(character: &str) -> Option<f32> {
    DRAG.with(|d| {
        d.borrow()
            .as_ref()
            .filter(|(id, _)| id == character)
            .map(|(_, x)| *x)
    })
}

pub fn set_drag(character: Option<(&str, f32)>) {
    DRAG.with(|d| *d.borrow_mut() = character.map(|(id, x)| (id.to_string(), x)));
}

pub fn nearest(slots: &[f32; 5], x: f32) -> Position {
    Position::ALL
        .iter()
        .zip(slots)
        .min_by(|(_, a), (_, b)| (**a - x).abs().total_cmp(&(**b - x).abs()))
        .map(|(position, _)| *position)
        .unwrap_or(Position::Center)
}

pub fn story_line(character: &str, image: &str, slot: Position) -> String {
    format!("show {character} {image} at {}", slot.name())
}

pub fn rust_line(slot: Position, x: f32) -> String {
    format!(".position(Position::{slot:?}, {x:.3})")
}

pub fn hit(characters: &[Drawn], point: Vector2) -> Option<&Drawn> {
    characters
        .iter()
        .rev()
        .find(|drawn| drawn.rect.check_collision_point_rec(point))
}

#[derive(Clone, Debug, PartialEq)]
struct Held {
    character: String,
    image: String,
    grab: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Placed {
    pub character: String,
    pub image: String,
    pub x: f32,
}

#[derive(Default)]
pub struct PositionPickerOverlay {
    held: Option<Held>,
    placed: Option<Placed>,
    copied: Option<&'static str>,
}

impl PositionPickerOverlay {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Drop for PositionPickerOverlay {
    fn drop(&mut self) {
        listen(false);
    }
}

impl Overlay for PositionPickerOverlay {
    fn update(&mut self, ctx: GameContext) -> OverlayAction {
        listen(true);
        if ctx.nav.back || ctx.rl.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
            return OverlayAction::Close;
        }

        let screen = ui::screen_size(ctx.rl);
        let mouse = viewport::mouse_position(ctx.rl);
        let snapshot = last();

        if ctx
            .rl
            .is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT)
            && let Some(drawn) = hit(&snapshot.characters, mouse)
        {
            let centre = drawn.rect.x + drawn.rect.width / 2.0;
            self.held = Some(Held {
                character: drawn.character.clone(),
                image: drawn.image.clone(),
                grab: mouse.x - centre,
            });
            self.copied = None;
        }

        if let Some(held) = &self.held {
            let x = ((mouse.x - held.grab) / screen.x.max(1.0)).clamp(0.0, 1.0);
            set_drag(Some((&held.character, x)));
            self.placed = Some(Placed {
                character: held.character.clone(),
                image: held.image.clone(),
                x,
            });
            if ctx
                .rl
                .is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT)
            {
                self.held = None;
                if let (Some(placed), Some(slots)) = (&self.placed, snapshot.slots) {
                    let line =
                        story_line(&placed.character, &placed.image, nearest(&slots, placed.x));
                    println!("{line}");
                    if ctx.rl.set_clipboard_text(&line).is_ok() {
                        self.copied = Some("story line");
                    }
                }
            }
        }

        if ctx.nav.accept
            && let (Some(placed), Some(slots)) = (&self.placed, snapshot.slots)
        {
            let line = rust_line(nearest(&slots, placed.x), placed.x);
            println!("{line}");
            if ctx.rl.set_clipboard_text(&line).is_ok() {
                self.copied = Some("Rust line");
            }
        }

        OverlayAction::Stay
    }

    fn draw(&self, d: &mut RaylibDrawHandle, ctx: &DrawContext) {
        let fonts = ctx.fonts();
        let screen = ui::screen_size(d);
        let snapshot = current();
        let ink = Color::new(236, 236, 240, 255);
        let dim = Color::new(150, 154, 168, 255);
        let accent = Color::new(242, 190, 92, 255);
        let small = TextStyle::new(FontRole::Menu, 13.0, dim);
        let text = TextStyle::new(FontRole::Menu, 15.0, ink);
        let code = TextStyle::new(FontRole::Menu, 15.0, accent);

        let slot = match (&self.placed, snapshot.slots) {
            (Some(placed), Some(slots)) => Some(nearest(&slots, placed.x)),
            _ => None,
        };

        if let Some(slots) = snapshot.slots {
            for (position, x) in Position::ALL.iter().zip(slots) {
                let on = slot == Some(*position);
                let line_x = screen.x * x;
                let colour = if on {
                    Color::new(242, 190, 92, 220)
                } else {
                    Color::new(150, 154, 168, 90)
                };
                let mut y = 0.0;
                while y < screen.y {
                    d.draw_line_ex(
                        Vector2::new(line_x, y),
                        Vector2::new(line_x, (y + 10.0).min(screen.y)),
                        if on { 2.0 } else { 1.0 },
                        colour,
                    );
                    y += 18.0;
                }
                let label = position.name();
                let size = ui::measure_text(fonts, label, &small);
                let tag = Rectangle::new(
                    line_x - size.x / 2.0 - 6.0,
                    44.0,
                    size.x + 12.0,
                    size.y + 6.0,
                );
                d.draw_rectangle_rec(tag, Color::new(14, 15, 20, 220));
                ui::draw_text(
                    d,
                    fonts,
                    label,
                    Vector2::new(tag.x + 6.0, tag.y + 3.0),
                    &TextStyle::new(FontRole::Menu, 13.0, if on { accent } else { dim }),
                );
            }
        }

        for drawn in &snapshot.characters {
            let dragging = self
                .placed
                .as_ref()
                .is_some_and(|placed| placed.character == drawn.character);
            let colour = if dragging {
                accent
            } else {
                Color::new(90, 200, 255, 160)
            };
            d.draw_rectangle_lines_ex(drawn.rect, if dragging { 2.0 } else { 1.0 }, colour);
        }

        let mut lines: Vec<(String, TextStyle)> = Vec::new();
        match (&self.placed, slot) {
            (Some(placed), Some(slot)) => {
                lines.push(("In the story".into(), small.clone()));
                lines.push((
                    story_line(&placed.character, &placed.image, slot),
                    code.clone(),
                ));
                lines.push((
                    format!(
                        "Exactly {:.3} of the width. To move the {} slot there, in Rust:",
                        placed.x,
                        slot.name()
                    ),
                    small.clone(),
                ));
                lines.push((rust_line(slot, placed.x), code.clone()));
            }
            _ if snapshot.characters.is_empty() => {
                lines.push(("No character on stage to move.".into(), text.clone()));
            }
            _ => {
                lines.push(("Drag a character to place it.".into(), text.clone()));
            }
        }
        let hint = match self.copied {
            Some(what) => format!("Copied the {what}.   Enter copies the Rust line.   Esc closes."),
            None => "Letting go copies the story line.   Enter copies the Rust line.   Esc closes."
                .into(),
        };
        lines.push((hint, small.clone()));

        let width = lines
            .iter()
            .map(|(line, style)| ui::measure_text(fonts, line, style).x)
            .fold(0.0, f32::max)
            + 28.0;
        let height = lines.len() as f32 * 22.0 + 20.0;
        let panel = Rectangle::new((screen.x - width) / 2.0, 80.0, width, height);
        d.draw_rectangle_rec(panel, Color::new(14, 15, 20, 235));
        d.draw_rectangle_lines_ex(panel, 1.0, Color::new(70, 74, 92, 255));
        let mut y = panel.y + 10.0;
        for (line, style) in &lines {
            ui::draw_text(d, fonts, line, Vector2::new(panel.x + 14.0, y), style);
            y += 22.0;
        }
    }
}
