use std::cell::{Cell, RefCell};

use raylib::prelude::*;

use crate::context::{DrawContext, GameContext};
use crate::dev::inspector::{self, Styled};
use crate::dev::scene_jump::{centred, followed};
use crate::frame::viewport;
use crate::overlay::{Overlay, OverlayAction};
use crate::ui;
use crate::ui::TextStyle;
use crate::ui::button::ButtonStyle;
use crate::ui::fonts::FontRole;
use crate::ui::shape::{CornerShape, Corners, PanelStyle};

pub const STYLE_EDITOR_OVERLAY: &str = "dev.style_editor";

thread_local! {
    static EDITING: Cell<bool> = const { Cell::new(false) };
    static BUTTONS: RefCell<Vec<(ButtonStyle, ButtonStyle)>> = const { RefCell::new(Vec::new()) };
    static PANELS: RefCell<Vec<(PanelStyle, PanelStyle)>> = const { RefCell::new(Vec::new()) };
}

pub fn editing() -> bool {
    EDITING.with(Cell::get)
}

pub fn button(original: &ButtonStyle) -> Option<ButtonStyle> {
    if !editing() {
        return None;
    }
    BUTTONS.with(|b| {
        b.borrow()
            .iter()
            .find(|(from, _)| from == original)
            .map(|(_, to)| to.clone())
    })
}

pub fn panel(original: &PanelStyle) -> Option<PanelStyle> {
    if !editing() {
        return None;
    }
    PANELS.with(|p| {
        p.borrow()
            .iter()
            .find(|(from, _)| from == original)
            .map(|(_, to)| *to)
    })
}

fn set_override(original: &Styled, edited: Styled) {
    match (original, edited) {
        (Styled::Button(from), Styled::Button(to)) => BUTTONS.with(|b| {
            let mut b = b.borrow_mut();
            b.retain(|(f, _)| f != from.as_ref());
            b.push(((**from).clone(), *to));
        }),
        (Styled::Panel(from), Styled::Panel(to)) => PANELS.with(|p| {
            let mut p = p.borrow_mut();
            p.retain(|(f, _)| f != from);
            p.push((*from, to));
        }),
        _ => {}
    }
}

pub fn resized(rect: Rectangle, original: &ButtonStyle, edited: &ButtonStyle) -> Rectangle {
    if original.width <= 0.0 || original.height <= 0.0 {
        return rect;
    }
    let width = rect.width * edited.width / original.width;
    let height = rect.height * edited.height / original.height;
    Rectangle::new(
        rect.x + (rect.width - width) / 2.0,
        rect.y + (rect.height - height) / 2.0,
        width,
        height,
    )
}

pub fn union(rects: impl IntoIterator<Item = Rectangle>) -> Option<Rectangle> {
    rects.into_iter().reduce(|a, b| {
        let left = a.x.min(b.x);
        let top = a.y.min(b.y);
        let right = (a.x + a.width).max(b.x + b.width);
        let bottom = (a.y + a.height).max(b.y + b.height);
        Rectangle::new(left, top, right - left, bottom - top)
    })
}

pub fn placement(screen: Vector2, avoid: Option<Rectangle>, width: f32, margin: f32) -> Rectangle {
    let full = screen.y - margin * 2.0;
    let Some(area) = avoid else {
        return Rectangle::new(screen.x - width - margin, margin, width, full);
    };
    let right = Rectangle::new(screen.x - width - margin, margin, width, full);
    let left = Rectangle::new(margin, margin, width, full);
    let overlap = |panel: Rectangle| {
        let x = (panel.x + panel.width).min(area.x + area.width) - panel.x.max(area.x);
        x.max(0.0)
    };
    let mut panel = if overlap(right) <= overlap(left) {
        right
    } else {
        left
    };
    if overlap(panel) > 0.0 {
        let middle = area.y + area.height / 2.0;
        if middle > screen.y / 2.0 {
            panel.height = (area.y - margin * 2.0).max(300.0_f32.min(full));
        } else {
            let top = area.y + area.height + margin;
            panel.y = top.min(screen.y - margin - 300.0_f32.min(full));
            panel.height = screen.y - margin - panel.y;
        }
    }
    panel
}

