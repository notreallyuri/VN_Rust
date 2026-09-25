use std::collections::{BTreeMap, BTreeSet};

use raylib::prelude::*;

use crate::data::assets::{Assets, extension_of};
use crate::game::visuals::{CharacterVisual, CharacterVisualFactory, VisualFrame};

pub const BREATH: &str = "breath";
pub const SWAY: &str = "sway";
pub const BLINK: &str = "blink";

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Motion {
    Rotate(f32),
    Offset(Vector2),
    Scale(Vector2),
    Opacity(f32),
}

impl Motion {
    pub fn rotate(degrees: f32) -> Self {
        Motion::Rotate(degrees)
    }

    pub fn offset(x: f32, y: f32) -> Self {
        Motion::Offset(Vector2::new(x, y))
    }

    pub fn scale(x: f32, y: f32) -> Self {
        Motion::Scale(Vector2::new(x, y))
    }

    pub fn opacity(opacity: f32) -> Self {
        Motion::Opacity(opacity)
    }

    fn finite(&self) -> bool {
        match self {
            Motion::Rotate(degrees) => degrees.is_finite(),
            Motion::Offset(by) | Motion::Scale(by) => by.x.is_finite() && by.y.is_finite(),
            Motion::Opacity(opacity) => opacity.is_finite(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Part {
    name: String,
    image: String,
    at: Vector2,
    pivot: Vector2,
    binds: Vec<(String, Motion)>,
}

impl Part {
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            image: name.clone(),
            name,
            at: Vector2::zero(),
            pivot: Vector2::new(0.5, 0.5),
            binds: Vec::new(),
        }
    }

    pub fn image(mut self, image: impl Into<String>) -> Self {
        self.image = image.into();
        self
    }

    pub fn at(mut self, x: f32, y: f32) -> Self {
        self.at = Vector2::new(x, y);
        self
    }

    pub fn pivot(mut self, x: f32, y: f32) -> Self {
        self.pivot = Vector2::new(x, y);
        self
    }

    pub fn bind(mut self, parameter: impl Into<String>, motion: Motion) -> Self {
        self.binds.push((parameter.into(), motion));
        self
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Debug, Default)]
pub struct Pose {
    swaps: BTreeMap<String, String>,
    hidden: BTreeSet<String>,
    parameters: BTreeMap<String, f32>,
}

impl Pose {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn swap(mut self, part: impl Into<String>, image: impl Into<String>) -> Self {
        self.swaps.insert(part.into(), image.into());
        self
    }

    pub fn hide(mut self, part: impl Into<String>) -> Self {
        self.hidden.insert(part.into());
        self
    }

    pub fn parameter(mut self, id: impl Into<String>, value: f32) -> Self {
        self.parameters.insert(id.into(), value);
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Idle {
    pub breath: f32,
    pub sway: f32,
    pub blink_every: f32,
    pub blink_for: f32,
}

impl Default for Idle {
    fn default() -> Self {
        Self {
            breath: 4.0,
            sway: 9.0,
            blink_every: 5.0,
            blink_for: 0.12,
        }
    }
}

impl Idle {
    pub fn at(&self, clock: f32) -> [(&'static str, f32); 3] {
        let wave = |period: f32| match period > 0.0 {
            true => (clock / period * std::f32::consts::TAU).sin(),
            false => 0.0,
        };
        let blink = match self.blink_every > 0.0 && self.blink_for > 0.0 {
            true => {
                let into = clock.rem_euclid(self.blink_every);
                match into < self.blink_for {
                    true => 1.0 - (into / self.blink_for * 2.0 - 1.0).abs(),
                    false => 0.0,
                }
            }
            false => 0.0,
        };
        [
            (BREATH, (wave(self.breath) + 1.0) / 2.0),
            (SWAY, wave(self.sway)),
            (BLINK, blink),
        ]
    }
}

#[derive(Clone, Debug)]
pub struct Puppet {
    size: Vector2,
    folder: Option<String>,
    parts: Vec<Part>,
    appearances: BTreeMap<String, Pose>,
    idle: Idle,
}

impl Puppet {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            size: Vector2::new(width, height),
            folder: None,
            parts: Vec::new(),
            appearances: BTreeMap::new(),
            idle: Idle::default(),
        }
    }

    pub fn folder(mut self, folder: impl Into<String>) -> Self {
        self.folder = Some(folder.into());
        self
    }

    pub fn part(mut self, part: Part) -> Self {
        self.parts.push(part);
        self
    }

    pub fn appearance(mut self, name: impl Into<String>, pose: Pose) -> Self {
        self.appearances.insert(name.into(), pose);
        self
    }

    pub fn idle(mut self, idle: impl FnOnce(Idle) -> Idle) -> Self {
        self.idle = idle(self.idle);
        self
    }

    pub fn path(&self, image: &str) -> String {
        match &self.folder {
            Some(folder) => format!("characters/{}/{}.png", folder, image).to_lowercase(),
            None => format!("{}.png", image).to_lowercase(),
        }
    }

    fn images(&self, appearance: &str) -> Vec<(usize, String)> {
        let pose = self.appearances.get(appearance);
        self.parts
            .iter()
            .enumerate()
            .filter(|(_, part)| pose.is_none_or(|pose| !pose.hidden.contains(&part.name)))
            .map(|(index, part)| {
                let image = pose
                    .and_then(|pose| pose.swaps.get(&part.name))
                    .unwrap_or(&part.image);
                (index, self.path(image))
            })
            .collect()
    }

    pub fn check(&self, assets: &Assets) -> Result<(), String> {
        if !self.size.x.is_finite()
            || !self.size.y.is_finite()
            || self.size.x <= 0.0
            || self.size.y <= 0.0
        {
            return Err("puppet size must be finite and positive".into());
        }
        if self.parts.is_empty() {
            return Err("puppet needs at least one part".into());
        }
        if self.appearances.is_empty() {
            return Err("puppet needs at least one named appearance".into());
        }

        let mut names = BTreeSet::new();
        for part in &self.parts {
            if !names.insert(part.name.as_str()) {
                return Err(format!("two parts are called '{}'", part.name));
            }
            if !part.at.x.is_finite() || !part.at.y.is_finite() {
                return Err(format!("part '{}': position must be finite", part.name));
            }
            if !part.pivot.x.is_finite() || !part.pivot.y.is_finite() {
                return Err(format!("part '{}': pivot must be finite", part.name));
            }
            for (parameter, motion) in &part.binds {
                if parameter.is_empty() {
                    return Err(format!(
                        "part '{}': a bound parameter has no name",
                        part.name
                    ));
                }
                if !motion.finite() {
                    return Err(format!(
                        "part '{}': parameter '{}' moves it by a value that is not finite",
                        part.name, parameter
                    ));
                }
            }
        }

        for (name, pose) in &self.appearances {
            for part in pose.swaps.keys().chain(&pose.hidden) {
                if !names.contains(part.as_str()) {
                    return Err(format!("appearance '{name}': there is no part '{part}'"));
                }
            }
            for (id, value) in &pose.parameters {
                if id.is_empty() || !value.is_finite() {
                    return Err(format!("appearance '{name}': invalid parameter '{id}'"));
                }
            }
            for (_, path) in self.images(name) {
                if !assets.exists(&path) {
                    return Err(format!(
                        "appearance '{name}': {} is missing",
                        assets.describe(&path)
                    ));
                }
            }
        }
        Ok(())
    }
}

impl CharacterVisualFactory for Puppet {
    fn appearances(&self) -> Vec<String> {
        self.appearances.keys().cloned().collect()
    }

    fn validate(&self, assets: &Assets) -> Result<(), String> {
        self.check(assets)
    }

    fn load(
        &self,
        appearance: &str,
        assets: &Assets,
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
    ) -> Result<Box<dyn CharacterVisual>, String> {
        let pose = self
            .appearances
            .get(appearance)
            .ok_or_else(|| format!("no appearance called '{appearance}'"))?;

        let mut parts = Vec::new();
        for (index, path) in self.images(appearance) {
            let bytes = assets
                .read(&path)
                .map_err(|e| format!("{}: {e}", assets.describe(&path)))?;
            let image = Image::load_image_from_mem(&extension_of(&path), &bytes)
                .map_err(|e| format!("{}: {e}", assets.describe(&path)))?;
            let texture = rl
                .load_texture_from_image(thread, &image)
                .map_err(|e| format!("{}: {e}", assets.describe(&path)))?;
            parts.push(LoadedPart {
                part: self.parts[index].clone(),
                natural: Vector2::new(texture.width as f32, texture.height as f32),
                texture,
            });
        }

        Ok(Box::new(PuppetVisual {
            size: self.size,
            parts,
            pose: pose.parameters.clone(),
            overrides: BTreeMap::new(),
            idle: self.idle,
            clock: 0.0,
        }))
    }
}

struct LoadedPart {
    part: Part,
    texture: Texture2D,
    natural: Vector2,
}

pub(crate) struct PuppetVisual {
    size: Vector2,
    parts: Vec<LoadedPart>,
    pose: BTreeMap<String, f32>,
    overrides: BTreeMap<String, f32>,
    idle: Idle,
    clock: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Placement {
    pub dest: Rectangle,
    pub origin: Vector2,
    pub rotation: f32,
    pub opacity: f32,
}

impl PuppetVisual {
    pub(crate) fn value(&self, parameter: &str) -> f32 {
        if let Some(value) = self.overrides.get(parameter) {
            return *value;
        }
        if let Some(value) = self.pose.get(parameter) {
            return *value;
        }
        self.idle
            .at(self.clock)
            .into_iter()
            .find(|(id, _)| *id == parameter)
            .map_or(0.0, |(_, value)| value)
    }

    pub(crate) fn placement(&self, index: usize, frame: VisualFrame) -> Placement {
        let loaded = &self.parts[index];
        let part = &loaded.part;
        let scale = frame.rect.width / self.size.x;

        let mut offset = Vector2::zero();
        let mut stretch = Vector2::new(1.0, 1.0);
        let mut rotation = 0.0;
        let mut opacity = frame.opacity;

        for (parameter, motion) in &part.binds {
            let value = self.value(parameter);
            match motion {
                Motion::Rotate(degrees) => rotation += degrees * value,
                Motion::Offset(by) => offset += *by * value,
                Motion::Scale(by) => {
                    stretch.x *= 1.0 + (by.x - 1.0) * value;
                    stretch.y *= 1.0 + (by.y - 1.0) * value;
                }
                Motion::Opacity(by) => opacity *= 1.0 + (by - 1.0) * value,
            }
        }

        let size = Vector2::new(
            loaded.natural.x * stretch.x * scale,
            loaded.natural.y * stretch.y * scale,
        );
        let origin = Vector2::new(size.x * part.pivot.x, size.y * part.pivot.y);
        let at = (part.at + offset) * scale;

        Placement {
            dest: Rectangle::new(frame.rect.x + at.x, frame.rect.y + at.y, size.x, size.y),
            origin,
            rotation,
            opacity: opacity.clamp(0.0, 1.0),
        }
    }
}

impl CharacterVisual for PuppetVisual {
    fn size(&self) -> Vector2 {
        self.size
    }

    fn update(&mut self, seconds: f32) -> Result<(), String> {
        if seconds.is_finite() {
            self.clock += seconds;
        }
        Ok(())
    }

    fn restart(&mut self) -> Result<(), String> {
        self.clock = 0.0;
        self.overrides.clear();
        Ok(())
    }

    fn set_parameter(&mut self, id: &str, value: Option<f32>) -> Result<(), String> {
        match value {
            Some(value) if !value.is_finite() => {
                return Err(format!("parameter '{id}' must be finite"));
            }
            Some(value) => self.overrides.insert(id.to_string(), value),
            None => self.overrides.remove(id),
        };
        Ok(())
    }

    fn draw(&mut self, draw: &mut RaylibDrawHandle, frame: VisualFrame) -> Result<(), String> {
        for index in 0..self.parts.len() {
            let placed = self.placement(index, frame);
            if placed.opacity <= 0.0 {
                continue;
            }
            let loaded = &self.parts[index];
            draw.draw_texture_pro(
                &loaded.texture,
                Rectangle::new(0.0, 0.0, loaded.natural.x, loaded.natural.y),
                placed.dest,
                placed.origin,
                placed.rotation,
                Color::WHITE.alpha(placed.opacity),
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rig() -> Puppet {
        Puppet::new(600.0, 900.0)
            .folder("mary")
            .part(Part::new("body").at(300.0, 900.0).pivot(0.5, 1.0))
            .part(
                Part::new("head")
                    .at(300.0, 380.0)
                    .pivot(0.5, 1.0)
                    .bind(SWAY, Motion::rotate(6.0))
                    .bind(BREATH, Motion::offset(0.0, -4.0)),
            )
            .part(
                Part::new("eyes")
                    .at(300.0, 300.0)
                    .bind(BLINK, Motion::scale(1.0, 0.1)),
            )
            .part(
                Part::new("mouth")
                    .at(300.0, 340.0)
                    .bind("mouth", Motion::scale(1.0, 2.0)),
            )
            .appearance("neutral", Pose::new())
            .appearance(
                "happy",
                Pose::new()
                    .swap("mouth", "mouth_smile")
                    .parameter(SWAY, 0.5),
            )
    }

    fn visual(appearance: &str) -> PuppetVisual {
        visual_of(&rig(), appearance)
    }

    fn visual_of(puppet: &Puppet, appearance: &str) -> PuppetVisual {
        let pose = puppet.appearances[appearance].clone();
        let parts = puppet
            .images(appearance)
            .into_iter()
            .map(|(index, _)| LoadedPart {
                part: puppet.parts[index].clone(),
                texture: unsafe { std::mem::zeroed() },
                natural: Vector2::new(300.0, 300.0),
            })
            .collect();
        PuppetVisual {
            size: puppet.size,
            parts,
            pose: pose.parameters,
            overrides: BTreeMap::new(),
            idle: puppet.idle,
            clock: 0.0,
        }
    }

    fn forget(visual: PuppetVisual) {
        for part in visual.parts {
            std::mem::forget(part.texture);
        }
    }

    #[test]
    fn a_rig_names_the_pictures_each_appearance_needs() {
        let puppet = rig();

        let neutral: Vec<String> = puppet
            .images("neutral")
            .into_iter()
            .map(|(_, p)| p)
            .collect();
        assert_eq!(
            neutral,
            [
                "characters/mary/body.png",
                "characters/mary/head.png",
                "characters/mary/eyes.png",
                "characters/mary/mouth.png",
            ]
        );

        let happy: Vec<String> = puppet.images("happy").into_iter().map(|(_, p)| p).collect();
        assert_eq!(
            happy[3], "characters/mary/mouth_smile.png",
            "a pose swaps one part's picture and leaves the rest"
        );

        let hidden = rig().appearance("shy", Pose::new().hide("eyes"));
        assert_eq!(hidden.images("shy").len(), 3, "a hidden part is not loaded");
    }

    #[test]
    fn idle_movement_is_a_function_of_the_clock() {
        let idle = Idle::default();
        assert_eq!(idle.at(0.0), idle.at(0.0), "the same moment, the same pose");
        assert!(
            (idle.at(4.0)[0].1 - idle.at(8.0)[0].1).abs() < 1e-5,
            "breathing repeats with its period"
        );

        let blink = |clock: f32| idle.at(clock)[2].1;
        assert_eq!(blink(0.0), 0.0);
        assert!(
            blink(idle.blink_for / 2.0) > 0.9,
            "the eye closes and opens"
        );
        assert_eq!(blink(1.0), 0.0, "and stays open in between");
        assert_eq!(blink(5.0), blink(10.0), "every blink_every seconds");
    }

    #[test]
    fn a_pose_holds_a_parameter_and_the_game_can_still_speak_over_it() {
        let mut shown = visual("happy");
        assert_eq!(shown.value(SWAY), 0.5, "the appearance holds it still");

        shown.set_parameter(SWAY, Some(-1.0)).unwrap();
        assert_eq!(shown.value(SWAY), -1.0, "what the game pushes wins");

        shown.set_parameter(SWAY, None).unwrap();
        assert_eq!(shown.value(SWAY), 0.5, "releasing it returns to the pose");

        assert!(shown.set_parameter(SWAY, Some(f32::NAN)).is_err());

        let mut idle = visual("neutral");
        idle.update(1.0).unwrap();
        assert_eq!(
            idle.value(SWAY),
            idle.idle.at(1.0)[1].1,
            "without a pose the clock drives it"
        );
        forget(shown);
        forget(idle);
    }

    #[test]
    fn a_part_is_placed_from_its_pivot_and_scales_with_the_frame() {
        let mut shown = visual("neutral");
        let frame = VisualFrame {
            rect: Rectangle::new(100.0, 50.0, 300.0, 450.0),
            layout_size: Vector2::new(1280.0, 720.0),
            target_size: (1280, 720),
            opacity: 1.0,
        };

        let body = shown.placement(0, frame);
        assert_eq!(
            (body.dest.x, body.dest.y),
            (100.0 + 150.0, 50.0 + 450.0),
            "the rig's coordinates are halved with the frame"
        );
        assert_eq!((body.dest.width, body.dest.height), (150.0, 150.0));
        assert_eq!(body.origin, Vector2::new(75.0, 150.0), "pivot at the feet");
        assert_eq!(body.rotation, 0.0);

        shown.set_parameter(SWAY, Some(1.0)).unwrap();
        shown.set_parameter(BREATH, Some(1.0)).unwrap();
        let head = shown.placement(1, frame);
        assert_eq!(
            head.rotation, 6.0,
            "a bound parameter at 1 gives its amount"
        );
        assert_eq!(
            head.dest.y,
            50.0 + (380.0 - 4.0) / 2.0,
            "and its offset, in frame pixels"
        );

        shown.set_parameter(BLINK, Some(1.0)).unwrap();
        let eyes = shown.placement(2, frame);
        assert!(
            (eyes.dest.height - 15.0).abs() < 0.01,
            "a closed eye is thin"
        );
        assert_eq!(eyes.dest.width, 150.0, "and no narrower");
        forget(shown);
    }

    #[test]
    fn restarting_forgets_the_clock_and_what_the_game_pushed() {
        let mut shown = visual("happy");
        shown.update(2.0).unwrap();
        shown.set_parameter("mouth", Some(1.0)).unwrap();

        shown.restart().unwrap();
        assert_eq!(shown.clock, 0.0);
        assert_eq!(shown.value("mouth"), 0.0);
        assert_eq!(shown.value(SWAY), 0.5, "the appearance's own pose stays");
        forget(shown);
    }

    #[test]
    fn a_rig_is_checked_before_it_is_ever_drawn() {
        let assets = Assets::from(std::path::PathBuf::from("/nowhere"));

        let broken = [
            (
                Puppet::new(0.0, 900.0)
                    .part(Part::new("body"))
                    .appearance("a", Pose::new()),
                "size must be finite and positive",
            ),
            (
                Puppet::new(600.0, 900.0).appearance("a", Pose::new()),
                "at least one part",
            ),
            (
                Puppet::new(600.0, 900.0).part(Part::new("body")),
                "at least one named appearance",
            ),
            (
                Puppet::new(600.0, 900.0)
                    .part(Part::new("body"))
                    .part(Part::new("body"))
                    .appearance("a", Pose::new()),
                "two parts are called 'body'",
            ),
            (
                Puppet::new(600.0, 900.0)
                    .part(Part::new("body"))
                    .appearance("a", Pose::new().swap("hed", "x")),
                "there is no part 'hed'",
            ),
            (
                Puppet::new(600.0, 900.0)
                    .part(Part::new("body").bind("x", Motion::rotate(f32::INFINITY)))
                    .appearance("a", Pose::new()),
                "not finite",
            ),
        ];
        for (puppet, expected) in broken {
            let error = puppet.check(&assets).unwrap_err();
            assert!(error.contains(expected), "{error}\nexpected: {expected}");
        }

        let missing = rig().check(&assets).unwrap_err();
        assert!(missing.contains("is missing"), "{missing}");
    }

    #[test]
    fn hiding_a_part_keeps_every_picture_on_the_part_that_asked_for_it() {
        let puppet = rig().appearance("shy", Pose::new().hide("head"));
        let picked: Vec<(&str, String)> = puppet
            .images("shy")
            .into_iter()
            .map(|(index, path)| (puppet.parts[index].name(), path))
            .collect();

        assert_eq!(
            picked,
            [
                ("body", "characters/mary/body.png".to_string()),
                ("eyes", "characters/mary/eyes.png".to_string()),
                ("mouth", "characters/mary/mouth.png".to_string()),
            ],
            "the index a picture comes back with is the part it belongs to"
        );

        let loose = Puppet::new(600.0, 900.0)
            .part(Part::new("Body").image("Body_Front"))
            .appearance("a", Pose::new());
        assert_eq!(
            loose.images("a")[0].1,
            "body_front.png",
            "no folder, lowercased"
        );
    }

    #[test]
    fn an_idle_of_zeroes_holds_the_character_still() {
        let idle = Idle {
            breath: 0.0,
            sway: 0.0,
            blink_every: 0.0,
            blink_for: 0.0,
        };
        assert_eq!(
            idle.at(3.7),
            [(BREATH, 0.5), (SWAY, 0.0), (BLINK, 0.0)],
            "a period of zero rests the parameter in its middle"
        );
        assert_eq!(idle.at(3.7), idle.at(120.0));
    }

    #[test]
    fn a_part_composes_every_motion_bound_to_it() {
        let puppet = Puppet::new(100.0, 100.0)
            .part(
                Part::new("aura")
                    .at(50.0, 50.0)
                    .bind("turn", Motion::rotate(30.0))
                    .bind("turn", Motion::rotate(-10.0))
                    .bind("grow", Motion::scale(2.0, 2.0))
                    .bind("fade", Motion::opacity(0.2)),
            )
            .appearance("a", Pose::new());
        let mut shown = visual_of(&puppet, "a");
        let frame = VisualFrame {
            rect: Rectangle::new(0.0, 0.0, 100.0, 100.0),
            layout_size: Vector2::new(1280.0, 720.0),
            target_size: (1280, 720),
            opacity: 0.5,
        };

        shown.set_parameter("turn", Some(1.0)).unwrap();
        shown.set_parameter("grow", Some(0.5)).unwrap();
        shown.set_parameter("fade", Some(0.5)).unwrap();
        let placed = shown.placement(0, frame);

        assert_eq!(placed.rotation, 20.0, "two turns on one part add up");
        assert_eq!(placed.dest.width, 450.0, "halfway to twice the size");
        assert_eq!(
            placed.origin,
            Vector2::new(225.0, 225.0),
            "the pivot follows"
        );
        assert!(
            (placed.opacity - 0.3).abs() < 1e-6,
            "the fade multiplies the transition's own opacity, not replaces it"
        );

        shown.set_parameter("fade", Some(4.0)).unwrap();
        assert_eq!(shown.placement(0, frame).opacity, 0.0, "and stays in range");
        shown.set_parameter("fade", Some(-4.0)).unwrap();
        assert_eq!(shown.placement(0, frame).opacity, 1.0, "at both ends");
        forget(shown);
    }

    #[test]
    fn a_stalled_frame_does_not_move_the_clock() {
        let mut shown = visual("neutral");
        shown.update(0.5).unwrap();
        shown.update(f32::NAN).unwrap();
        shown.update(f32::INFINITY).unwrap();
        assert_eq!(shown.clock, 0.5);
        forget(shown);
    }
}
