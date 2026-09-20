use std::rc::Rc;

use raylib::prelude::*;

use crate::input::hit::{Highlight, LabelStyle, Shape};
use crate::{Action, DrawContext, Focus, GameContext, GameView, ResourceManager, ScreenState};

type EnabledCheck = Rc<dyn Fn(&GameView) -> bool>;

#[derive(Clone)]
pub struct Hotspot {
    pub id: String,
    pub shape: Shape,
    pub label: Option<String>,
    pub tooltip: Option<String>,
    pub action: Option<Action>,
    idle: Option<Highlight>,
    hovered: Option<Highlight>,
    enabled: Option<EnabledCheck>,
}

impl Hotspot {
    pub fn new(id: impl Into<String>, shape: Shape) -> Self {
        Self {
            id: id.into(),
            shape,
            label: None,
            tooltip: None,
            action: None,
            idle: None,
            hovered: None,
            enabled: None,
        }
    }

    pub fn label(mut self, text: impl Into<String>) -> Self {
        self.label = Some(text.into());
        self
    }

    pub fn tooltip(mut self, text: impl Into<String>) -> Self {
        self.tooltip = Some(text.into());
        self
    }

    pub fn action(mut self, action: Action) -> Self {
        self.action = Some(action);
        self
    }

    pub fn look(mut self, look: Highlight) -> Self {
        self.idle = Some(look);
        self
    }

    pub fn hover_look(mut self, look: Highlight) -> Self {
        self.hovered = Some(look);
        self
    }

    pub fn enabled_if(mut self, check: impl Fn(&GameView) -> bool + 'static) -> Self {
        self.enabled = Some(Rc::new(check));
        self
    }

