use novn_script::{StoryVm, Value, VarType, VmError};
use raylib::prelude::*;

use crate::context::{DrawContext, GameContext};
use crate::frame::viewport;
use crate::input::navigation::NavInput;
use crate::overlay::{Overlay, OverlayAction};
use crate::screen::ScreenState;
use crate::ui;
use crate::ui::TextStyle;
use crate::ui::fonts::FontRole;

pub const SCENE_JUMP_OVERLAY: &str = "dev.scene_jump";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Panel {
    Scenes,
    Variables,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Step {
    Stay,
    Close,
    Jump,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SceneRow {
    pub id: String,
    pub place: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VariableRow {
    pub id: String,
    pub ty: VarType,
    pub value: Value,
    pub changed: bool,
}

impl VariableRow {
    pub fn editable(&self) -> bool {
        !matches!(self.ty, VarType::String)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SceneJump {
    pub scenes: Vec<SceneRow>,
    pub variables: Vec<VariableRow>,
    pub panel: Panel,
    pub scene: usize,
    pub variable: usize,
    pub scene_top: usize,
    pub variable_top: usize,
}

impl SceneJump {
    pub fn from_story(story: &StoryVm) -> Self {
        let program = story.program();
        let scenes: Vec<SceneRow> = program
            .scene_order
            .iter()
            .map(|id| {
                let place = program
                    .scene_locations
                    .get(id)
                    .map(|at| match program.files.get(at.file) {
                        Some(file) => format!("{}:{}", short(file), at.line),
                        None => format!("line {}", at.line),
                    })
                    .unwrap_or_default();
                SceneRow {
                    id: id.clone(),
                    place,
                }
            })
            .collect();

        let mut variables: Vec<VariableRow> = story
            .variables()
            .iter()
            .filter_map(|(id, value)| {
                Some(VariableRow {
                    id: id.clone(),
                    ty: story.variable_type(id)?.clone(),
                    value: value.clone(),
                    changed: false,
                })
            })
            .collect();
        variables.sort_by(|a, b| a.id.cmp(&b.id));

        let scene = story
            .current_scene()
            .and_then(|current| scenes.iter().position(|row| row.id == current))
            .unwrap_or(0);

        Self {
            scenes,
            variables,
            panel: Panel::Scenes,
            scene,
            variable: 0,
            scene_top: 0,
            variable_top: 0,
        }
    }

    pub fn centre(&mut self, scene_rows: usize, variable_rows: usize) {
        self.scene_top = centred(self.scene, self.scenes.len(), scene_rows);
        self.variable_top = centred(self.variable, self.variables.len(), variable_rows);
    }

    pub fn follow(&mut self, scene_rows: usize, variable_rows: usize) {
        self.scene_top = followed(self.scene_top, self.scene, self.scenes.len(), scene_rows);
        self.variable_top = followed(
            self.variable_top,
            self.variable,
            self.variables.len(),
            variable_rows,
        );
    }

    pub fn selected_scene(&self) -> Option<&str> {
        self.scenes.get(self.scene).map(|row| row.id.as_str())
    }

    pub fn step(&mut self, nav: &NavInput) -> Step {
        if nav.back {
            return Step::Close;
        }
        if nav.next || nav.previous {
            self.switch_panel();
        }
        match self.panel {
            Panel::Scenes => {
                if nav.up {
                    self.scene = self.scene.saturating_sub(1);
                }
                if nav.down && self.scene + 1 < self.scenes.len() {
                    self.scene += 1;
                }
                if nav.right && !self.variables.is_empty() {
                    self.panel = Panel::Variables;
                }
            }
            Panel::Variables => {
                if nav.up {
                    self.variable = self.variable.saturating_sub(1);
                }
                if nav.down && self.variable + 1 < self.variables.len() {
                    self.variable += 1;
                }
                if nav.left {
                    self.adjust(-1);
                }
                if nav.right {
                    self.adjust(1);
                }
            }
        }
        if nav.accept && !self.scenes.is_empty() {
            return Step::Jump;
        }
        Step::Stay
    }

    pub fn switch_panel(&mut self) {
        self.panel = match self.panel {
            Panel::Scenes if !self.variables.is_empty() => Panel::Variables,
            _ => Panel::Scenes,
        };
    }

    pub fn adjust(&mut self, by: i32) {
        let Some(row) = self.variables.get_mut(self.variable) else {
            return;
        };
        let next = match (&row.ty, &row.value) {
            (VarType::Bool, Value::Bool(on)) => Value::Bool(!on),
            (VarType::Int, Value::Int(n)) => Value::Int(n.saturating_add(by)),
            (VarType::Enum(members), Value::Enum(current)) if !members.is_empty() => {
                let at = members.iter().position(|m| m == current).unwrap_or(0) as i32;
                let len = members.len() as i32;
                Value::Enum(members[(at + by).rem_euclid(len) as usize].clone())
            }
            _ => return,
        };
        row.value = next;
        row.changed = true;
    }

    pub fn apply(&self, story: &mut StoryVm) -> Result<(), VmError> {
        let Some(scene) = self.selected_scene() else {
            return Ok(());
        };
        story.start_at(scene)?;
        for row in &self.variables {
            story.set_variable(row.id.clone(), row.value.clone())?;
        }
        Ok(())
    }
}

fn short(file: &str) -> &str {
    file.rsplit(['/', '\\']).next().unwrap_or(file)
}

const MARGIN: f32 = 24.0;
const PADDING: f32 = 18.0;
const ROW: f32 = 26.0;
const TITLE: f32 = 20.0;
const TEXT: f32 = 16.0;
const SMALL: f32 = 13.0;

#[derive(Clone, Copy, Debug)]
struct Layout {
    panel: Rectangle,
    scenes: Rectangle,
    variables: Rectangle,
}

impl Layout {
    fn of(screen: Vector2) -> Self {
        let width = (screen.x - MARGIN * 2.0).min(1040.0);
        let height = (screen.y - MARGIN * 2.0).min(720.0);
        let panel = Rectangle::new(
            (screen.x - width) / 2.0,
            (screen.y - height) / 2.0,
            width,
            height,
        );
        let top = panel.y + PADDING + TITLE * 1.5 + SMALL * 2.2;
        let bottom = panel.y + panel.height - PADDING - SMALL * 1.8;
        let inner = panel.width - PADDING * 3.0;
        let left = inner * 0.55;
        Self {
            panel,
            scenes: Rectangle::new(panel.x + PADDING, top, left, bottom - top),
            variables: Rectangle::new(
                panel.x + PADDING * 2.0 + left,
                top,
                inner - left,
                bottom - top,
            ),
        }
    }

    fn rows(area: Rectangle) -> usize {
        ((area.height - ROW) / ROW).floor().max(1.0) as usize
    }
}

pub fn centred(selected: usize, total: usize, rows: usize) -> usize {
    if total <= rows {
        0
    } else {
        selected.saturating_sub(rows / 2).min(total - rows)
    }
}

pub fn followed(top: usize, selected: usize, total: usize, rows: usize) -> usize {
    if total <= rows || rows == 0 {
        return 0;
    }
    let top = if selected < top {
        selected
    } else if selected >= top + rows {
        selected + 1 - rows
    } else {
        top
    };
    top.min(total - rows)
}

fn row_at(area: Rectangle, point: Vector2, first: usize, total: usize) -> Option<usize> {
    let list = Rectangle::new(area.x, area.y + ROW, area.width, area.height - ROW);
    if !list.check_collision_point_rec(point) {
        return None;
    }
    let at = first + ((point.y - list.y) / ROW) as usize;
    (at < total).then_some(at)
}

pub struct SceneJumpOverlay {
    jump: Option<SceneJump>,
}

impl SceneJumpOverlay {
    pub fn new() -> Self {
        Self { jump: None }
    }

    fn reset_for_jump(ctx: &mut GameContext) {
        #[cfg(feature = "character-visuals")]
        ctx.resources.visuals.reset();
        ctx.log.clear();
        *ctx.modes = crate::data::session::PlayModes::default();
        ctx.state.reset();
        ctx.rollback.clear();
    }
}

impl Default for SceneJumpOverlay {
    fn default() -> Self {
        Self::new()
    }
}

impl Overlay for SceneJumpOverlay {
    fn update(&mut self, mut ctx: GameContext) -> OverlayAction {
        let screen = ui::screen_size(ctx.rl);
        let layout = Layout::of(screen);
        let (scene_rows, variable_rows) =
            (Layout::rows(layout.scenes), Layout::rows(layout.variables));
        let jump = self.jump.get_or_insert_with(|| {
            let mut jump = SceneJump::from_story(ctx.story);
            jump.centre(scene_rows, variable_rows);
            jump
        });
        let mouse = viewport::mouse_position(ctx.rl);
        let clicked = ctx
            .rl
            .is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT);
        let mut nav = ctx.nav;
        nav.back |= ctx.rl.is_key_pressed(KeyboardKey::KEY_ESCAPE);
        let mut step = jump.step(&nav);

        if clicked {
            if let Some(at) = row_at(layout.scenes, mouse, jump.scene_top, jump.scenes.len()) {
                if jump.panel == Panel::Scenes && at == jump.scene {
                    step = Step::Jump;
                }
                jump.panel = Panel::Scenes;
                jump.scene = at;
            }
            if let Some(at) = row_at(
                layout.variables,
                mouse,
                jump.variable_top,
                jump.variables.len(),
            ) {
                jump.panel = Panel::Variables;
                jump.variable = at;
                let half = layout.variables.x + layout.variables.width * 0.5;
                jump.adjust(if mouse.x < half { -1 } else { 1 });
            }
            if !layout.panel.check_collision_point_rec(mouse) {
                step = Step::Close;
            }
        }

        jump.follow(scene_rows, variable_rows);

        match step {
            Step::Stay => OverlayAction::Stay,
            Step::Close => OverlayAction::Close,
            Step::Jump => {
                let chosen = jump.clone();
                match chosen.apply(ctx.story) {
                    Ok(()) => {
                        Self::reset_for_jump(&mut ctx);
                        OverlayAction::Goto(ScreenState::Playing)
                    }
                    Err(error) => {
                        eprintln!("⚠️ Scene jump failed: {:?}", error);
                        ctx.notify_error(format!("Could not jump: {:?}", error));
                        OverlayAction::Stay
                    }
                }
            }
        }
    }

    fn draw(&self, d: &mut RaylibDrawHandle, ctx: &DrawContext) {
        let fonts = ctx.fonts();
        let screen = ui::screen_size(d);
        let layout = Layout::of(screen);
        let jump = match &self.jump {
            Some(jump) => jump.clone(),
            None => {
                let mut jump = SceneJump::from_story(ctx.story);
                jump.centre(Layout::rows(layout.scenes), Layout::rows(layout.variables));
                jump
            }
        };

        let ink = Color::new(236, 236, 240, 255);
        let dim = Color::new(150, 154, 168, 255);
        let accent = Color::new(242, 190, 92, 255);
        let changed = Color::new(130, 210, 160, 255);
        let title = TextStyle::new(FontRole::Menu, TITLE, ink);
        let text = TextStyle::new(FontRole::Menu, TEXT, ink);
        let muted = TextStyle::new(FontRole::Menu, TEXT, dim);
        let small = TextStyle::new(FontRole::Menu, SMALL, dim);

        d.draw_rectangle_v(Vector2::zero(), screen, Color::new(8, 8, 12, 190));
        d.draw_rectangle_rec(layout.panel, Color::new(20, 21, 28, 250));
        d.draw_rectangle_lines_ex(layout.panel, 1.0, Color::new(70, 74, 92, 255));

        let x = layout.panel.x + PADDING;
        let mut y = layout.panel.y + PADDING;
        ui::draw_text(d, fonts, "Jump to a scene", Vector2::new(x, y), &title);
        y += TITLE * 1.5;
        ui::draw_text(
            d,
            fonts,
            "Up/Down choose   Tab switches lists   Left/Right change a value   Enter or click twice to jump   Esc closes",
            Vector2::new(x, y),
            &small,
        );

        let heading = |d: &mut RaylibDrawHandle, area: Rectangle, label: &str, on: bool| {
            let style = if on {
                TextStyle::new(FontRole::Menu, SMALL, accent)
            } else {
                small.clone()
            };
            ui::draw_text(d, fonts, label, Vector2::new(area.x, area.y + 4.0), &style);
        };

        let rows = Layout::rows(layout.scenes);
        let first = jump.scene_top;
        let count = if jump.scenes.len() > rows {
            format!(
                "SCENES  {}-{} of {}",
                first + 1,
                (first + rows).min(jump.scenes.len()),
                jump.scenes.len()
            )
        } else {
            format!("SCENES  {}", jump.scenes.len())
        };
        heading(d, layout.scenes, &count, jump.panel == Panel::Scenes);
        for (offset, row) in jump.scenes.iter().enumerate().skip(first).take(rows) {
            let top = layout.scenes.y + ROW * (1 + offset - first) as f32;
            let selected = offset == jump.scene;
            if selected {
                let band =
                    Rectangle::new(layout.scenes.x - 6.0, top, layout.scenes.width + 6.0, ROW);
                let fill = if jump.panel == Panel::Scenes {
                    Color::new(242, 190, 92, 40)
                } else {
                    Color::new(255, 255, 255, 14)
                };
                d.draw_rectangle_rec(band, fill);
            }
            let style = if selected { &text } else { &muted };
            ui::draw_text(
                d,
                fonts,
                &row.id,
                Vector2::new(layout.scenes.x, top + 4.0),
                style,
            );
            let place = ui::measure_text(fonts, &row.place, &small);
            ui::draw_text(
                d,
                fonts,
                &row.place,
                Vector2::new(
                    layout.scenes.x + layout.scenes.width - place.x - 6.0,
                    top + 6.0,
                ),
                &small,
            );
        }

        heading(
            d,
            layout.variables,
            &format!("VARIABLES  {}", jump.variables.len()),
            jump.panel == Panel::Variables,
        );
        if jump.variables.is_empty() {
            ui::draw_text(
                d,
                fonts,
                "none registered",
                Vector2::new(layout.variables.x, layout.variables.y + ROW + 4.0),
                &muted,
            );
        }
        let rows = Layout::rows(layout.variables);
        let first = jump.variable_top;
        for (offset, row) in jump.variables.iter().enumerate().skip(first).take(rows) {
            let top = layout.variables.y + ROW * (1 + offset - first) as f32;
            let selected = offset == jump.variable && jump.panel == Panel::Variables;
            if selected {
                let band = Rectangle::new(
                    layout.variables.x - 6.0,
                    top,
                    layout.variables.width + 6.0,
                    ROW,
                );
                d.draw_rectangle_rec(band, Color::new(242, 190, 92, 40));
            }
            let name = if row.changed { &text } else { &muted };
            ui::draw_text(
                d,
                fonts,
                &row.id,
                Vector2::new(layout.variables.x, top + 4.0),
                name,
            );

            let shown = match &row.value {
                novn_script::Value::String(s) => format!("\"{s}\""),
                other => other.to_string(),
            };
            let colour = if !row.editable() {
                dim
            } else if row.changed {
                changed
            } else {
                ink
            };
            let value = TextStyle::new(FontRole::Menu, TEXT, colour);
            let label = if selected && row.editable() {
                format!("< {shown} >")
            } else {
                shown
            };
            let size = ui::measure_text(fonts, &label, &value);
            ui::draw_text(
                d,
                fonts,
                &label,
                Vector2::new(
                    layout.variables.x + layout.variables.width - size.x - 6.0,
                    top + 4.0,
                ),
                &value,
            );
        }

        let note = match jump.selected_scene() {
            Some(scene) => format!(
                "Enter starts '{scene}' with these values. Game state, the log and rollback start over, as with New Game."
            ),
            None => "This story has no scenes.".to_string(),
        };
        ui::draw_text(
            d,
            fonts,
            &note,
            Vector2::new(x, layout.panel.y + layout.panel.height - PADDING - SMALL),
            &small,
        );
    }
}