pub fn clear() {
    EDITING.with(|e| e.set(false));
    BUTTONS.with(|b| b.borrow_mut().clear());
    PANELS.with(|p| p.borrow_mut().clear());
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Channel {
    R,
    G,
    B,
    A,
}

impl Channel {
    const ALL: [Channel; 4] = [Channel::R, Channel::G, Channel::B, Channel::A];

    fn get(self, color: Color) -> u8 {
        match self {
            Channel::R => color.r,
            Channel::G => color.g,
            Channel::B => color.b,
            Channel::A => color.a,
        }
    }

    fn set(self, mut color: Color, value: u8) -> Color {
        match self {
            Channel::R => color.r = value,
            Channel::G => color.g = value,
            Channel::B => color.b = value,
            Channel::A => color.a = value,
        }
        color
    }

    fn name(self) -> &'static str {
        match self {
            Channel::R => "r",
            Channel::G => "g",
            Channel::B => "b",
            Channel::A => "a",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Knob {
    Width,
    Height,
    FontSize,
    Text(Channel),
    Fill(Channel),
    PaddingX,
    PaddingY,
    Roundness,
    BorderWidth,
    Border(Channel),
}

pub fn knobs(styled: &Styled) -> Vec<Knob> {
    let mut knobs = Vec::new();
    if matches!(styled, Styled::Button(_)) {
        knobs.extend([Knob::Width, Knob::Height, Knob::FontSize]);
        knobs.extend(Channel::ALL.map(Knob::Text));
    }
    knobs.extend(Channel::ALL.map(Knob::Fill));
    if matches!(styled, Styled::Button(_)) {
        knobs.extend([Knob::PaddingX, Knob::PaddingY]);
    }
    knobs.extend([Knob::Roundness, Knob::BorderWidth]);
    knobs.extend(Channel::ALL.map(Knob::Border));
    knobs
}

impl Knob {
    pub fn label(self) -> String {
        match self {
            Knob::Width => "width".into(),
            Knob::Height => "height".into(),
            Knob::FontSize => "font size".into(),
            Knob::Text(c) => format!("text {}", c.name()),
            Knob::Fill(c) => format!("fill {}", c.name()),
            Knob::PaddingX => "padding x".into(),
            Knob::PaddingY => "padding y".into(),
            Knob::Roundness => "roundness".into(),
            Knob::BorderWidth => "border".into(),
            Knob::Border(c) => format!("border {}", c.name()),
        }
    }

    pub fn step(self, big: bool) -> f32 {
        match (self, big) {
            (Knob::Roundness, false) => 0.05,
            (Knob::Roundness, true) => 0.2,
            (Knob::Text(_) | Knob::Fill(_) | Knob::Border(_), false) => 5.0,
            (Knob::Text(_) | Knob::Fill(_) | Knob::Border(_), true) => 25.0,
            (_, false) => 1.0,
            (_, true) => 10.0,
        }
    }

    fn range(self) -> (f32, f32) {
        match self {
            Knob::Roundness => (0.0, 1.0),
            Knob::Text(_) | Knob::Fill(_) | Knob::Border(_) => (0.0, 255.0),
            Knob::FontSize => (1.0, 400.0),
            _ => (0.0, 4000.0),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Edit {
    pub size: Option<(f32, f32)>,
    pub font_size: Option<f32>,
    pub text_color: Option<Color>,
    pub color: Option<Color>,
    pub padding: Option<(f32, f32)>,
    pub roundness: Option<f32>,
    pub border: Option<(f32, Color)>,
}

pub fn roundness_of(corners: &Corners) -> Option<f32> {
    let all = [
        corners.top_left,
        corners.top_right,
        corners.bottom_right,
        corners.bottom_left,
    ];
    if all.iter().all(|c| c.shape == CornerShape::Square) {
        return Some(0.0);
    }
    let first = all[0];
    (corners.relative && first.shape == CornerShape::Round && all.iter().all(|c| *c == first))
        .then_some(first.size)
}

fn fill_of(styled: &Styled) -> Color {
    match styled {
        Styled::Button(b) => b.color,
        Styled::Panel(p) => p.color,
    }
}

fn corners_of(styled: &Styled) -> Corners {
    match styled {
        Styled::Button(b) => b.corners,
        Styled::Panel(p) => p.corners,
    }
}

fn border_of(styled: &Styled) -> Option<(f32, Color)> {
    match styled {
        Styled::Button(b) => b.border.map(|b| (b.width, b.color)),
        Styled::Panel(p) => p.border.map(|b| (b.width, b.color)),
    }
}

impl Edit {
    pub fn apply(&self, styled: &Styled) -> Styled {
        match styled {
            Styled::Button(original) => {
                let mut b = (**original).clone();
                if let Some((w, h)) = self.size {
                    b = b.size(w, h);
                }
                if let Some(size) = self.font_size {
                    b = b.font_size(size);
                }
                if let Some(color) = self.text_color {
                    b = b.text_color(color);
                }
                if let Some(color) = self.color {
                    b = b.color(color);
                }
                if let Some((x, y)) = self.padding {
                    b = b.padding(x, y);
                }
                if let Some(r) = self.roundness {
                    b = b.roundness(r);
                }
                if let Some((w, c)) = self.border {
                    b = b.border(w, c);
                }
                Styled::Button(Box::new(b))
            }
            Styled::Panel(original) => {
                let mut p = *original;
                if let Some(color) = self.color {
                    p = p.color(color);
                }
                if let Some(r) = self.roundness {
                    p = p.roundness(r);
                }
                if let Some((w, c)) = self.border {
                    p = p.border(w, c);
                }
                Styled::Panel(p)
            }
        }
    }

    pub fn value(styled: &Styled, knob: Knob) -> Option<f32> {
        let button = match styled {
            Styled::Button(b) => Some(b),
            Styled::Panel(_) => None,
        };
        Some(match knob {
            Knob::Width => button?.width,
            Knob::Height => button?.height,
            Knob::FontSize => button?.text.size,
            Knob::Text(c) => c.get(button?.text.color) as f32,
            Knob::Fill(c) => c.get(fill_of(styled)) as f32,
            Knob::PaddingX => button?.padding.x,
            Knob::PaddingY => button?.padding.y,
            Knob::Roundness => roundness_of(&corners_of(styled))?,
            Knob::BorderWidth => border_of(styled).map_or(0.0, |(w, _)| w),
            Knob::Border(c) => border_of(styled).map_or(0.0, |(_, colour)| c.get(colour) as f32),
        })
    }

    pub fn nudge(&mut self, original: &Styled, knob: Knob, by: f32) {
        let current = self.apply(original);
        let (low, high) = knob.range();
        let start = Self::value(&current, knob).unwrap_or(0.0);
        let value = (start + by).clamp(low, high);
        let value = if matches!(knob, Knob::Roundness) {
            (value * 100.0).round() / 100.0
        } else {
            value.round()
        };
        let channel = value as u8;
        match knob {
            Knob::Width | Knob::Height => {
                let (Some(w), Some(h)) = (
                    Self::value(&current, Knob::Width),
                    Self::value(&current, Knob::Height),
                ) else {
                    return;
                };
                self.size = Some(if knob == Knob::Width {
                    (value, h)
                } else {
                    (w, value)
                });
            }
            Knob::FontSize => self.font_size = Some(value),
            Knob::Text(c) => {
                if let Styled::Button(b) = &current {
                    self.text_color = Some(c.set(b.text.color, channel));
                }
            }
            Knob::Fill(c) => self.color = Some(c.set(fill_of(&current), channel)),
            Knob::PaddingX | Knob::PaddingY => {
                if let Styled::Button(b) = &current {
                    self.padding = Some(if knob == Knob::PaddingX {
                        (value, b.padding.y)
                    } else {
                        (b.padding.x, value)
                    });
                }
            }
            Knob::Roundness => self.roundness = Some(value),
            Knob::BorderWidth => {
                let colour = border_of(&current).map_or(Color::WHITE, |(_, c)| c);
                self.border = Some((value, colour));
            }
            Knob::Border(c) => {
                let (width, colour) = border_of(&current).unwrap_or((1.0, Color::WHITE));
                self.border = Some((width, c.set(colour, channel)));
            }
        }
    }

    pub fn rust(&self) -> String {
        self.calls().concat()
    }

    pub fn calls(&self) -> Vec<String> {
        let colour = |c: Color| format!("Color::new({}, {}, {}, {})", c.r, c.g, c.b, c.a);
        let number = |v: f32| format!("{v:.1}");
        let mut calls = Vec::new();
        if let Some((w, h)) = self.size {
            calls.push(format!(".size({}, {})", number(w), number(h)));
        }
        if let Some(size) = self.font_size {
            calls.push(format!(".font_size({})", number(size)));
        }
        if let Some(c) = self.text_color {
            calls.push(format!(".text_color({})", colour(c)));
        }
        if let Some(c) = self.color {
            calls.push(format!(".color({})", colour(c)));
        }
        if let Some((x, y)) = self.padding {
            calls.push(format!(".padding({}, {})", number(x), number(y)));
        }
        if let Some(r) = self.roundness {
            calls.push(format!(".roundness({r:.2})"));
        }
        if let Some((w, c)) = self.border {
            calls.push(format!(".border({}, {})", number(w), colour(c)));
        }
        calls
    }

    pub fn is_empty(&self) -> bool {
        *self == Edit::default()
    }
}

const WIDTH: f32 = 330.0;
const MARGIN: f32 = 16.0;
const PAD: f32 = 14.0;
const ROW: f32 = 22.0;

fn describe(styled: &Styled) -> &'static str {
    match styled {
        Styled::Button(_) => "button",
        Styled::Panel(_) => "panel",
    }
}

#[derive(Default)]
pub struct StyleEditorOverlay {
    selected: Option<Styled>,
    label: String,
    rect: Option<Rectangle>,
    area: Option<Rectangle>,
    edit: Edit,
    kept: Vec<(Styled, Edit)>,
    row: usize,
    top: usize,
    copied: bool,
}

impl StyleEditorOverlay {
    pub fn new() -> Self {
        Self::default()
    }

    fn panel(&self, screen: Vector2) -> Rectangle {
        placement(screen, self.area.or(self.rect), WIDTH, MARGIN)
    }

    fn list(panel: Rectangle) -> Rectangle {
        let top = panel.y + PAD + 52.0;
        let bottom = panel.y + panel.height - PAD - 110.0;
        Rectangle::new(panel.x + PAD, top, panel.width - PAD * 2.0, bottom - top)
    }

    fn rows(list: Rectangle) -> usize {
        (list.height / ROW).floor().max(1.0) as usize
    }

    fn select(&mut self, widget: &inspector::Widget, rows: usize) {
        let Some(styled) = widget.style.clone() else {
            return;
        };
        if let Some(previous) = self.selected.take() {
            let edit = std::mem::take(&mut self.edit);
            self.kept.retain(|(s, _)| *s != previous);
            if !edit.is_empty() {
                self.kept.push((previous, edit));
            }
        }
        self.edit = self
            .kept
            .iter()
            .find(|(s, _)| *s == styled)
            .map(|(_, e)| e.clone())
            .unwrap_or_default();
        self.label = if widget.label.is_empty() {
            widget.kind.to_string()
        } else {
            format!("{} \"{}\"", widget.kind, widget.label)
        };
        self.rect = Some(widget.rect);
        self.selected = Some(styled);
        self.row = 0;
        self.top = centred(0, self.knobs().len(), rows);
        self.copied = false;
    }

    fn knobs(&self) -> Vec<Knob> {
        self.selected.as_ref().map(knobs).unwrap_or_default()
    }

    fn refresh(&self) {
        if let Some(original) = &self.selected {
            set_override(original, self.edit.apply(original));
        }
    }
}

impl Drop for StyleEditorOverlay {
    fn drop(&mut self) {
        clear();
    }
}

impl Overlay for StyleEditorOverlay {
    fn update(&mut self, ctx: GameContext) -> OverlayAction {
        EDITING.with(|e| e.set(true));
        if let Some(selected) = &self.selected {
            let last = inspector::last();
            self.area = union(
                last.widgets
                    .iter()
                    .filter(|w| w.style.as_ref() == Some(selected))
                    .map(|w| w.rect),
            );
        }
        let escape = ctx.rl.is_key_pressed(KeyboardKey::KEY_ESCAPE);
        if ctx.nav.back || escape {
            return OverlayAction::Close;
        }

        let screen = ui::screen_size(ctx.rl);
        let panel = self.panel(screen);
        let list = Self::list(panel);
        let rows = Self::rows(list);
        let mouse = viewport::mouse_position(ctx.rl);
        let big = ctx.rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT)
            || ctx.rl.is_key_down(KeyboardKey::KEY_RIGHT_SHIFT);

        if ctx
            .rl
            .is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT)
        {
            if panel.check_collision_point_rec(mouse) {
                if list.check_collision_point_rec(mouse) {
                    let at = self.top + ((mouse.y - list.y) / ROW) as usize;
                    if at < self.knobs().len() {
                        self.row = at;
                    }
                }
            } else {
                let last = inspector::last();
                let picked = last
                    .widgets
                    .iter()
                    .rev()
                    .find(|w| w.style.is_some() && w.rect.check_collision_point_rec(mouse))
                    .cloned();
                if let Some(widget) = picked {
                    self.select(&widget, rows);
                }
            }
        }

        let knobs = self.knobs();
        if let Some(original) = self.selected.clone()
            && !knobs.is_empty()
        {
            if ctx.nav.up {
                self.row = self.row.saturating_sub(1);
            }
            if ctx.nav.down && self.row + 1 < knobs.len() {
                self.row += 1;
            }
            let wheel = ctx.rl.get_mouse_wheel_move();
            let over = panel.check_collision_point_rec(mouse);
            let by = if ctx.nav.right || (over && wheel > 0.0) {
                1.0
            } else if ctx.nav.left || (over && wheel < 0.0) {
                -1.0
            } else {
                0.0
            };
            let knob = knobs[self.row.min(knobs.len() - 1)];
            if by != 0.0 {
                self.edit.nudge(&original, knob, by * knob.step(big));
                self.copied = false;
            }
            if ctx.rl.is_key_pressed(KeyboardKey::KEY_R) {
                self.edit = Edit::default();
                self.copied = false;
            }
            if ctx.nav.accept && !self.edit.is_empty() {
                let rust = self.edit.rust();
                println!("{rust}");
                self.copied = ctx.rl.set_clipboard_text(&rust).is_ok();
            }
            self.top = followed(self.top, self.row, knobs.len(), rows);
            self.refresh();
        }
        OverlayAction::Stay
    }

    fn draw(&self, d: &mut RaylibDrawHandle, ctx: &DrawContext) {
        let fonts = ctx.fonts();
        let screen = ui::screen_size(d);
        let mouse = viewport::mouse_position(d);
        let frame = inspector::frame();
        let ink = Color::new(236, 236, 240, 255);
        let dim = Color::new(150, 154, 168, 255);
        let accent = Color::new(242, 190, 92, 255);
        let small = TextStyle::new(FontRole::Menu, 13.0, dim);
        let text = TextStyle::new(FontRole::Menu, 14.0, ink);
        let muted = TextStyle::new(FontRole::Menu, 14.0, dim);

        let sharing = self
            .selected
            .as_ref()
            .map(|selected| {
                frame
                    .widgets
                    .iter()
                    .filter(|w| w.style.as_ref() == Some(selected))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        for widget in &sharing {
            d.draw_rectangle_lines_ex(widget.rect, 2.0, accent);
        }
        let panel = self.panel(screen);
        if !panel.check_collision_point_rec(mouse)
            && let Some(hover) = frame
                .widgets
                .iter()
                .rev()
                .find(|w| w.style.is_some() && w.rect.check_collision_point_rec(mouse))
        {
            d.draw_rectangle_lines_ex(hover.rect, 1.0, Color::new(90, 200, 255, 220));
        }

        d.draw_rectangle_rec(panel, Color::new(20, 21, 28, 235));
        d.draw_rectangle_lines_ex(panel, 1.0, Color::new(70, 74, 92, 255));
        let x = panel.x + PAD;
        let Some(selected) = &self.selected else {
            ui::draw_text(
                d,
                fonts,
                "Style editor",
                Vector2::new(x, panel.y + PAD),
                &TextStyle::new(FontRole::Menu, 18.0, ink),
            );
            ui::draw_text(
                d,
                fonts,
                "Click a button or panel to style it.",
                Vector2::new(x, panel.y + PAD + 30.0),
                &muted,
            );
            ui::draw_text(
                d,
                fonts,
                "Esc closes and puts everything back.",
                Vector2::new(x, panel.y + PAD + 52.0),
                &small,
            );
            return;
        };

        ui::draw_text(
            d,
            fonts,
            &self.label,
            Vector2::new(x, panel.y + PAD),
            &TextStyle::new(FontRole::Menu, 16.0, accent),
        );
        let share = match sharing.len() {
            0 | 1 => format!("this {} only", describe(selected)),
            n => format!("{n} {}s share this style", describe(selected)),
        };
        ui::draw_text(
            d,
            fonts,
            &share,
            Vector2::new(x, panel.y + PAD + 24.0),
            &small,
        );

        let list = Self::list(panel);
        let rows = Self::rows(list);
        let current = self.edit.apply(selected);
        let knobs = self.knobs();
        for (index, knob) in knobs.iter().enumerate().skip(self.top).take(rows) {
            let top = list.y + ROW * (index - self.top) as f32;
            let on = index == self.row;
            if on {
                d.draw_rectangle_rec(
                    Rectangle::new(list.x - 6.0, top, list.width + 12.0, ROW),
                    Color::new(242, 190, 92, 40),
                );
            }
            ui::draw_text(
                d,
                fonts,
                &knob.label(),
                Vector2::new(list.x, top + 3.0),
                if on { &text } else { &muted },
            );
            let value = match Edit::value(&current, *knob) {
                Some(v) if matches!(knob, Knob::Roundness) => format!("{v:.2}"),
                Some(v) => format!("{v:.0}"),
                None => "custom".to_string(),
            };
            let changed = Edit::value(&current, *knob) != Edit::value(selected, *knob);
            let shown = if on { format!("< {value} >") } else { value };
            let colour = if changed {
                Color::new(130, 210, 160, 255)
            } else {
                ink
            };
            let style = TextStyle::new(FontRole::Menu, 14.0, colour);
            let size = ui::measure_text(fonts, &shown, &style);
            ui::draw_text(
                d,
                fonts,
                &shown,
                Vector2::new(list.x + list.width - size.x, top + 3.0),
                &style,
            );
            let swatch = match knob {
                Knob::Text(Channel::R) => match &current {
                    Styled::Button(b) => Some(b.text.color),
                    Styled::Panel(_) => None,
                },
                Knob::Fill(Channel::R) => Some(fill_of(&current)),
                Knob::Border(Channel::R) => border_of(&current).map(|(_, c)| c),
                _ => None,
            };
            if let Some(colour) = swatch {
                let square = Rectangle::new(list.x + 110.0, top + 4.0, 14.0, 14.0);
                d.draw_rectangle_rec(square, colour);
                d.draw_rectangle_lines_ex(square, 1.0, Color::new(90, 94, 110, 255));
            }
        }

        let mut y = panel.y + panel.height - PAD - 104.0;
        let calls = self.edit.calls();
        let intro = if calls.is_empty() {
            "Nothing changed yet."
        } else {
            "Add to the chain that builds this style."
        };
        ui::draw_text(d, fonts, intro, Vector2::new(x, y), &small);
        if self.edit.size.is_some() {
            y += 16.0;
            ui::draw_text(
                d,
                fonts,
                "Size grows in place here; the layout reflows it once pasted.",
                Vector2::new(x, y),
                &small,
            );
        }
        for call in &calls {
            y += 16.0;
            if y > panel.y + panel.height - PAD - 36.0 {
                break;
            }
            ui::draw_text(
                d,
                fonts,
                call,
                Vector2::new(x, y),
                &TextStyle::new(FontRole::Menu, 13.0, accent),
            );
        }
        let hint = if self.copied {
            "Copied.   R resets   Esc closes"
        } else {
            "Left/Right or wheel change   Shift x10   Enter copies   R resets"
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
