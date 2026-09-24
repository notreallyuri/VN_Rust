use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

use raylib::prelude::*;
use vn_script::Schema;

use crate::data::assets::Assets;
use crate::data::rollback::VisualParameters;

pub trait CharacterVisual {
    fn size(&self) -> Vector2;
    fn update(&mut self, seconds: f32) -> Result<(), String>;
    fn restart(&mut self) -> Result<(), String> {
        Ok(())
    }
    fn set_parameter(&mut self, _id: &str, _value: Option<f32>) -> Result<(), String> {
        Ok(())
    }
    fn draw(&mut self, draw: &mut RaylibDrawHandle, frame: VisualFrame) -> Result<(), String>;
}

pub trait CharacterVisualFactory {
    fn appearances(&self) -> Vec<String>;
    fn validate(&self, assets: &Assets) -> Result<(), String>;
    fn load(
        &self,
        appearance: &str,
        assets: &Assets,
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
    ) -> Result<Box<dyn CharacterVisual>, String>;
}

#[derive(Clone, Copy, Debug)]
pub struct VisualFrame {
    pub rect: Rectangle,
    pub layout_size: Vector2,
    pub target_size: (u32, u32),
    pub opacity: f32,
}

impl VisualFrame {
    pub fn placed(
        natural: Vector2,
        layout: Vector2,
        x: f32,
        height: Option<f32>,
        opacity: f32,
        render_scale: f32,
    ) -> Self {
        let scale = height.map_or(1.0, |fraction| layout.y * fraction / natural.y);
        let size = natural * scale;
        Self {
            rect: Rectangle::new(
                layout.x * x - size.x / 2.0,
                layout.y - size.y,
                size.x,
                size.y,
            ),
            layout_size: layout,
            target_size: (
                (layout.x * render_scale).round().max(1.0) as u32,
                (layout.y * render_scale).round().max(1.0) as u32,
            ),
            opacity,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct VisualKey {
    pub character: String,
    pub appearance: String,
}

impl VisualKey {
    pub fn new(character: impl Into<String>, appearance: impl Into<String>) -> Self {
        Self {
            character: character.into(),
            appearance: appearance.into(),
        }
    }
}

#[derive(Clone, Default)]
pub struct VisualRegistry {
    factories: BTreeMap<String, Rc<dyn CharacterVisualFactory>>,
}

impl std::fmt::Debug for VisualRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.factories.keys()).finish()
    }
}

impl VisualRegistry {
    pub fn insert(&mut self, character: String, factory: impl CharacterVisualFactory + 'static) {
        self.factories.insert(character, Rc::new(factory));
    }

    pub fn handles(&self, character: &str, appearance: &str) -> bool {
        self.factories
            .get(character)
            .is_some_and(|factory| factory.appearances().iter().any(|name| name == appearance))
    }

    pub fn extend_schema(&self, schema: &mut Schema) {
        for (id, factory) in &self.factories {
            if let Some(character) = schema.characters.get_mut(id) {
                character.images.extend(factory.appearances());
                character.images.sort();
                character.images.dedup();
            }
        }
    }

    pub fn validate(&self, schema: &Schema, assets: &Assets) -> Result<(), String> {
        for (id, factory) in &self.factories {
            if !schema.characters.contains_key(id) {
                return Err(format!(
                    "visual backend refers to unregistered character '{id}'"
                ));
            }
            let names = factory.appearances();
            let unique: BTreeSet<_> = names.iter().collect();
            if names.is_empty()
                || unique.len() != names.len()
                || names.iter().any(|name| !vn_script::is_identifier(name))
            {
                return Err(format!(
                    "character '{id}': appearance names must be unique story identifiers"
                ));
            }
            factory
                .validate(assets)
                .map_err(|e| format!("character '{id}': {e}"))?;
        }
        Ok(())
    }
}

enum Instance {
    Ready(Box<dyn CharacterVisual>),
    Failed,
}

#[derive(Default)]
pub struct CharacterVisuals {
    pub registry: VisualRegistry,
    instances: RefCell<BTreeMap<VisualKey, Instance>>,
    pushed: VisualParameters,
}

impl CharacterVisuals {
    pub fn reset(&mut self) {
        self.instances.get_mut().clear();
        self.pushed.clear();
    }

