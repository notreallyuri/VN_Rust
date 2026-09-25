use std::collections::{BTreeMap, HashMap};

use novn_script::{Event, Position, StoryVm, Transition, TransitionKind};
use raylib::prelude::*;

use crate::data::resources::{ResourceManager, background_path, character_path};
use crate::screens::playing::PlayingConfig;
use crate::ui;
use crate::ui::Background;
use crate::ui::ease::{Easing, lerp};

const OFFSCREEN_LEFT: f32 = -0.25;
const OFFSCREEN_RIGHT: f32 = 1.25;

#[derive(Clone, Debug, PartialEq)]
pub struct Placed {
    pub image: String,
    pub x: f32,
}

pub fn stage_layout(story: &StoryVm, config: &PlayingConfig) -> BTreeMap<String, Placed> {
    let mut characters: Vec<_> = story.active_characters().iter().collect();
    characters.sort();

    let unplaced = characters
        .iter()
        .filter(|(name, _)| story.position(name).is_none())
        .count();
    let mut next_unplaced = 0;

    characters
        .into_iter()
        .map(|(name, image)| {
            let x = match story.position(name) {
                Some(position) => config.position_x(position),
                None => {
                    next_unplaced += 1;
                    next_unplaced as f32 / (unplaced as f32 + 1.0)
                }
            };
            (
                name.clone(),
                Placed {
                    image: image.clone(),
                    x,
                },
            )
        })
        .collect()
}

