use std::rc::Rc;

use raylib::prelude::*;

use vn_script::SCHEMA_FILE_NAME;

use super::{AppError, DefaultScreens, VnApp};
use crate::data::rollback::Rollback;
use crate::data::session::SeenLines;
use crate::data::settings::{SETTINGS_FILE_NAME, SettingsStore};
use crate::frame::screen_transition::ScreenTransition;
use crate::frame::target::RenderTarget;
use crate::game::audio::Audio;
use crate::game::hot_reload::StoryWatcher;
use crate::game::script_errors::ScriptErrors;
use crate::input::navigation::Navigation;
use crate::screen_manager::ScreenStateManager;

impl VnApp {
    pub fn run(mut self) -> Result<(), AppError> {
        if std::env::args().any(|arg| arg == "--export-schema") {
            let path = self
                .schema_path()
                .unwrap_or_else(|| self.assets.join(SCHEMA_FILE_NAME));
            let written = self.write_schema(&path)?;
            let status = if written { "wrote" } else { "unchanged:" };
            println!("{} {}", status, path.display());
            if let Some(path) = self.export_ui_strings()? {
                println!("wrote {}", path.display());
            }
            return Ok(());
        }

        if cfg!(debug_assertions) {
            for export in [self.export_schema(), self.export_ui_strings()] {
                match export {
                    Ok(Some(path)) => println!("Updated {}", path.display()),
                    Ok(None) => {}
                    Err(e) => eprintln!("⚠️ {}", e),
                }
            }
        }

        let loader = self.loader();
        if cfg!(debug_assertions) {
            println!("Assets: {:?}", loader.assets);
        }
        let (mut story, warnings) = loader.load()?;
        story.set_scene_events(true);
        for warning in &warnings {
            eprintln!("{}", warning);
        }

        let (mut rl, thread) = raylib::init()
            .size(self.width, self.height)
            .title(&self.title)
            .build();
        rl.set_target_fps(self.target_fps);
        rl.set_exit_key(self.exit_key);

        let saves_dir = self.saves_path();
        if cfg!(debug_assertions) {
            println!("Saves: {}", saves_dir.display());
        }
        let mut settings = SettingsStore::load(saves_dir.join(SETTINGS_FILE_NAME));
        let saves = self.saves();

        let known =
            |code: &Option<String>| self.languages.iter().any(|language| language.code == *code);
        if !known(&settings.values.language) {
            eprintln!(
                "⚠️ Language '{}' is not one of this game's; using {}",
                settings.values.language.as_deref().unwrap_or(""),
                self.languages[0].label
            );
            settings.update(|values| values.language = None);
        }
        if let Some(code) = settings.values.language.clone() {
            match crate::game::language::load_catalog(&loader.assets, &code) {
                Ok(catalog) => story.set_catalog(Some(catalog)),
                Err(e) => eprintln!("⚠️ {}; playing in the source language", e),
            }
        }

        self.settings.languages = self.languages.clone();

        let factory = DefaultScreens {
            title: self.title,
            save_menu: Rc::new(self.save_menu),
            text_input: Rc::new(self.text_input),
            pause_menu: Rc::new(self.pause_menu),
            confirm_dialog: Rc::new(self.confirm_dialog),
            settings: Rc::new(self.settings),
            log: Rc::new(self.log),
            keybinds: Rc::new(self.keybinds.clone()),
            rollback: self.rollback.clone(),
            navigation: self.navigation.clone(),
            start: Rc::new(self.start),
            menu: Rc::new(self.menu),
            playing: Rc::new(self.playing),
            overrides: self.overrides,
            overlays: self.overlays,
        };

        let mut manager = ScreenStateManager::with_story(
            &mut rl,
            &thread,
            self.initial_screen,
            Box::new(factory),
            loader.assets.clone(),
            story,
        )
        .map_err(AppError::Screen)?;

        manager.state = self.state;
        manager.commands = Rc::new(self.commands);
        manager.hooks = Rc::new(self.hooks);
        manager.saves = saves;
        manager.characters = self.characters;
        manager.toast_config = self.toast;
        manager.effects = crate::frame::effects::ScreenEffects::new(self.screen_effects);
        manager.rollback = Rollback::new(self.rollback);
        manager.settings = settings;
        manager.close_confirmation = self.close_confirmation;
        manager.tooltip_config = self.tooltips;
        manager.keybind_keys = self.keybinds.open_keys.clone();
        manager.navigation = Navigation::new(self.navigation);
        manager.seen = SeenLines::load(
            manager
                .saves
                .dir()
                .join(crate::data::session::SEEN_FILE_NAME),
        );
        manager.audio = Audio::new(manager.resources.assets().clone(), self.audio);

        for (role, file) in &self.fonts {
            manager.resources.set_font(&mut rl, &thread, *role, file);
        }

        for (name, fragment, amount) in &self.shaders {
            manager.post.load(&mut rl, &thread, name, fragment, *amount);
        }

        for (role, variant, file) in &self.font_variants {
            manager
                .resources
                .set_font_variant(&mut rl, &thread, *role, *variant, file);
        }

        let mut target = RenderTarget::new();
        let mut snapshot = RenderTarget::new();
        let mut transition: Option<ScreenTransition> = None;

        let mut watcher = loader
            .watch_dir()
            .filter(|_| self.hot_reload)
            .map(StoryWatcher::new);

        while !manager.quit_requested() {
            if let Some(watcher) = &mut watcher
                && watcher.poll(rl.get_time())
            {
                match loader.load() {
                    Ok((mut story, warnings)) => {
                        for warning in &warnings {
                            eprintln!("{}", warning);
                        }
                        story.set_scene_events(true);
                        manager.reload_story(story);
                    }
                    Err(e) => {
                        eprintln!("❌ Story not reloaded: {}", e);
                        manager.show_script_errors(ScriptErrors::from_error(&e, &loader.path()));
                    }
                }
            }

            if rl.window_should_close() {
                manager.request_close();
                if manager.quit_requested() {
                    break;
                }
            }

            let screen = (rl.get_screen_width(), rl.get_screen_height());
            let now = rl.get_time();
            let layout = self.design_size.unwrap_or(screen);
            let drawn = (
                (layout.0 as f32 * self.render_scale).round() as i32,
                (layout.1 as f32 * self.render_scale).round() as i32,
            );
            target.resize(&mut rl, &thread, drawn);
            let destination = crate::frame::target::destination(layout, screen);
            if target.frame().is_some() {
                crate::frame::viewport::set_render_scale(self.render_scale);
                crate::frame::viewport::set(crate::frame::viewport::Viewport {
                    size: layout,
                    destination,
                });
            } else {
                crate::frame::viewport::clear();
            }

            manager.update(&mut rl, &thread);

            let starting = manager.take_screen_changed() && target.frame().is_some();
            if starting {
                snapshot.resize(&mut rl, &thread, target.size());
            }
            manager.post.resize(&mut rl, &thread, target.size());

            let source = target.source();
            let snapshot_source = snapshot.source();
            let size = target.size();
            let mut d = rl.begin_drawing(&thread);
            match target.frame_mut() {
                Some(frame) => {
                    if starting {
                        if let Some(previous) = snapshot.frame_mut() {
                            crate::frame::target::copy_into(&mut d, &thread, frame, previous, size);
                        }
                        transition = ScreenTransition::start(self.screen_transition, now);
                    }

                    {
                        let mut t = d.begin_texture_mode(&thread, frame);
                        t.clear_background(self.clear_color);
                        let mut scaled = t.begin_mode2D(Camera2D {
                            offset: Vector2::zero(),
                            target: Vector2::zero(),
                            rotation: 0.0,
                            zoom: self.render_scale,
                        });
                        manager.draw(&mut scaled, &thread);
                    }
                    let shake = manager.effects().offset(now);
                    let shaken = Rectangle::new(
                        destination.x + shake.x,
                        destination.y + shake.y,
                        destination.width,
                        destination.height,
                    );
                    let flash = manager.effects().flash_color(now);
                    let flashing = manager.effects().flash_alpha(now) > 0.0;

                    let shown = manager.post.apply(&mut d, &thread, frame, size, now);

                    d.clear_background(Color::BLACK);
                    d.draw_texture_pro(
                        shown.texture(),
                        source,
                        shaken,
                        Vector2::zero(),
                        0.0,
                        Color::WHITE,
                    );

                    if let Some(active) = transition
                        && let Some(previous) = snapshot.frame()
                    {
                        let alpha = active.previous_alpha(now);
                        if alpha > 0.0 {
                            let mut over = destination;
                            over.x += active.previous_offset(now, destination.width);
                            d.draw_texture_pro(
                                previous.texture(),
                                snapshot_source,
                                over,
                                Vector2::zero(),
                                0.0,
                                Color::WHITE.alpha(alpha),
                            );
                        }
                        let cover = active.cover_alpha(now);
                        if cover > 0.0 {
                            d.draw_rectangle(0, 0, screen.0, screen.1, Color::BLACK.alpha(cover));
                        }
                        if active.finished(now) {
                            transition = None;
                        }
                    }

                    if flashing {
                        d.draw_rectangle(0, 0, screen.0, screen.1, flash);
                    }
                }
                None => {
                    d.clear_background(self.clear_color);
                    manager.draw(&mut d, &thread);
                }
            }
        }

        manager.autosave();
        manager.seen.save();
        Ok(())
    }
}