    pub fn restart(&mut self) {
        self.pushed.clear();
        for (key, instance) in self.instances.get_mut() {
            if let Instance::Ready(visual) = instance
                && let Err(error) = visual.restart()
            {
                report(key, &error);
                *instance = Instance::Failed;
            }
        }
    }

    pub fn pushed(&self) -> &VisualParameters {
        &self.pushed
    }

    pub fn restore(&mut self, parameters: VisualParameters) {
        let touched: BTreeSet<(String, String)> = self
            .pushed
            .iter()
            .chain(parameters.iter())
            .flat_map(|(character, values)| {
                values.keys().map(move |id| (character.clone(), id.clone()))
            })
            .collect();
        self.pushed = parameters;
        for (character, id) in touched {
            let value = self
                .pushed
                .get(&character)
                .and_then(|values| values.get(&id))
                .copied();
            self.apply(&character, &id, value);
        }
    }

    pub fn set_parameter(&mut self, character: &str, id: &str, value: Option<f32>) {
        match value {
            Some(value) => {
                self.pushed
                    .entry(character.to_string())
                    .or_default()
                    .insert(id.to_string(), value);
            }
            None => {
                if let Some(parameters) = self.pushed.get_mut(character) {
                    parameters.remove(id);
                    if parameters.is_empty() {
                        self.pushed.remove(character);
                    }
                }
            }
        }
        self.apply(character, id, value);
    }

    fn apply(&mut self, character: &str, id: &str, value: Option<f32>) {
        for (key, instance) in self.instances.get_mut() {
            if key.character != character {
                continue;
            }
            if let Instance::Ready(visual) = instance
                && let Err(error) = visual.set_parameter(id, value)
            {
                report(key, &error);
                *instance = Instance::Failed;
            }
        }
    }

    pub fn prepare(
        &mut self,
        keys: impl IntoIterator<Item = VisualKey>,
        assets: &Assets,
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        seconds: f32,
    ) -> Vec<VisualKey> {
        let registry = self.registry.clone();
        self.prepare_with(keys, seconds, |key| {
            if !registry.handles(&key.character, &key.appearance) {
                return None;
            }
            Some(registry.factories[&key.character].load(&key.appearance, assets, rl, thread))
        })
    }

    fn prepare_with(
        &mut self,
        keys: impl IntoIterator<Item = VisualKey>,
        seconds: f32,
        mut load: impl FnMut(&VisualKey) -> Option<Result<Box<dyn CharacterVisual>, String>>,
    ) -> Vec<VisualKey> {
        let wanted: BTreeSet<_> = keys.into_iter().collect();
        let pushed = &self.pushed;
        let instances = self.instances.get_mut();
        instances.retain(|key, _| wanted.contains(key));
        let mut fallback = Vec::new();
        let seconds = if seconds.is_finite() {
            seconds.clamp(0.0, 0.1)
        } else {
            0.0
        };
        for key in wanted {
            if !instances.contains_key(&key) {
                let Some(result) = load(&key) else {
                    fallback.push(key);
                    continue;
                };
                let result = result
                    .and_then(|visual| {
                        let size = visual.size();
                        if size.x.is_finite() && size.y.is_finite() && size.x > 0.0 && size.y > 0.0
                        {
                            Ok(visual)
                        } else {
                            Err("backend returned invalid natural dimensions".into())
                        }
                    })
                    .and_then(|mut visual| {
                        for (id, value) in pushed.get(&key.character).into_iter().flatten() {
                            visual.set_parameter(id, Some(*value))?;
                        }
                        Ok(visual)
                    });
                instances.insert(
                    key.clone(),
                    match result {
                        Ok(visual) => Instance::Ready(visual),
                        Err(error) => {
                            report(&key, &error);
                            Instance::Failed
                        }
                    },
                );
            }
            let instance = instances.get_mut(&key).unwrap();
            if let Instance::Ready(visual) = instance
                && let Err(error) = visual.update(seconds)
            {
                report(&key, &error);
                *instance = Instance::Failed;
            }
            if matches!(instance, Instance::Failed) {
                fallback.push(key);
            }
        }
        fallback
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &self,
        character: &str,
        appearance: &str,
        draw: &mut RaylibDrawHandle,
        layout: Vector2,
        x: f32,
        height: Option<f32>,
        opacity: f32,
    ) -> bool {
        let key = VisualKey::new(character, appearance);
        let mut instances = self.instances.borrow_mut();
        let Some(instance @ Instance::Ready(_)) = instances.get_mut(&key) else {
            return false;
        };
        let Instance::Ready(visual) = instance else {
            unreachable!()
        };
        let render_scale = if crate::frame::viewport::current().is_some() {
            crate::frame::viewport::render_scale()
        } else {
            1.0
        };
        let frame = VisualFrame::placed(visual.size(), layout, x, height, opacity, render_scale);
        match visual.draw(draw, frame) {
            Ok(()) => true,
            Err(error) => {
                report(&key, &error);
                *instance = Instance::Failed;
                false
            }
        }
    }
}

