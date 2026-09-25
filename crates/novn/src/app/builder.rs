use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;

use raylib::prelude::*;

use novn_script::{SCHEMA_FILE_NAME, VariableDef};

use super::{OverlayBuilder, ScreenBuilder};
use crate::context::GameContext;
use crate::data::assets::EmbeddedFile;
use crate::data::rollback::RollbackConfig;
use crate::data::saves::{Migrations, SaveMigration, Saves};
use crate::data::state::GameState;
use crate::frame::effects::ScreenEffectsConfig;
use crate::frame::screen_transition::ScreenTransitionConfig;
use crate::game::audio::AudioConfig;
use crate::game::characters::{Character, Characters};
use crate::game::commands::{Command, Commands, FromArgs};
use crate::game::hooks::Hooks;
use crate::game::language::Language;
use crate::input::navigation::NavigationConfig;
use crate::overlay::Overlay;
use crate::screen::{Screen, ScreenState};
use crate::screen_manager::CLOSE_MESSAGE;
use crate::screens::confirm::ConfirmConfig;
use crate::screens::keybinds::KeybindsConfig;
use crate::screens::log::LogConfig;
use crate::screens::main_menu::MainMenuConfig;
use crate::screens::pause_menu::PauseMenuConfig;
use crate::screens::playing::PlayingConfig;
use crate::screens::save_menu::SaveMenuConfig;
use crate::screens::settings::SettingsConfig;
use crate::screens::start::StartScreenConfig;
use crate::screens::text_input::TextInputConfig;
use crate::ui::fonts::{FontRole, FontVariant};
use crate::ui::theme::Theme;
use crate::ui::toast::ToastConfig;
use crate::ui::tooltip::TooltipConfig;

pub struct VnApp {
    #[cfg(feature = "character-visuals")]
    pub(super) visuals: crate::game::visuals::VisualRegistry,
    pub(super) title: String,
    pub(super) width: i32,
    pub(super) height: i32,
    pub(super) target_fps: u32,
    pub(super) clear_color: Color,
    pub(super) assets: PathBuf,
    pub(super) embedded: Option<&'static [EmbeddedFile]>,
    pub(super) story_dir: String,
    pub(super) schema_file: Option<PathBuf>,
    pub(super) initial_screen: ScreenState,
    pub(super) fonts: Vec<(FontRole, String)>,
    pub(super) language_fonts: Vec<(String, FontRole, String)>,
    pub(super) prompts: crate::input::prompts::Prompts,
    pub(super) cursor: Option<crate::ui::cursor::CursorStyle>,
    pub(super) cursor_shapes: bool,
    pub(super) font_variants: Vec<(FontRole, FontVariant, String)>,
    pub(super) start: StartScreenConfig,
    pub(super) menu: MainMenuConfig,
    pub(super) playing: PlayingConfig,
    pub(super) styled: bool,
    pub(super) overrides: HashMap<ScreenState, ScreenBuilder>,
    pub(super) overlays: HashMap<String, OverlayBuilder>,
    pub(super) state: GameState,
    pub(super) persistent: crate::data::persistent::Persistent,
    pub(super) commands: Commands,
    pub(super) hooks: Hooks,
    pub(super) saves_dir: Option<PathBuf>,
    pub(super) save_version: u32,
    pub(super) migrations: Migrations,
    pub(super) autosave: bool,
    pub(super) audio: AudioConfig,
    pub(super) tooltips: TooltipConfig,
    pub(super) log: LogConfig,
    pub(super) keybinds: KeybindsConfig,
    pub(super) navigation: NavigationConfig,
    pub(super) save_menu: SaveMenuConfig,
    pub(super) text_input: TextInputConfig,
    pub(super) pause_menu: PauseMenuConfig,
    pub(super) confirm_dialog: ConfirmConfig,
    pub(super) settings: SettingsConfig,
    pub(super) languages: Vec<Language>,
    pub(super) extra_ui_strings: Vec<String>,
    pub(super) close_confirmation: Option<String>,
    pub(super) rollback: RollbackConfig,
    pub(super) toast: ToastConfig,
    pub(super) exit_key: Option<KeyboardKey>,
    pub(super) entry_scene: Option<String>,
    pub(super) variables: BTreeMap<String, VariableDef>,
    pub(super) characters: Characters,
    pub(super) warn_missing_art: bool,
    pub(super) hot_reload: bool,
    pub(super) dev_tools: bool,
    pub(super) screen_transition: ScreenTransitionConfig,
    pub(super) design_size: Option<(i32, i32)>,
    pub(super) screen_effects: ScreenEffectsConfig,
    pub(super) shaders: Vec<(String, String, f32)>,
    pub(super) render_scale: f32,
}