    pub fn is_enabled(&self, view: &GameView) -> bool {
        self.enabled.as_ref().is_none_or(|check| check(view))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ImageMapStyle {
    pub idle: Highlight,
    pub hovered: Highlight,
    pub focused: Highlight,
    pub disabled: Highlight,
    pub label: LabelStyle,
    pub hover_sound: Option<String>,
    pub click_sound: Option<String>,
}

impl Default for ImageMapStyle {
    fn default() -> Self {
        Self {
            idle: Highlight::new(),
            hovered: Highlight::new()
                .fill(Color::new(255, 255, 255, 40))
                .border(2.0, Color::new(255, 255, 255, 200)),
            focused: Highlight::new().border(2.0, Color::new(255, 255, 255, 230)),
            disabled: Highlight::new().opacity(0.35),
            label: LabelStyle::default(),
            hover_sound: None,
            click_sound: None,
        }
    }
}

impl ImageMapStyle {
    pub fn idle(mut self, look: Highlight) -> Self {
        self.idle = look;
        self
    }

    pub fn hovered(mut self, look: Highlight) -> Self {
        self.hovered = look;
        self
    }

    pub fn focused(mut self, look: Highlight) -> Self {
        self.focused = look;
        self
    }

    pub fn disabled(mut self, look: Highlight) -> Self {
        self.disabled = look;
        self
    }

    pub fn label(mut self, label: LabelStyle) -> Self {
        self.label = label;
        self
    }

    pub fn hover_sound(mut self, id: impl Into<String>) -> Self {
        self.hover_sound = Some(id.into());
        self
    }

    pub fn click_sound(mut self, id: impl Into<String>) -> Self {
        self.click_sound = Some(id.into());
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct HotspotPick {
    pub index: usize,
    pub id: String,
    pub screen: Option<ScreenState>,
}

#[derive(Clone, Default)]
pub struct ImageMap {
    pub background: Option<String>,
    pub hotspots: Vec<Hotspot>,
    pub style: ImageMapStyle,
    focus: Focus,
    pressed: Option<usize>,
    entered: Option<usize>,
}

impl ImageMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn background(mut self, path: impl Into<String>) -> Self {
        self.background = Some(path.into());
        self
    }

    pub fn hotspot(mut self, hotspot: Hotspot) -> Self {
        self.hotspots.push(hotspot);
        self
    }

    pub fn style(mut self, style: impl FnOnce(ImageMapStyle) -> ImageMapStyle) -> Self {
        self.style = style(self.style);
        self
    }

    pub fn find(&self, id: &str) -> Option<usize> {
        self.hotspots.iter().position(|hotspot| hotspot.id == id)
    }

    pub fn focused(&self) -> Option<usize> {
        self.focus.index()
    }

    pub fn frame(&self, resources: &ResourceManager, area: Rectangle) -> Rectangle {
        let size = self
            .background
            .as_ref()
            .and_then(|path| resources.texture(path))
            .map(|texture| Vector2::new(texture.width as f32, texture.height as f32));
        match size {
            Some(size) => crate::ui::cover_rect(size, area),
            None => area,
        }
    }

    pub fn hovered_at(&self, area: Rectangle, point: Vector2) -> Option<usize> {
        crate::input::hit::pick(
            self.hotspots.iter().map(|hotspot| &hotspot.shape),
            area,
            point,
        )
    }

    pub fn load(&self, ctx: &mut GameContext) {
        let images = self
            .background
            .iter()
            .map(String::as_str)
            .chain(self.looks().filter_map(|look| look.image.as_deref()));
        for path in images.collect::<Vec<&str>>() {
            ctx.resources.get_or_load(path, ctx.rl, ctx.thread);
        }
    }

    fn looks(&self) -> impl Iterator<Item = &Highlight> {
        [
            &self.style.idle,
            &self.style.hovered,
            &self.style.focused,
            &self.style.disabled,
        ]
        .into_iter()
        .chain(
            self.hotspots
                .iter()
                .flat_map(|hotspot| [hotspot.idle.as_ref(), hotspot.hovered.as_ref()])
                .flatten(),
        )
    }

    fn topmost(&self, area: Rectangle, point: Vector2, enabled: &[bool]) -> Option<usize> {
        (0..self.hotspots.len())
            .rfind(|&index| enabled[index] && self.hotspots[index].shape.contains(area, point))
    }

    fn look(&self, index: usize, enabled: bool, hovered: bool, focused: bool) -> Highlight {
        let hotspot = &self.hotspots[index];
        let idle = match &hotspot.idle {
            Some(look) => look.over(&self.style.idle),
            None => self.style.idle.clone(),
        };
        if !enabled {
            return self.style.disabled.over(&idle);
        }

        let mut look = idle;
        if focused {
            look = self.style.focused.over(&look);
        }
        if hovered {
            let hover = match &hotspot.hovered {
                Some(look) => look.over(&self.style.hovered),
                None => self.style.hovered.clone(),
            };
            look = hover.over(&look);
        }
        look
    }

    pub fn update(&mut self, ctx: &mut GameContext, area: Rectangle) -> Option<HotspotPick> {
        self.load(ctx);
        let area = self.frame(ctx.resources, area);

        let view = ctx.view();
        let enabled: Vec<bool> = self
            .hotspots
            .iter()
            .map(|hotspot| hotspot.is_enabled(&view))
            .collect();
        let pointer = crate::frame::viewport::mouse_position(ctx.rl);
        let hovered = self.topmost(area, pointer, &enabled);

        if hovered != self.entered {
            self.entered = hovered;
            if hovered.is_some()
                && let Some(sound) = self.style.hover_sound.clone()
            {
                ctx.play_sound(&sound);
            }
        }
        if let Some(index) = hovered
            && let Some(text) = &self.hotspots[index].tooltip
        {
            *ctx.tooltip = Some(text.clone());
        }

        if ctx
            .rl
            .is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT)
        {
            self.pressed = hovered;
        }
        let mut clicked = None;
        if ctx
            .rl
            .is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT)
        {
            clicked = self.pressed.filter(|&index| hovered == Some(index));
            self.pressed = None;
        }

        let rects: Vec<Rectangle> = self
            .hotspots
            .iter()
            .map(|hotspot| hotspot.shape.bounds(area))
            .collect();
        let accepted = self.focus.update(&ctx.nav, &rects, &enabled, hovered);
        let index = clicked.or(accepted)?;

        if let Some(sound) = self.style.click_sound.clone() {
            ctx.play_sound(&sound);
        }
        let action = self.hotspots[index].action.clone();
        Some(HotspotPick {
            index,
            id: self.hotspots[index].id.clone(),
            screen: action.and_then(|action| action.run(ctx)),
        })
    }

    pub fn draw(&self, d: &mut RaylibDrawHandle, ctx: &DrawContext, area: Rectangle) {
        if let Some(path) = &self.background
            && let Some(texture) = ctx.resources.texture(path)
        {
            crate::ui::draw_texture_cover(d, texture, area);
        }
        let area = self.frame(ctx.resources, area);

        let view = ctx.view();
        let enabled: Vec<bool> = self
            .hotspots
            .iter()
            .map(|hotspot| hotspot.is_enabled(&view))
            .collect();
        let pointer = crate::frame::viewport::mouse_position(d);
        let hovered = match ctx.interactive {
            true => self.topmost(area, pointer, &enabled),
            false => None,
        };

        for (index, hotspot) in self.hotspots.iter().enumerate() {
            let focused = ctx.shows_focus(&self.focus, index);
            let look = self.look(index, enabled[index], hovered == Some(index), focused);
            look.draw(d, ctx, &hotspot.shape, area);
        }

        let labelled = hovered.or_else(|| match ctx.focus_visible && ctx.interactive {
            true => self
                .focus
                .index()
                .filter(|&index| enabled.get(index).copied().unwrap_or(false)),
            false => None,
        });
        if let Some(index) = labelled
            && let Some(text) = &self.hotspots[index].label
        {
            let bounds = self.hotspots[index].shape.bounds(area);
            self.style.label.draw(d, ctx, bounds, pointer, text);
        }
    }
}