fn report(key: &VisualKey, error: &str) {
    eprintln!(
        "⚠️ Visual {}/{}: {}; using PNG fallback",
        key.character, key.appearance, error
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Trace {
        loaded: Vec<String>,
        updated: Vec<(String, f32)>,
        restarted: Vec<String>,
        dropped: Vec<String>,
        parameters: Vec<(String, String, Option<f32>)>,
    }

    struct Mock {
        name: String,
        trace: Rc<RefCell<Trace>>,
        fail_update: bool,
    }

    impl Mock {
        fn fails_restart(&self) -> bool {
            self.name == "restart_error"
        }
    }

    impl CharacterVisual for Mock {
        fn size(&self) -> Vector2 {
            Vector2::new(200.0, 400.0)
        }
        fn update(&mut self, seconds: f32) -> Result<(), String> {
            self.trace
                .borrow_mut()
                .updated
                .push((self.name.clone(), seconds));
            if self.fail_update {
                Err("update failed".into())
            } else {
                Ok(())
            }
        }
        fn restart(&mut self) -> Result<(), String> {
            self.trace.borrow_mut().restarted.push(self.name.clone());
            if self.fails_restart() {
                Err("restart failed".into())
            } else {
                Ok(())
            }
        }
        fn set_parameter(&mut self, id: &str, value: Option<f32>) -> Result<(), String> {
            self.trace
                .borrow_mut()
                .parameters
                .push((self.name.clone(), id.to_string(), value));
            if id == "unknown" {
                return Err("no such parameter".into());
            }
            Ok(())
        }
        fn draw(&mut self, _: &mut RaylibDrawHandle, _: VisualFrame) -> Result<(), String> {
            Ok(())
        }
    }

    impl Drop for Mock {
        fn drop(&mut self) {
            self.trace.borrow_mut().dropped.push(self.name.clone());
        }
    }

    fn load(
        trace: &Rc<RefCell<Trace>>,
        key: &VisualKey,
    ) -> Option<Result<Box<dyn CharacterVisual>, String>> {
        if key.character == "png" {
            return None;
        }
        trace.borrow_mut().loaded.push(key.appearance.clone());
        if key.appearance == "load_error" {
            return Some(Err("load failed".into()));
        }
        Some(Ok(Box::new(Mock {
            name: key.appearance.clone(),
            trace: Rc::clone(trace),
            fail_update: key.appearance == "update_error",
        })))
    }

    #[test]
    fn appearances_are_loaded_once_and_outgoing_instances_live_until_released() {
        let trace = Rc::new(RefCell::new(Trace::default()));
        let mut visuals = CharacterVisuals::default();
        let old = VisualKey::new("mary", "neutral");
        let new = VisualKey::new("mary", "happy");
        visuals.prepare_with([old.clone(), old.clone()], 0.02, |key| load(&trace, key));
        visuals.prepare_with([old, new.clone()], 0.02, |key| load(&trace, key));
        assert_eq!(trace.borrow().loaded, ["neutral", "happy"]);
        assert!(trace.borrow().dropped.is_empty());
        assert_eq!(trace.borrow().updated.len(), 3);
        visuals.prepare_with([new.clone()], 0.02, |key| load(&trace, key));
        assert_eq!(trace.borrow().dropped, ["neutral"]);
        visuals.reset();
        assert_eq!(trace.borrow().dropped, ["neutral", "happy"]);
        visuals.prepare_with([new], 0.02, |key| load(&trace, key));
        assert_eq!(trace.borrow().loaded, ["neutral", "happy", "happy"]);
    }

    #[test]
    fn failed_loads_and_updates_use_pngs_without_retrying_every_frame() {
        let trace = Rc::new(RefCell::new(Trace::default()));
        let mut visuals = CharacterVisuals::default();
        let keys = [
            VisualKey::new("mary", "load_error"),
            VisualKey::new("mary", "update_error"),
            VisualKey::new("png", "normal"),
        ];
        for _ in 0..3 {
            let fallback = visuals.prepare_with(keys.clone(), 0.02, |key| load(&trace, key));
            assert_eq!(fallback.len(), 3);
        }
        assert_eq!(trace.borrow().loaded.len(), 2);
        assert_eq!(trace.borrow().updated.len(), 1);
        assert_eq!(trace.borrow().dropped, ["update_error"]);
        visuals.reset();
        visuals.prepare_with(keys, 0.02, |key| load(&trace, key));
        assert_eq!(trace.borrow().loaded.len(), 4);
    }

    #[test]
    fn hidden_instances_are_released_and_stalls_do_not_explode_animation_time() {
        let trace = Rc::new(RefCell::new(Trace::default()));
        let mut visuals = CharacterVisuals::default();
        visuals.prepare_with([VisualKey::new("mary", "happy")], 4.0, |key| {
            load(&trace, key)
        });
        assert_eq!(trace.borrow().updated[0].1, 0.1);
        visuals.prepare_with([], 0.02, |key| load(&trace, key));
        assert_eq!(trace.borrow().dropped, ["happy"]);
    }

    #[test]
    fn rolling_back_restarts_the_models_on_screen_instead_of_reloading_them() {
        let trace = Rc::new(RefCell::new(Trace::default()));
        let mut visuals = CharacterVisuals::default();
        let keys = [
            VisualKey::new("mary", "happy"),
            VisualKey::new("hugo", "restart_error"),
        ];
        let sorted = |names: &[String]| {
            let mut names = names.to_vec();
            names.sort();
            names
        };
        visuals.prepare_with(keys.clone(), 0.02, |key| load(&trace, key));
        assert_eq!(sorted(&trace.borrow().loaded), ["happy", "restart_error"]);

        visuals.restart();
        assert_eq!(
            sorted(&trace.borrow().restarted),
            ["happy", "restart_error"]
        );
        assert_eq!(
            trace.borrow().dropped,
            ["restart_error"],
            "only the model that failed to restart is released"
        );

        let fallback = visuals.prepare_with(keys, 0.02, |key| load(&trace, key));
        assert_eq!(sorted(&trace.borrow().loaded), ["happy", "restart_error"]);
        assert_eq!(fallback, [VisualKey::new("hugo", "restart_error")]);
    }

    #[test]
    fn a_pushed_parameter_reaches_every_appearance_the_character_loads() {
        let trace = Rc::new(RefCell::new(Trace::default()));
        let mut visuals = CharacterVisuals::default();
        let neutral = VisualKey::new("mary", "neutral");
        let happy = VisualKey::new("mary", "happy");
        let hugo = VisualKey::new("hugo", "neutral");
        let keys = [neutral.clone(), hugo.clone()];
        visuals.prepare_with(keys.clone(), 0.02, |key| load(&trace, key));

        visuals.set_parameter("mary", "mouth", Some(0.4));
        assert_eq!(
            trace.borrow().parameters,
            [("neutral".to_string(), "mouth".to_string(), Some(0.4))],
            "only the character addressed hears it"
        );

        visuals.prepare_with([happy.clone(), hugo.clone()], 0.02, |key| load(&trace, key));
        assert_eq!(
            trace.borrow().parameters[1],
            ("happy".to_string(), "mouth".to_string(), Some(0.4)),
            "a new appearance is caught up as it loads"
        );

        visuals.set_parameter("mary", "mouth", None);
        visuals.prepare_with([neutral.clone(), hugo.clone()], 0.02, |key| {
            load(&trace, key)
        });
        assert_eq!(
            trace.borrow().parameters[3..],
            [],
            "releasing it stops it reaching the next appearance"
        );

        visuals.set_parameter("mary", "mouth", Some(1.0));
        visuals.restart();
        visuals.prepare_with([happy, hugo], 0.02, |key| load(&trace, key));
        assert!(
            !trace
                .borrow()
                .parameters
                .iter()
                .any(|(name, _, value)| name == "happy" && *value == Some(1.0)),
            "a rollback forgets what the game pushed"
        );
    }

    #[test]
    fn a_parameter_the_backend_refuses_falls_back_to_the_png() {
        let trace = Rc::new(RefCell::new(Trace::default()));
        let mut visuals = CharacterVisuals::default();
        let neutral = VisualKey::new("mary", "neutral");
        let happy = VisualKey::new("mary", "happy");
        visuals.prepare_with([neutral.clone()], 0.02, |key| load(&trace, key));

        visuals.set_parameter("mary", "unknown", Some(1.0));
        let mut fallback = visuals.prepare_with([neutral.clone(), happy.clone()], 0.02, |key| {
            load(&trace, key)
        });
        fallback.sort();
        assert_eq!(
            fallback,
            [happy.clone(), neutral.clone()],
            "the instance that refused it fails, and so does the one that loads holding it"
        );

        let again = visuals.prepare_with([neutral, happy], 0.02, |key| load(&trace, key));
        assert_eq!(again.len(), 2);
        assert_eq!(
            trace.borrow().loaded.len(),
            2,
            "a failed instance is not retried every frame"
        );
    }

    #[test]
    fn a_save_puts_back_what_was_pushed_and_releases_what_was_not() {
        let trace = Rc::new(RefCell::new(Trace::default()));
        let mut visuals = CharacterVisuals::default();
        let mary = VisualKey::new("mary", "neutral");
        let hugo = VisualKey::new("hugo", "neutral");
        visuals.prepare_with([mary.clone(), hugo.clone()], 0.02, |key| load(&trace, key));

        visuals.set_parameter("mary", "mouth", Some(0.4));
        visuals.set_parameter("mary", "blush", Some(1.0));
        let saved = visuals.pushed().clone();
        assert_eq!(
            saved["mary"],
            BTreeMap::from([("blush".to_string(), 1.0), ("mouth".to_string(), 0.4),])
        );

        visuals.set_parameter("mary", "mouth", None);
        visuals.set_parameter("hugo", "blush", Some(0.5));
        trace.borrow_mut().parameters.clear();

        visuals.restore(saved.clone());
        assert_eq!(
            visuals.pushed(),
            &saved,
            "the step's own parameters, exactly"
        );

        let mut told = trace.borrow().parameters.clone();
        told.sort_by(|a, b| {
            (&a.0, &a.1, a.2.map(f32::to_bits)).cmp(&(&b.0, &b.1, b.2.map(f32::to_bits)))
        });
        assert_eq!(
            told,
            [
                ("neutral".to_string(), "blush".to_string(), None),
                ("neutral".to_string(), "blush".to_string(), Some(1.0)),
                ("neutral".to_string(), "mouth".to_string(), Some(0.4)),
            ],
            "mary hears her two again, and hugo is released from one the step never pushed"
        );
    }

    #[test]
    fn restoring_nothing_releases_everything_the_game_had_pushed() {
        let trace = Rc::new(RefCell::new(Trace::default()));
        let mut visuals = CharacterVisuals::default();
        visuals.prepare_with([VisualKey::new("mary", "neutral")], 0.02, |key| {
            load(&trace, key)
        });
        visuals.set_parameter("mary", "mouth", Some(1.0));
        trace.borrow_mut().parameters.clear();

        visuals.restore(VisualParameters::new());
        assert!(visuals.pushed().is_empty());
        assert_eq!(
            trace.borrow().parameters,
            [("neutral".to_string(), "mouth".to_string(), None)],
            "a step before the push releases it"
        );
    }

    #[test]
    fn placement_matches_png_geometry_and_supersampling() {
        let frame = VisualFrame::placed(
            Vector2::new(200.0, 400.0),
            Vector2::new(1280.0, 720.0),
            0.25,
            Some(0.8),
            0.5,
            1.5,
        );
        assert_eq!(frame.rect, Rectangle::new(176.0, 144.0, 288.0, 576.0));
        assert_eq!(frame.target_size, (1920, 1080));
        assert_eq!(frame.opacity, 0.5);
        let natural = VisualFrame::placed(
            Vector2::new(200.0, 400.0),
            Vector2::new(1280.0, 720.0),
            0.5,
            None,
            1.0,
            1.0,
        );
        assert_eq!(natural.rect, Rectangle::new(540.0, 320.0, 200.0, 400.0));
    }
}
