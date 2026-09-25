use crate::ui::cursor::CursorKind;
use std::rc::Rc;

use raylib::prelude::*;

use crate::context::{DrawContext, GameContext, GameView};
use crate::input::hit::{Highlight, LabelStyle, Shape};
use crate::input::navigation::Focus;

type EnabledCheck = Rc<dyn Fn(&GameView) -> bool>;
type DropRule = Rc<dyn Fn(&str, &GameView) -> bool>;

#[derive(Clone)]
pub struct Draggable {
    pub id: String,
    pub shape: Shape,
    pub label: Option<String>,
    look: Option<Highlight>,
    enabled: Option<EnabledCheck>,
}

impl Draggable {
    pub fn new(id: impl Into<String>, shape: Shape) -> Self {
        Self {
            id: id.into(),
            shape,
            label: None,
            look: None,
            enabled: None,
        }
    }

    pub fn label(mut self, text: impl Into<String>) -> Self {
        self.label = Some(text.into());
        self
    }

    pub fn image(mut self, path: impl Into<String>) -> Self {
        let look = self.look.unwrap_or_default().image(path);
        self.look = Some(look);
        self
    }

    pub fn look(mut self, look: Highlight) -> Self {
        self.look = Some(look);
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

#[derive(Clone)]
pub struct DropTarget {
    pub id: String,
    pub shape: Shape,
    pub label: Option<String>,
    look: Option<Highlight>,
    accepts: Option<DropRule>,
}

impl DropTarget {
    pub fn new(id: impl Into<String>, shape: Shape) -> Self {
        Self {
            id: id.into(),
            shape,
            label: None,
            look: None,
            accepts: None,
        }
    }

    pub fn label(mut self, text: impl Into<String>) -> Self {
        self.label = Some(text.into());
        self
    }

    pub fn look(mut self, look: Highlight) -> Self {
        self.look = Some(look);
        self
    }

    pub fn accepts(mut self, rule: impl Fn(&str, &GameView) -> bool + 'static) -> Self {
        self.accepts = Some(Rc::new(rule));
        self
    }

    pub fn allows(&self, item: &str, view: &GameView) -> bool {
        self.accepts.as_ref().is_none_or(|rule| rule(item, view))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DragStyle {
    pub item: Highlight,
    pub item_hovered: Highlight,
    pub item_held: Highlight,
    pub target: Highlight,
    pub target_ready: Highlight,
    pub target_blocked: Highlight,
    pub target_hovered: Highlight,
    pub label: LabelStyle,
    pub pick_sound: Option<String>,
    pub drop_sound: Option<String>,
    pub reject_sound: Option<String>,
}

impl Default for DragStyle {
    fn default() -> Self {
        Self {
            item: Highlight::new(),
            item_hovered: Highlight::new().border(2.0, Color::new(255, 255, 255, 200)),
            item_held: Highlight::new().opacity(0.85),
            target: Highlight::new().border(2.0, Color::new(255, 255, 255, 60)),
            target_ready: Highlight::new().border(2.0, Color::new(180, 230, 180, 200)),
            target_blocked: Highlight::new().border(2.0, Color::new(230, 160, 160, 140)),
            target_hovered: Highlight::new().fill(Color::new(255, 255, 255, 40)),
            label: LabelStyle::default(),
            pick_sound: None,
            drop_sound: None,
            reject_sound: None,
        }
    }
}

impl DragStyle {
    pub fn item(mut self, look: Highlight) -> Self {
        self.item = look;
        self
    }

    pub fn item_hovered(mut self, look: Highlight) -> Self {
        self.item_hovered = look;
        self
    }

    pub fn item_held(mut self, look: Highlight) -> Self {
        self.item_held = look;
        self
    }

    pub fn target(mut self, look: Highlight) -> Self {
        self.target = look;
        self
    }

    pub fn target_ready(mut self, look: Highlight) -> Self {
        self.target_ready = look;
        self
    }

    pub fn target_blocked(mut self, look: Highlight) -> Self {
        self.target_blocked = look;
        self
    }

    pub fn target_hovered(mut self, look: Highlight) -> Self {
        self.target_hovered = look;
        self
    }

    pub fn label(mut self, label: LabelStyle) -> Self {
        self.label = label;
        self
    }

    pub fn sounds(
        mut self,
        pick: impl Into<String>,
        drop: impl Into<String>,
        reject: impl Into<String>,
    ) -> Self {
        self.pick_sound = Some(pick.into());
        self.drop_sound = Some(drop.into());
        self.reject_sound = Some(reject.into());
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Held {
    item: usize,
    from: Vector2,
    to: Vector2,
    keyboard: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Dropped {
    pub item: usize,
    pub item_id: String,
    pub target: Option<usize>,
    pub target_id: Option<String>,
    pub accepted: bool,
    pub point: Vector2,
}

#[derive(Clone, Default)]
pub struct DragBoard {
    pub items: Vec<Draggable>,
    pub targets: Vec<DropTarget>,
    pub style: DragStyle,
    held: Option<Held>,
    focus: Focus,
    target_focus: Focus,
}

impl DragBoard {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn item(mut self, item: Draggable) -> Self {
        self.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = Draggable>) -> Self {
        self.items = items.into_iter().collect();
        self
    }

    pub fn target(mut self, target: DropTarget) -> Self {
        self.targets.push(target);
        self
    }

    pub fn style(mut self, style: impl FnOnce(DragStyle) -> DragStyle) -> Self {
        self.style = style(self.style);
        self
    }

    pub fn set_items(&mut self, items: impl IntoIterator<Item = Draggable>) {
        self.items = items.into_iter().collect();
        if self.held.is_some_and(|held| held.item >= self.items.len()) {
            self.held = None;
        }
    }

    pub fn held(&self) -> Option<usize> {
        self.held.map(|held| held.item)
    }

    pub fn held_id(&self) -> Option<&str> {
        self.held.map(|held| self.items[held.item].id.as_str())
    }

    pub fn item_at(&self, area: Rectangle, point: Vector2) -> Option<usize> {
        crate::input::hit::pick(self.items.iter().map(|item| &item.shape), area, point)
    }

    pub fn target_at(&self, area: Rectangle, point: Vector2) -> Option<usize> {
        crate::input::hit::pick(self.targets.iter().map(|target| &target.shape), area, point)
    }

    pub fn offset(&self, item: usize) -> Vector2 {
        match self.held.filter(|held| held.item == item) {
            Some(held) => Vector2::new(held.to.x - held.from.x, held.to.y - held.from.y),
            None => Vector2::zero(),
        }
    }

    pub fn item_area(&self, item: usize, area: Rectangle) -> Rectangle {
        let offset = self.offset(item);
        Rectangle::new(
            area.x + offset.x,
            area.y + offset.y,
            area.width,
            area.height,
        )
    }

    pub fn item_rect(&self, item: usize, area: Rectangle) -> Rectangle {
        self.items[item].shape.bounds(self.item_area(item, area))
    }

    pub fn grab(&mut self, item: usize, pointer: Vector2, keyboard: bool) {
        if item < self.items.len() {
            self.held = Some(Held {
                item,
                from: pointer,
                to: pointer,
                keyboard,
            });
        }
    }

    pub fn drag(&mut self, pointer: Vector2) {
        if let Some(held) = &mut self.held {
            held.to = pointer;
        }
    }

    pub fn release(
        &mut self,
        area: Rectangle,
        pointer: Vector2,
        allowed: impl Fn(usize, usize) -> bool,
    ) -> Option<Dropped> {
        let held = self.held.take()?;
        let target = self.target_at(area, pointer);
        let accepted = target.is_some_and(|target| allowed(held.item, target));
        Some(Dropped {
            item: held.item,
            item_id: self.items[held.item].id.clone(),
            target,
            target_id: target.map(|target| self.targets[target].id.clone()),
            accepted,
            point: pointer,
        })
    }

    pub fn cancel(&mut self) -> Option<Dropped> {
        let held = self.held.take()?;
        Some(Dropped {
            item: held.item,
            item_id: self.items[held.item].id.clone(),
            target: None,
            target_id: None,
            accepted: false,
            point: held.to,
        })
    }

    pub fn load(&self, ctx: &mut GameContext) {
        let looks = [
            &self.style.item,
            &self.style.item_hovered,
            &self.style.item_held,
            &self.style.target,
            &self.style.target_ready,
            &self.style.target_blocked,
            &self.style.target_hovered,
        ]
        .into_iter()
        .chain(self.items.iter().filter_map(|item| item.look.as_ref()))
        .chain(
            self.targets
                .iter()
                .filter_map(|target| target.look.as_ref()),
        );

        let paths: Vec<&str> = looks.filter_map(|look| look.image.as_deref()).collect();
        for path in paths {
            ctx.resources.get_or_load(path, ctx.rl, ctx.thread);
        }
    }

    fn allowances(&self, item: usize, view: &GameView) -> Vec<bool> {
        let id = self.items[item].id.as_str();
        self.targets
            .iter()
            .map(|target| target.allows(id, view))
            .collect()
    }

    pub fn update(&mut self, ctx: &mut GameContext, area: Rectangle) -> Option<Dropped> {
        self.load(ctx);

        let pointer = crate::frame::viewport::mouse_position(ctx.rl);
        let released = ctx
            .rl
            .is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT);
        let pressed = ctx
            .rl
            .is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT);
        let cancelled = ctx
            .rl
            .is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_RIGHT)
            || ctx.rl.is_key_pressed(KeyboardKey::KEY_ESCAPE);

        match self.held {
            None => {
                self.pick_up(ctx, area, pointer, pressed);
                None
            }
            Some(held) if held.keyboard => self.hold_with_keys(ctx, area, held),
            Some(held) => {
                self.drag(pointer);
                if cancelled {
                    return self.cancel();
                }
                let refused = self
                    .target_at(area, pointer)
                    .is_some_and(|target| !self.allowances(held.item, &ctx.view())[target]);
                ctx.cursor(match refused {
                    true => CursorKind::NotAllowed,
                    false => CursorKind::Grabbing,
                });
                let holding =
                    !released && ctx.rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT);
                if holding {
                    return None;
                }
                let allowed = self.allowances(held.item, &ctx.view());
                let drop = self.release(area, pointer, |_, target| allowed[target]);
                self.played(ctx, drop)
            }
        }
    }

    fn pick_up(&mut self, ctx: &mut GameContext, area: Rectangle, pointer: Vector2, pressed: bool) {
        let view = ctx.view();
        let enabled: Vec<bool> = self
            .items
            .iter()
            .map(|item| item.is_enabled(&view))
            .collect();
        let under = self.item_at(area, pointer);
        let hovered = under.filter(|&index| enabled[index]);
        match (hovered, under) {
            (Some(_), _) => ctx.cursor(CursorKind::Grab),
            (None, Some(_)) => ctx.cursor(CursorKind::NotAllowed),
            (None, None) => {}
        }

        let rects: Vec<Rectangle> = self
            .items
            .iter()
            .map(|item| item.shape.bounds(area))
            .collect();
        let accepted = self.focus.update(&ctx.nav, &rects, &enabled, hovered);

        let grabbed = match (pressed, hovered, accepted) {
            (true, Some(index), _) => Some((index, pointer, false)),
            (_, _, Some(index)) => Some((index, self.items[index].shape.center(area), true)),
            _ => None,
        };
        let Some((index, from, keyboard)) = grabbed else {
            return;
        };

        self.grab(index, from, keyboard);
        self.target_focus.set(None);
        if let Some(sound) = self.style.pick_sound.clone() {
            ctx.play_sound(&sound);
        }
    }

    fn hold_with_keys(
        &mut self,
        ctx: &mut GameContext,
        area: Rectangle,
        held: Held,
    ) -> Option<Dropped> {
        if ctx.nav.back {
            return self.cancel();
        }

        let allowed = self.allowances(held.item, &ctx.view());
        let rects: Vec<Rectangle> = self
            .targets
            .iter()
            .map(|target| target.shape.bounds(area))
            .collect();
        let accepted = self.target_focus.update(&ctx.nav, &rects, &allowed, None);

        if let Some(index) = self.target_focus.index() {
            let center = self.targets[index].shape.center(area);
            self.drag(center);
        }

        let index = accepted?;
        let point = self.targets[index].shape.center(area);
        let drop = self.release(area, point, |_, target| allowed[target]);
        self.played(ctx, drop)
    }

    fn played(&mut self, ctx: &mut GameContext, drop: Option<Dropped>) -> Option<Dropped> {
        let drop = drop?;
        let sound = match drop.accepted {
            true => self.style.drop_sound.clone(),
            false => self.style.reject_sound.clone(),
        };
        if let Some(sound) = sound {
            ctx.play_sound(&sound);
        }
        Some(drop)
    }

    fn item_look(&self, index: usize, enabled: bool, hovered: bool, focused: bool) -> Highlight {
        let base = match &self.items[index].look {
            Some(look) => look.over(&self.style.item),
            None => self.style.item.clone(),
        };
        if !enabled {
            return Highlight::new().opacity(0.35).over(&base);
        }

        let mut look = base;
        if hovered || focused {
            look = self.style.item_hovered.over(&look);
        }
        if self.held() == Some(index) {
            look = self.style.item_held.over(&look);
        }
        look
    }

    fn target_look(&self, index: usize, ready: Option<bool>, hovered: bool) -> Highlight {
        let mut look = match &self.targets[index].look {
            Some(look) => look.over(&self.style.target),
            None => self.style.target.clone(),
        };
        look = match ready {
            Some(true) => self.style.target_ready.over(&look),
            Some(false) => self.style.target_blocked.over(&look),
            None => look,
        };
        if hovered {
            look = self.style.target_hovered.over(&look);
        }
        look
    }

    pub fn draw(&self, d: &mut RaylibDrawHandle, ctx: &DrawContext, area: Rectangle) {
        let view = ctx.view();
        let pointer = crate::frame::viewport::mouse_position(d);
        let enabled: Vec<bool> = self
            .items
            .iter()
            .map(|item| item.is_enabled(&view))
            .collect();

        let allowed = self
            .held()
            .map(|item| self.allowances(item, &view))
            .unwrap_or_default();
        let over_target = match self.held {
            Some(held) if !held.keyboard => self.target_at(area, pointer),
            Some(_) => self.target_focus.index(),
            None => None,
        };

        for index in 0..self.targets.len() {
            let ready = allowed.get(index).copied();
            let look = self.target_look(index, ready, over_target == Some(index));
            look.draw(d, ctx, &self.targets[index].shape, area);
        }

        let hovered = match ctx.interactive && self.held.is_none() {
            true => self.item_at(area, pointer).filter(|&index| enabled[index]),
            false => None,
        };
        let held = self.held();
        let order = (0..self.items.len())
            .filter(|&index| Some(index) != held)
            .chain(held);
        for index in order {
            let focused = self.held.is_none() && ctx.shows_focus(&self.focus, index);
            let look = self.item_look(index, enabled[index], hovered == Some(index), focused);
            look.draw(
                d,
                ctx,
                &self.items[index].shape,
                self.item_area(index, area),
            );
        }

        let label = match (held, hovered, over_target) {
            (Some(_), _, Some(target)) => self.targets[target]
                .label
                .as_ref()
                .map(|text| (self.targets[target].shape.bounds(area), text)),
            (None, Some(item), _) => self.items[item]
                .label
                .as_ref()
                .map(|text| (self.item_rect(item, area), text)),
            _ => None,
        };
        if let Some((bounds, text)) = label {
            self.style.label.draw(d, ctx, bounds, pointer, text);
        }
    }
}
