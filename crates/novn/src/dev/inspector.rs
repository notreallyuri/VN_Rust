use std::cell::{Cell, RefCell};
use std::collections::VecDeque;

use raylib::prelude::*;

use crate::ui;
use crate::ui::TextStyle;
use crate::ui::fonts::{FontRole, Fonts};

pub const SAMPLES: usize = 120;

thread_local! {
    static ACTIVE: Cell<bool> = const { Cell::new(false) };
    static FRAME: RefCell<Frame> = RefCell::new(Frame::default());
    static LAST: RefCell<Frame> = RefCell::new(Frame::default());
}

#[derive(Clone, Debug, PartialEq)]
pub enum Styled {
    Button(Box<crate::ui::button::ButtonStyle>),
    Panel(crate::ui::shape::PanelStyle),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Widget {
    pub kind: &'static str,
    pub rect: Rectangle,
    pub label: String,
    pub focused: bool,
    pub disabled: bool,
    pub details: Vec<(&'static str, String)>,
    pub style: Option<Styled>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct FocusList {
    pub rects: Vec<Rectangle>,
    pub enabled: Vec<bool>,
    pub focused: Option<usize>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Frame {
    pub widgets: Vec<Widget>,
    pub focus: Vec<FocusList>,
}

pub fn active() -> bool {
    ACTIVE.with(Cell::get)
}

pub fn set_active(on: bool) {
    if active() == on {
        return;
    }
    ACTIVE.with(|active| active.set(on));
    FRAME.with(|frame| *frame.borrow_mut() = Frame::default());
    LAST.with(|last| *last.borrow_mut() = Frame::default());
}

pub fn begin_frame() {
    if active() {
        let done = FRAME.with(|frame| std::mem::take(&mut *frame.borrow_mut()));
        LAST.with(|last| *last.borrow_mut() = done);
    }
}

pub fn last() -> Frame {
    LAST.with(|last| last.borrow().clone())
}

pub fn widget(widget: Widget) {
    if active() {
        FRAME.with(|frame| frame.borrow_mut().widgets.push(widget));
    }
}

pub fn focus_list(rects: &[Rectangle], enabled: &[bool], focused: Option<usize>) {
    if active() && !rects.is_empty() {
        FRAME.with(|frame| {
            frame.borrow_mut().focus.push(FocusList {
                rects: rects.to_vec(),
                enabled: enabled.to_vec(),
                focused,
            });
        });
    }
}

pub fn frame() -> Frame {
    FRAME.with(|frame| frame.borrow().clone())
}

pub fn under(widgets: &[Widget], point: Vector2) -> Option<&Widget> {
    widgets
        .iter()
        .rev()
        .find(|widget| widget.rect.check_collision_point_rec(point))
}

pub fn colour(color: Color) -> String {
    if color.a == 255 {
        format!("#{:02x}{:02x}{:02x}", color.r, color.g, color.b)
    } else {
        format!(
            "#{:02x}{:02x}{:02x}{:02x}",
            color.r, color.g, color.b, color.a
        )
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Timing {
    samples: VecDeque<f32>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Summary {
    pub fps: f32,
    pub average_ms: f32,
    pub worst_ms: f32,
}

impl Timing {
    pub fn push(&mut self, seconds: f32) {
        if !seconds.is_finite() || seconds <= 0.0 {
            return;
        }
        if self.samples.len() == SAMPLES {
            self.samples.pop_front();
        }
        self.samples.push_back(seconds);
    }

    pub fn samples(&self) -> impl Iterator<Item = f32> + '_ {
        self.samples.iter().copied()
    }

    pub fn summary(&self) -> Option<Summary> {
        if self.samples.is_empty() {
            return None;
        }
        let total: f32 = self.samples.iter().sum();
        let average = total / self.samples.len() as f32;
        let worst = self.samples.iter().copied().fold(0.0, f32::max);
        Some(Summary {
            fps: 1.0 / average,
            average_ms: average * 1000.0,
            worst_ms: worst * 1000.0,
        })
    }
}

pub const GRAPH: Vector2 = Vector2::new(240.0, 64.0);
const GRAPH_MARGIN: f32 = 8.0;
const GRAPH_CLEARANCE: f32 = 60.0;

pub fn graph_rect(screen: Vector2, mouse: Vector2) -> Rectangle {
    let right = Rectangle::new(
        screen.x - GRAPH.x - GRAPH_MARGIN,
        GRAPH_MARGIN,
        GRAPH.x,
        GRAPH.y,
    );
    let near = Rectangle::new(
        right.x - GRAPH_CLEARANCE,
        0.0,
        right.width + GRAPH_CLEARANCE + GRAPH_MARGIN,
        right.y + right.height + GRAPH_CLEARANCE,
    );
    if near.check_collision_point_rec(mouse) {
        Rectangle::new(GRAPH_MARGIN, GRAPH_MARGIN, GRAPH.x, GRAPH.y)
    } else {
        right
    }
}

const PAD: f32 = 10.0;
const TEXT: f32 = 14.0;
const OUTLINE: Color = Color::new(90, 200, 255, 110);
const HOVER: Color = Color::new(255, 214, 90, 255);
const FOCUS: Color = Color::new(140, 230, 170, 255);
const BACKDROP: Color = Color::new(14, 15, 20, 235);
const INK: Color = Color::new(236, 236, 240, 255);
const DIM: Color = Color::new(150, 154, 168, 255);

pub fn draw(
    d: &mut RaylibDrawHandle,
    fonts: &Fonts,
    frame: &Frame,
    timing: &Timing,
    mouse: Vector2,
) {
    let screen = ui::screen_size(d);
    let text = TextStyle::new(FontRole::Menu, TEXT, INK);
    let dim = TextStyle::new(FontRole::Menu, TEXT, DIM);
    let small = TextStyle::new(FontRole::Menu, 12.0, DIM);

    for widget in &frame.widgets {
        d.draw_rectangle_lines_ex(widget.rect, 1.0, OUTLINE);
    }

    for list in &frame.focus {
        for (index, rect) in list.rects.iter().enumerate() {
            let focused = list.focused == Some(index);
            let usable = list.enabled.get(index).copied().unwrap_or(true);
            let colour = if focused {
                FOCUS
            } else if usable {
                Color::new(140, 230, 170, 150)
            } else {
                Color::new(150, 154, 168, 120)
            };
            if focused {
                d.draw_rectangle_lines_ex(*rect, 2.0, FOCUS);
            }
            let label = format!("{}", index + 1);
            let size = ui::measure_text(fonts, &label, &small);
            let badge = Rectangle::new(rect.x, rect.y, size.x + 8.0, size.y + 2.0);
            d.draw_rectangle_rec(badge, Color::new(0, 0, 0, 200));
            ui::draw_text(
                d,
                fonts,
                &label,
                Vector2::new(badge.x + 4.0, badge.y + 1.0),
                &TextStyle::new(FontRole::Menu, 12.0, colour),
            );
        }
    }

    if let Some(widget) = under(&frame.widgets, mouse) {
        d.draw_rectangle_lines_ex(widget.rect, 2.0, HOVER);

        let mut lines: Vec<(String, TextStyle)> = Vec::new();
        let title = if widget.label.is_empty() {
            widget.kind.to_string()
        } else {
            format!("{} \"{}\"", widget.kind, widget.label)
        };
        lines.push((title, TextStyle::new(FontRole::Menu, TEXT, HOVER)));
        lines.push((
            format!(
                "rect  {:.0}, {:.0}   {:.0} x {:.0}",
                widget.rect.x, widget.rect.y, widget.rect.width, widget.rect.height
            ),
            text.clone(),
        ));
        let mut state = Vec::new();
        if widget.focused {
            state.push("focused");
        }
        if widget.disabled {
            state.push("disabled");
        }
        if !state.is_empty() {
            lines.push((format!("state  {}", state.join(", ")), text.clone()));
        }
        for (name, value) in &widget.details {
            lines.push((format!("{name}  {value}"), dim.clone()));
        }

        let width = lines
            .iter()
            .map(|(line, style)| ui::measure_text(fonts, line, style).x)
            .fold(0.0, f32::max)
            + PAD * 2.0;
        let height = lines.len() as f32 * TEXT * 1.45 + PAD * 2.0;
        let mut at = Vector2::new(mouse.x + 18.0, mouse.y + 18.0);
        if at.x + width > screen.x - 4.0 {
            at.x = (mouse.x - width - 12.0).max(4.0);
        }
        if at.y + height > screen.y - 4.0 {
            at.y = (mouse.y - height - 12.0).max(4.0);
        }
        let panel = Rectangle::new(at.x, at.y, width, height);
        d.draw_rectangle_rec(panel, BACKDROP);
        d.draw_rectangle_lines_ex(panel, 1.0, HOVER);
        let mut y = panel.y + PAD;
        for (line, style) in &lines {
            ui::draw_text(d, fonts, line, Vector2::new(panel.x + PAD, y), style);
            y += TEXT * 1.45;
        }
    }

    let graph = graph_rect(screen, mouse);
    d.draw_rectangle_rec(graph, BACKDROP);
    d.draw_rectangle_lines_ex(graph, 1.0, Color::new(70, 74, 92, 255));
    let heading = match timing.summary() {
        Some(s) => format!(
            "{:.0} fps   {:.1} ms   worst {:.1}",
            s.fps, s.average_ms, s.worst_ms
        ),
        None => "measuring".to_string(),
    };
    ui::draw_text(
        d,
        fonts,
        &heading,
        Vector2::new(graph.x + 8.0, graph.y + 6.0),
        &text,
    );

    let plot = Rectangle::new(
        graph.x + 8.0,
        graph.y + 28.0,
        graph.width - 16.0,
        graph.height - 34.0,
    );
    let budget = 1.0 / 60.0;
    let ceiling = timing.samples().fold(budget * 2.0, f32::max);
    let line_y = plot.y + plot.height - plot.height * (budget / ceiling);
    d.draw_line_v(
        Vector2::new(plot.x, line_y),
        Vector2::new(plot.x + plot.width, line_y),
        Color::new(140, 230, 170, 90),
    );
    let step = plot.width / (SAMPLES - 1) as f32;
    let mut previous: Option<Vector2> = None;
    for (index, sample) in timing.samples().enumerate() {
        let point = Vector2::new(
            plot.x + index as f32 * step,
            plot.y + plot.height - plot.height * (sample / ceiling).min(1.0),
        );
        let over = sample > budget * 1.5;
        if let Some(from) = previous {
            d.draw_line_v(
                from,
                point,
                if over {
                    Color::new(255, 120, 110, 255)
                } else {
                    Color::new(90, 200, 255, 255)
                },
            );
        }
        previous = Some(point);
    }

    let counts = format!(
        "F3  {} widgets   {} focus list{}",
        frame.widgets.len(),
        frame.focus.len(),
        if frame.focus.len() == 1 { "" } else { "s" }
    );
    let size = ui::measure_text(fonts, &counts, &small);
    ui::draw_text(
        d,
        fonts,
        &counts,
        Vector2::new(graph.x + graph.width - size.x, graph.y + graph.height + 4.0),
        &small,
    );
}