impl VnApp {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            #[cfg(feature = "character-visuals")]
            visuals: Default::default(),
            title: title.into(),
            width: 1280,
            height: 720,
            target_fps: 60,
            clear_color: Color::BLACK,
            assets: PathBuf::from("assets"),
            embedded: None,
            story_dir: "story".to_string(),
            schema_file: Some(PathBuf::from(SCHEMA_FILE_NAME)),
            initial_screen: ScreenState::StartScreen,
            fonts: Vec::new(),
            language_fonts: Vec::new(),
            prompts: Default::default(),
            cursor: None,
            cursor_shapes: true,
            font_variants: Vec::new(),
            start: StartScreenConfig::default(),
            menu: MainMenuConfig::default(),
            playing: PlayingConfig::default(),
            styled: false,
            overrides: HashMap::new(),
            overlays: HashMap::new(),
            state: GameState::default(),
            persistent: crate::data::persistent::Persistent::in_memory(),
            commands: Commands::default(),
            hooks: Hooks::default(),
            saves_dir: None,
            save_version: 0,
            migrations: Migrations::default(),
            autosave: true,
            audio: AudioConfig::default(),
            tooltips: TooltipConfig::default(),
            log: LogConfig::default(),
            keybinds: KeybindsConfig::default(),
            navigation: NavigationConfig::default(),
            save_menu: SaveMenuConfig::default(),
            text_input: TextInputConfig::default(),
            pause_menu: PauseMenuConfig::default(),
            confirm_dialog: ConfirmConfig::default(),
            settings: SettingsConfig::default(),
            close_confirmation: Some(CLOSE_MESSAGE.to_string()),
            rollback: RollbackConfig::default(),
            toast: ToastConfig::default(),
            exit_key: None,
            entry_scene: None,
            variables: BTreeMap::new(),
            characters: Characters::default(),
            warn_missing_art: true,
            hot_reload: cfg!(debug_assertions),
            dev_tools: cfg!(debug_assertions),
            screen_transition: ScreenTransitionConfig::default(),
            design_size: None,
            screen_effects: ScreenEffectsConfig::default(),
            shaders: Vec::new(),
            render_scale: 1.0,
            languages: vec![Language::source("English")],
            extra_ui_strings: Vec::new(),
        }
    }

    pub fn ui_text(mut self, text: impl Into<String>) -> Self {
        self.extra_ui_strings.push(text.into());
        self
    }

    pub fn source_language(mut self, label: impl Into<String>) -> Self {
        self.languages[0] = Language::source(label);
        self
    }

    pub fn language(mut self, code: impl Into<String>, label: impl Into<String>) -> Self {
        let language = Language::new(code, label);
        match self.languages.iter().position(|l| l.code == language.code) {
            Some(index) => self.languages[index] = language,
            None => self.languages.push(language),
        }
        self
    }

    pub fn languages(&self) -> &[Language] {
        &self.languages
    }

    pub fn font_variant(
        mut self,
        role: FontRole,
        variant: FontVariant,
        file: impl Into<String>,
    ) -> Self {
        self.font_variants.push((role, variant, file.into()));
        self
    }

    pub fn render_scale(mut self, scale: f32) -> Self {
        self.render_scale = scale.clamp(1.0, 4.0);
        self
    }

    pub fn shader(mut self, name: impl Into<String>, fragment: impl Into<String>) -> Self {
        self.shaders.push((name.into(), fragment.into(), 1.0));
        self
    }

    pub fn shader_amount(
        mut self,
        name: impl Into<String>,
        fragment: impl Into<String>,
        amount: f32,
    ) -> Self {
        self.shaders.push((name.into(), fragment.into(), amount));
        self
    }

    pub fn screen_effects(mut self, config: ScreenEffectsConfig) -> Self {
        self.screen_effects = config;
        self
    }

    pub fn design_size(mut self, width: i32, height: i32) -> Self {
        self.design_size = Some((width, height));
        self
    }

    pub fn screen_transition(mut self, transition: ScreenTransitionConfig) -> Self {
        self.screen_transition = transition;
        self
    }

    pub fn entry_scene(mut self, scene: impl Into<String>) -> Self {
        self.entry_scene = Some(scene.into());
        self
    }

    pub fn variable(mut self, name: impl Into<String>, definition: VariableDef) -> Self {
        self.variables.insert(name.into(), definition);
        self
    }

    pub fn character(mut self, id: impl Into<String>, character: Character) -> Self {
        self.characters.insert(id, character);
        self
    }

    #[cfg(feature = "character-visuals")]
    pub fn character_visual(
        mut self,
        id: impl Into<String>,
        factory: impl crate::game::visuals::CharacterVisualFactory + 'static,
    ) -> Self {
        self.visuals.insert(id.into(), factory);
        self
    }

    pub fn text_input(mut self, config: impl FnOnce(TextInputConfig) -> TextInputConfig) -> Self {
        self.styled = true;
        self.text_input = config(self.text_input);
        self
    }

    pub fn pause_menu(mut self, config: impl FnOnce(PauseMenuConfig) -> PauseMenuConfig) -> Self {
        self.styled = true;
        self.pause_menu = config(self.pause_menu);
        self
    }

    pub fn confirm_dialog(mut self, config: impl FnOnce(ConfirmConfig) -> ConfirmConfig) -> Self {
        self.styled = true;
        self.confirm_dialog = config(self.confirm_dialog);
        self
    }

    pub fn settings(mut self, config: impl FnOnce(SettingsConfig) -> SettingsConfig) -> Self {
        self.styled = true;
        self.settings = config(self.settings);
        self
    }

    pub fn confirm_on_close(mut self, message: Option<&str>) -> Self {
        self.close_confirmation = message.map(str::to_string);
        self
    }

    pub fn rollback(mut self, config: impl FnOnce(RollbackConfig) -> RollbackConfig) -> Self {
        self.rollback = config(self.rollback);
        self
    }

    pub fn toast(mut self, config: impl FnOnce(ToastConfig) -> ToastConfig) -> Self {
        self.styled = true;
        self.toast = config(self.toast);
        self
    }

    pub fn exit_key(mut self, key: Option<KeyboardKey>) -> Self {
        self.exit_key = key;
        self
    }

    pub fn warn_missing_art(mut self, warn: bool) -> Self {
        self.warn_missing_art = warn;
        self
    }

    pub fn hot_reload(mut self, enabled: bool) -> Self {
        self.hot_reload = enabled;
        self
    }

    pub fn dev_tools(mut self, enabled: bool) -> Self {
        self.dev_tools = enabled;
        self
    }

    pub fn saves_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.saves_dir = Some(dir.into());
        self
    }

    pub fn saves_path(&self) -> PathBuf {
        self.saves_dir
            .clone()
            .unwrap_or_else(|| crate::data::saves::default_saves_dir(&self.title))
    }

    pub fn audio(mut self, config: impl FnOnce(AudioConfig) -> AudioConfig) -> Self {
        self.audio = config(self.audio);
        self
    }

    pub fn navigation(mut self, config: impl FnOnce(NavigationConfig) -> NavigationConfig) -> Self {
        self.navigation = config(self.navigation);
        self
    }

    pub fn keybinds(mut self, config: impl FnOnce(KeybindsConfig) -> KeybindsConfig) -> Self {
        self.styled = true;
        self.keybinds = config(self.keybinds);
        self
    }

    pub fn log(mut self, config: impl FnOnce(LogConfig) -> LogConfig) -> Self {
        self.styled = true;
        self.log = config(self.log);
        self
    }

    pub fn tooltips(mut self, config: impl FnOnce(TooltipConfig) -> TooltipConfig) -> Self {
        self.styled = true;
        self.tooltips = config(self.tooltips);
        self
    }

    pub fn save_version(mut self, version: u32) -> Self {
        self.save_version = version;
        self
    }

    pub fn migrate_save(
        mut self,
        from: u32,
        migration: impl Fn(&mut SaveMigration) -> Result<(), String> + 'static,
    ) -> Self {
        self.migrations.add(from, migration);
        self
    }

    pub fn saves(&self) -> Saves {
        Saves::new(self.saves_path(), self.title.clone())
            .with_version(self.save_version)
            .with_migrations(self.migrations.clone())
            .with_autosave(self.autosave)
    }

    pub fn autosave(mut self, enabled: bool) -> Self {
        self.autosave = enabled;
        self
    }

    pub fn save_menu(mut self, config: impl FnOnce(SaveMenuConfig) -> SaveMenuConfig) -> Self {
        self.styled = true;
        self.save_menu = config(self.save_menu);
        self
    }

    pub fn size(mut self, width: i32, height: i32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn target_fps(mut self, fps: u32) -> Self {
        self.target_fps = fps;
        self
    }

    pub fn clear_color(mut self, color: Color) -> Self {
        self.clear_color = color;
        self
    }

    pub fn assets(mut self, root: impl Into<PathBuf>) -> Self {
        self.assets = root.into();
        self
    }

    pub fn embedded_assets(mut self, files: &'static [EmbeddedFile]) -> Self {
        self.embedded = Some(files).filter(|files| !files.is_empty());
        self
    }

    pub fn schema_file(mut self, file: Option<&str>) -> Self {
        self.schema_file = file.map(PathBuf::from);
        self
    }

    pub fn story_dir(mut self, dir: impl Into<String>) -> Self {
        self.story_dir = dir.into();
        self
    }

    pub fn initial_screen(mut self, state: ScreenState) -> Self {
        self.initial_screen = state;
        self
    }

    pub fn font(mut self, role: FontRole, file: impl Into<String>) -> Self {
        self.fonts.push((role, file.into()));
        self
    }

    pub fn pad_label(
        mut self,
        family: crate::input::pad::PadFamily,
        button: crate::input::pad::PadButton,
        label: impl Into<String>,
    ) -> Self {
        self.prompts.pad_labels.set(family, button, label);
        self
    }

    pub fn cursor(mut self, style: crate::ui::cursor::CursorStyle) -> Self {
        self.cursor = Some(style);
        self
    }

    pub fn cursor_shapes(mut self, on: bool) -> Self {
        self.cursor_shapes = on;
        self
    }

    pub fn prompt(
        mut self,
        token: impl Into<String>,
        prompt: crate::input::prompts::Prompt,
    ) -> Self {
        self.prompts.insert(token, prompt);
        self
    }

    pub fn language_font(
        mut self,
        code: impl Into<String>,
        role: FontRole,
        file: impl Into<String>,
    ) -> Self {
        self.language_fonts.push((code.into(), role, file.into()));
        self
    }

    pub fn start_screen(
        mut self,
        config: impl FnOnce(StartScreenConfig) -> StartScreenConfig,
    ) -> Self {
        self.styled = true;
        self.start = config(self.start);
        self
    }

    pub fn main_menu(mut self, config: impl FnOnce(MainMenuConfig) -> MainMenuConfig) -> Self {
        self.styled = true;
        self.menu = config(self.menu);
        self
    }

    pub fn playing(mut self, config: impl FnOnce(PlayingConfig) -> PlayingConfig) -> Self {
        self.styled = true;
        self.playing = config(self.playing);
        self
    }

    pub fn screen<S: Screen + 'static>(
        mut self,
        state: ScreenState,
        build: impl Fn() -> S + 'static,
    ) -> Self {
        self.overrides
            .insert(state, Box::new(move || Box::new(build())));
        self
    }

    pub fn overlay<O: Overlay + 'static>(
        mut self,
        name: impl Into<String>,
        build: impl Fn() -> O + 'static,
    ) -> Self {
        self.overlays
            .insert(name.into(), Box::new(move || Box::new(build())));
        self
    }

    pub fn state<T>(mut self, initial: T) -> Self
    where
        T: Clone + serde::Serialize + serde::de::DeserializeOwned + 'static,
    {
        self.state.insert(initial);
        self
    }

    pub fn persistent<T>(mut self, initial: T) -> Self
    where
        T: Clone + serde::Serialize + serde::de::DeserializeOwned + 'static,
    {
        self.persistent.insert(initial);
        self
    }

    pub fn with(self, setup: impl FnOnce(Self) -> Self) -> Self {
        setup(self)
    }

    pub fn theme(mut self, theme: impl FnOnce(Theme) -> Theme) -> Self {
        assert!(
            !self.styled,
            "`theme` is what the default screens start from, so it comes before them; \
             move it above the first screen this game configures"
        );

        let theme = theme(Theme::default());
        self.start = std::mem::take(&mut self.start).themed(&theme);
        self.menu = std::mem::take(&mut self.menu).themed(&theme);
        self.playing = std::mem::take(&mut self.playing).themed(&theme);
        self.pause_menu = std::mem::take(&mut self.pause_menu).themed(&theme);
        self.confirm_dialog = std::mem::take(&mut self.confirm_dialog).themed(&theme);
        self.save_menu = std::mem::take(&mut self.save_menu).themed(&theme);
        self.settings = std::mem::take(&mut self.settings).themed(&theme);
        self.text_input = std::mem::take(&mut self.text_input).themed(&theme);
        self.keybinds = std::mem::take(&mut self.keybinds).themed(&theme);
        self.log = std::mem::take(&mut self.log).themed(&theme);
        self.tooltips = std::mem::take(&mut self.tooltips).themed(&theme);
        self.toast = std::mem::take(&mut self.toast).themed(&theme);
        self
    }

    pub fn command<C: Command>(mut self, command: C) -> Self {
        let _ = command;
        self.commands
            .insert(C::NAME, |context, arguments| C::run(context, arguments));
        self
    }

    pub fn command_as<A: FromArgs + 'static>(
        mut self,
        name: impl Into<String>,
        handler: impl Fn(&mut GameContext, A) -> Option<ScreenState> + 'static,
    ) -> Self {
        self.commands.insert(name, handler);
        self
    }

    pub fn on_scene_enter(
        mut self,
        hook: impl Fn(&mut GameContext, &str) -> Option<ScreenState> + 'static,
    ) -> Self {
        self.hooks.on_scene_enter(hook);
        self
    }

    pub fn on_choice(
        mut self,
        hook: impl Fn(&mut GameContext, usize, &str) -> Option<ScreenState> + 'static,
    ) -> Self {
        self.hooks.on_choice(hook);
        self
    }

    pub fn on_frame(mut self, hook: impl Fn(&mut GameContext, f32) + 'static) -> Self {
        self.hooks.on_frame(hook);
        self
    }

    pub fn hooks(&self) -> &Hooks {
        &self.hooks
    }
}