#[derive(Clone, Debug, PartialEq)]
pub struct CharacterAnim {
    pub transition: Transition,
    pub start: f64,
    pub from_image: Option<String>,
    pub from_x: Option<f32>,
    pub leaving: Option<Placed>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BackgroundAnim {
    pub transition: Transition,
    pub start: f64,
    pub from: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Stage {
    shown: BTreeMap<String, Placed>,
    background: Option<String>,
    characters: HashMap<String, CharacterAnim>,
    backdrop: Option<BackgroundAnim>,
    synced: bool,
}

pub fn progress(start: f64, transition: &Transition, now: f64) -> f32 {
    let seconds = transition.seconds() as f64;
    if seconds <= 0.0 {
        return 1.0;
    }
    ((now - start) / seconds).clamp(0.0, 1.0) as f32
}

impl Stage {
    #[cfg(feature = "character-visuals")]
    pub fn visual_keys(&self, story: &StoryVm) -> Vec<crate::game::visuals::VisualKey> {
        use crate::game::visuals::VisualKey;
        let mut keys: std::collections::BTreeSet<_> = story
            .active_characters()
            .iter()
            .map(|(id, image)| VisualKey::new(id, image))
            .collect();
        for (id, animation) in &self.characters {
            if let Some(image) = &animation.from_image {
                keys.insert(VisualKey::new(id, image));
            }
            if let Some(placed) = &animation.leaving {
                keys.insert(VisualKey::new(id, &placed.image));
            }
        }
        keys.into_iter().collect()
    }

    pub fn is_synced(&self) -> bool {
        self.synced
    }

    pub fn sync(&mut self, story: &StoryVm, config: &PlayingConfig) {
        self.shown = stage_layout(story, config);
        self.background = story.background().map(str::to_string);
        self.synced = true;
    }

    pub fn reset(&mut self, story: &StoryVm, config: &PlayingConfig) {
        self.characters.clear();
        self.backdrop = None;
        self.sync(story, config);
    }

    pub fn is_animating(&self) -> bool {
        !self.characters.is_empty() || self.backdrop.is_some()
    }

    pub fn character(&self, id: &str) -> Option<&CharacterAnim> {
        self.characters.get(id)
    }

    pub fn background_anim(&self) -> Option<&BackgroundAnim> {
        self.backdrop.as_ref()
    }

    pub fn apply(&mut self, event: &Event, story: &StoryVm, config: &PlayingConfig, now: f64) {
        match event {
            Event::Show {
                character,
                image,
                transition,
                ..
            } => match transition {
                Some(transition) => {
                    let before = self.shown.get(character).cloned();
                    let after = stage_layout(story, config);
                    let x = after.get(character).map_or(0.5, |placed| placed.x);
                    let from_image = before
                        .as_ref()
                        .filter(|placed| placed.image != *image)
                        .map(|placed| placed.image.clone());
                    let from_x = before
                        .as_ref()
                        .filter(|placed| (placed.x - x).abs() > 0.001)
                        .map(|placed| placed.x);
                    let from_x = match (&before, transition.kind) {
                        (None, TransitionKind::SlideLeft) => Some(OFFSCREEN_RIGHT),
                        (None, TransitionKind::SlideRight) => Some(OFFSCREEN_LEFT),
                        _ => from_x,
                    };
                    let entering = before.is_none();
                    self.characters.insert(
                        character.clone(),
                        CharacterAnim {
                            transition: *transition,
                            start: now,
                            from_image,
                            from_x,
                            leaving: None,
                        },
                    );
                    if !entering && !self.characters[character].changes() {
                        self.characters.remove(character);
                    }
                }
                None => {
                    self.characters.remove(character);
                }
            },
            Event::Hide {
                character,
                transition,
            } => match (transition, self.shown.get(character)) {
                (Some(transition), Some(placed)) => {
                    let leaving = placed.clone();
                    self.leave(character, leaving, *transition, now);
                }
                _ => {
                    self.characters.remove(character);
                }
            },
            Event::Clear { transition } => match transition {
                Some(transition) => {
                    let shown = std::mem::take(&mut self.shown);
                    for (id, placed) in shown {
                        self.leave(&id, placed, *transition, now);
                    }
                }
                None => self.characters.clear(),
            },
            Event::Background { transition, .. } => {
                self.backdrop = transition.map(|transition| BackgroundAnim {
                    transition,
                    start: now,
                    from: self.background.clone(),
                });
            }
            _ => return,
        }
        self.sync(story, config);
    }

    fn leave(&mut self, id: &str, placed: Placed, transition: Transition, now: f64) {
        self.characters.insert(
            id.to_string(),
            CharacterAnim {
                transition,
                start: now,
                from_image: None,
                from_x: None,
                leaving: Some(placed),
            },
        );
    }

    pub fn texture_paths(&self) -> Vec<String> {
        let mut paths = Vec::new();
        for (id, anim) in &self.characters {
            if let Some(image) = &anim.from_image {
                paths.push(character_path(id, image));
            }
            if let Some(placed) = &anim.leaving {
                paths.push(character_path(id, &placed.image));
            }
        }
        if let Some(from) = self.backdrop.as_ref().and_then(|anim| anim.from.as_deref()) {
            paths.push(background_path(from));
        }
        paths
    }

    pub fn expire(&mut self, now: f64) {
        self.characters
            .retain(|_, anim| progress(anim.start, &anim.transition, now) < 1.0);
        if self
            .backdrop
            .as_ref()
            .is_some_and(|anim| progress(anim.start, &anim.transition, now) >= 1.0)
        {
            self.backdrop = None;
        }
    }

    pub fn finish(&mut self) {
        self.characters.clear();
        self.backdrop = None;
    }

    pub fn draw(
        &self,
        d: &mut RaylibDrawHandle,
        resources: &ResourceManager,
        story: &StoryVm,
        config: &PlayingConfig,
        now: f64,
    ) {
        let screen = ui::screen_size(d);
        let fade = self.draw_background(d, resources, story, config.background.as_ref(), now);
        crate::dev::picker::record_slots(Position::ALL.map(|position| config.position_x(position)));

        let current = stage_layout(story, config);
        let mut ids: Vec<&String> = current.keys().collect();
        for (id, anim) in &self.characters {
            if anim.leaving.is_some() && !current.contains_key(id) {
                ids.push(id);
            }
        }
        ids.sort();
        ids.dedup();

        for id in ids {
            match (current.get(id), self.characters.get(id)) {
                (Some(placed), Some(anim)) if anim.leaving.is_none() => {
                    let t = progress(anim.start, &anim.transition, now);
                    let x = anim.from_x.map_or(placed.x, |from| {
                        lerp(from, placed.x, Easing::Smooth.apply(t))
                    });
                    match &anim.from_image {
                        Some(old) => {
                            sprite(d, resources, config, screen, id, old, x, 1.0 - t.powi(3));
                            sprite(d, resources, config, screen, id, &placed.image, x, t);
                        }
                        None if anim.from_x.is_some() => {
                            sprite(d, resources, config, screen, id, &placed.image, x, 1.0);
                        }
                        None => sprite(d, resources, config, screen, id, &placed.image, x, t),
                    }
                }
                (Some(placed), _) => {
                    sprite(
                        d,
                        resources,
                        config,
                        screen,
                        id,
                        &placed.image,
                        placed.x,
                        1.0,
                    );
                }
                (None, Some(anim)) => {
                    let Some(placed) = &anim.leaving else {
                        continue;
                    };
                    let t = progress(anim.start, &anim.transition, now);
                    let (x, alpha) = match anim.transition.kind {
                        TransitionKind::SlideLeft => {
                            (lerp(placed.x, OFFSCREEN_LEFT, Easing::Smooth.apply(t)), 1.0)
                        }
                        TransitionKind::SlideRight => (
                            lerp(placed.x, OFFSCREEN_RIGHT, Easing::Smooth.apply(t)),
                            1.0,
                        ),
                        TransitionKind::Dissolve | TransitionKind::Fade => (placed.x, 1.0 - t),
                        TransitionKind::Shake | TransitionKind::Flash => (placed.x, 0.0),
                    };
                    sprite(d, resources, config, screen, id, &placed.image, x, alpha);
                }
                (None, None) => {}
            }
        }

        if fade > 0.0 {
            let alpha = (fade * 255.0).round() as u8;
            d.draw_rectangle(
                0,
                0,
                screen.x as i32,
                screen.y as i32,
                Color::new(0, 0, 0, alpha),
            );
        }
    }

    fn draw_background(
        &self,
        d: &mut RaylibDrawHandle,
        resources: &ResourceManager,
        story: &StoryVm,
        fallback: Option<&Background>,
        now: f64,
    ) -> f32 {
        let screen = ui::screen_size(d);
        let current = story.background();
        ui::draw_background(d, resources, fallback);

        let Some(anim) = &self.backdrop else {
            backdrop(d, resources, current, 1.0, 0.0, screen);
            return 0.0;
        };

        let t = progress(anim.start, &anim.transition, now);
        let from = anim.from.as_deref();
        match anim.transition.kind {
            TransitionKind::Dissolve => {
                if current.is_some() {
                    backdrop(d, resources, from, 1.0, 0.0, screen);
                    backdrop(d, resources, current, t, 0.0, screen);
                } else {
                    backdrop(d, resources, from, 1.0 - t, 0.0, screen);
                }
                0.0
            }
            TransitionKind::Fade => {
                if t < 0.5 {
                    backdrop(d, resources, from, 1.0, 0.0, screen);
                    t * 2.0
                } else {
                    backdrop(d, resources, current, 1.0, 0.0, screen);
                    (1.0 - t) * 2.0
                }
            }
            TransitionKind::SlideLeft | TransitionKind::SlideRight => {
                let sign = if anim.transition.kind == TransitionKind::SlideLeft {
                    -1.0
                } else {
                    1.0
                };
                let shift = Easing::Smooth.apply(t) * screen.x * sign;
                backdrop(d, resources, from, 1.0, shift, screen);
                backdrop(d, resources, current, 1.0, shift - screen.x * sign, screen);
                0.0
            }
            TransitionKind::Shake | TransitionKind::Flash => {
                backdrop(d, resources, current, 1.0, 0.0, screen);
                0.0
            }
        }
    }
}

impl CharacterAnim {
    fn changes(&self) -> bool {
        self.from_image.is_some() || self.from_x.is_some()
    }
}

fn backdrop(
    d: &mut RaylibDrawHandle,
    resources: &ResourceManager,
    image: Option<&str>,
    alpha: f32,
    x_offset: f32,
    screen: Vector2,
) {
    let Some(texture) = image.and_then(|image| resources.textures.get(&background_path(image)))
    else {
        return;
    };
    let area = Rectangle::new(x_offset, 0.0, screen.x, screen.y);
    let (w, h) = (texture.width as f32, texture.height as f32);
    let scale = (area.width / w).max(area.height / h);
    let (source_w, source_h) = (area.width / scale, area.height / scale);
    let source = Rectangle::new(
        (w - source_w) / 2.0,
        (h - source_h) / 2.0,
        source_w,
        source_h,
    );
    d.draw_texture_pro(texture, source, area, Vector2::zero(), 0.0, tint(alpha));
}

fn tint(alpha: f32) -> Color {
    Color::new(255, 255, 255, (alpha.clamp(0.0, 1.0) * 255.0).round() as u8)
}

#[allow(clippy::too_many_arguments)]
fn sprite(
    d: &mut RaylibDrawHandle,
    resources: &ResourceManager,
    config: &PlayingConfig,
    screen: Vector2,
    character: &str,
    image: &str,
    x: f32,
    alpha: f32,
) {
    if alpha <= 0.0 {
        return;
    }
    let x = crate::dev::picker::dragged_x(character).unwrap_or(x);
    #[cfg(feature = "character-visuals")]
    if resources.visuals.draw(
        character,
        image,
        d,
        screen,
        x,
        config.character_height,
        alpha,
    ) {
        return;
    }
    let Some(texture) = resources.textures.get(&character_path(character, image)) else {
        return;
    };

    let (w, h) = (texture.width as f32, texture.height as f32);
    let scale = config
        .character_height
        .map_or(1.0, |fraction| screen.y * fraction / h);
    let center_x = screen.x * x;
    let dest = Rectangle::new(
        center_x - w * scale / 2.0,
        screen.y - h * scale,
        w * scale,
        h * scale,
    );
    d.draw_texture_pro(
        texture,
        Rectangle::new(0.0, 0.0, w, h),
        dest,
        Vector2::zero(),
        0.0,
        tint(alpha),
    );
    crate::dev::picker::record(character, image, dest);
    if crate::dev::inspector::active() {
        crate::dev::inspector::widget(crate::dev::inspector::Widget {
            kind: "character",
            rect: dest,
            label: format!("{character} {image}"),
            details: vec![
                ("x", format!("{x:.3} of the width")),
                ("texture", format!("{w:.0} x {h:.0}, drawn at {scale:.2}x")),
            ],
            ..Default::default()
        });
    }
}

pub fn effect_of(event: &Event) -> Option<(TransitionKind, f32)> {
    let transition = match event {
        Event::Show { transition, .. }
        | Event::Background { transition, .. }
        | Event::Hide { transition, .. }
        | Event::Clear { transition } => transition.as_ref()?,
        _ => return None,
    };
    transition
        .kind
        .is_effect()
        .then(|| (transition.kind, transition.seconds()))
}
