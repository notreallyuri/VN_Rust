use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use raylib::prelude::*;

use vn_script::{
    Diagnostic, Instruction, SCHEMA_FILE_NAME, Schema, SchemaFile, StoryVm, VariableDef,
};

use crate::screens::{
    CONFIRM_OVERLAY, ConfirmConfig, ConfirmDialog, LOAD_OVERLAY, MainMenuConfig, MainMenuScreen,
    PAUSE_OVERLAY, PauseMenu, PauseMenuConfig, PlayingConfig, PlayingScreen, SAVE_OVERLAY,
    SETTINGS_OVERLAY, SaveMenuConfig, SaveMenuMode, SaveMenuOverlay, SaveMenuScreen,
    SettingsConfig, SettingsOverlay, SettingsScreen, StartScreen, StartScreenConfig,
    TextInputConfig, TextInputScreen,
};
use crate::{
    Audio, AudioConfig, CLOSE_MESSAGE, Character, Characters, Commands, FontRole, FromArgs,
    GameContext, GameState, Hooks, Navigation, NavigationConfig, Overlay, Rollback, RollbackConfig,
    SETTINGS_FILE_NAME, Saves, Screen, ScreenFactory, ScreenState, ScreenStateManager,
    ScriptErrors, SettingsStore, StoryLoader, StoryWatcher, ToastConfig, TooltipConfig,
};

type ScreenBuilder = Box<dyn Fn() -> Box<dyn Screen>>;
type OverlayBuilder = Box<dyn Fn() -> Box<dyn Overlay>>;

pub struct VnApp {
    title: String,
    width: i32,
    height: i32,
    target_fps: u32,
    clear_color: Color,
    assets: PathBuf,
    story_dir: String,
    schema_file: Option<PathBuf>,
    initial_screen: ScreenState,
    fonts: Vec<(FontRole, String)>,
    start: StartScreenConfig,
    menu: MainMenuConfig,
    playing: PlayingConfig,
    overrides: HashMap<ScreenState, ScreenBuilder>,
    overlays: HashMap<String, OverlayBuilder>,
    state: GameState,
    commands: Commands,
    hooks: Hooks,
    saves_dir: Option<PathBuf>,
    autosave: bool,
    audio: AudioConfig,
    tooltips: TooltipConfig,
    navigation: NavigationConfig,
    save_menu: SaveMenuConfig,
    text_input: TextInputConfig,
    pause_menu: PauseMenuConfig,
    confirm_dialog: ConfirmConfig,
    settings: SettingsConfig,
    close_confirmation: Option<String>,
    rollback: RollbackConfig,
    toast: ToastConfig,
    exit_key: Option<KeyboardKey>,
    entry_scene: Option<String>,
    variables: BTreeMap<String, VariableDef>,
    characters: Characters,
    warn_missing_art: bool,
    hot_reload: bool,
}

#[derive(Debug)]
pub enum AppError {
    Story {
        path: PathBuf,
        source: io::Error,
    },
    Script {
        path: PathBuf,
        errors: Vec<Diagnostic>,
    },
    Schema {
        path: PathBuf,
        source: io::Error,
    },
    Screen(io::Error),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Story { source, .. } => write!(f, "could not load the story: {}", source),
            AppError::Script { path, errors } => {
                let count = errors.len();
                write!(
                    f,
                    "{} has {} error{}:",
                    path.display(),
                    count,
                    if count == 1 { "" } else { "s" }
                )?;
                for error in errors {
                    write!(f, "\n  {}", error)?;
                }
                Ok(())
            }
            AppError::Schema { source, .. } => {
                write!(f, "could not write the schema: {}", source)
            }
            AppError::Screen(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for AppError {}

impl VnApp {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            width: 1280,
            height: 720,
            target_fps: 60,
            clear_color: Color::BLACK,
            assets: PathBuf::from("assets"),
            story_dir: "story".to_string(),
            schema_file: Some(PathBuf::from(SCHEMA_FILE_NAME)),
            initial_screen: ScreenState::StartScreen,
            fonts: Vec::new(),
            start: StartScreenConfig::default(),
            menu: MainMenuConfig::default(),
            playing: PlayingConfig::default(),
            overrides: HashMap::new(),
            overlays: HashMap::new(),
            state: GameState::default(),
            commands: Commands::default(),
            hooks: Hooks::default(),
            saves_dir: None,
            autosave: true,
            audio: AudioConfig::default(),
            tooltips: TooltipConfig::default(),
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
        }
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

    pub fn text_input(mut self, config: impl FnOnce(TextInputConfig) -> TextInputConfig) -> Self {
        self.text_input = config(self.text_input);
        self
    }

    pub fn pause_menu(mut self, config: impl FnOnce(PauseMenuConfig) -> PauseMenuConfig) -> Self {
        self.pause_menu = config(self.pause_menu);
        self
    }

    pub fn confirm_dialog(mut self, config: impl FnOnce(ConfirmConfig) -> ConfirmConfig) -> Self {
        self.confirm_dialog = config(self.confirm_dialog);
        self
    }

    pub fn settings(mut self, config: impl FnOnce(SettingsConfig) -> SettingsConfig) -> Self {
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

    pub fn schema(&self) -> Schema {
        Schema {
            variables: self.variables.clone(),
            characters: self
                .characters
                .iter()
                .map(|(id, c)| (id.to_string(), c.definition()))
                .collect(),
            commands: self.commands.signatures().clone(),
        }
    }

    pub fn loader(&self) -> StoryLoader {
        StoryLoader {
            assets: self.assets.clone(),
            story_dir: PathBuf::from(&self.story_dir),
            schema: self.schema(),
            entry_scene: self.entry_scene.clone(),
            warn_missing_art: self.warn_missing_art,
        }
    }

    pub fn check(&self) -> Result<(StoryVm, Vec<Diagnostic>), AppError> {
        self.loader().load()
    }

    pub fn hot_reload(mut self, enabled: bool) -> Self {
        self.hot_reload = enabled;
        self
    }

    pub fn saves_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.saves_dir = Some(dir.into());
        self
    }

    pub fn saves_path(&self) -> PathBuf {
        self.saves_dir
            .clone()
            .unwrap_or_else(|| crate::default_saves_dir(&self.title))
    }

    pub fn audio(mut self, config: impl FnOnce(AudioConfig) -> AudioConfig) -> Self {
        self.audio = config(self.audio);
        self
    }

    pub fn navigation(mut self, config: impl FnOnce(NavigationConfig) -> NavigationConfig) -> Self {
        self.navigation = config(self.navigation);
        self
    }

    pub fn tooltips(mut self, config: impl FnOnce(TooltipConfig) -> TooltipConfig) -> Self {
        self.tooltips = config(self.tooltips);
        self
    }

    pub fn autosave(mut self, enabled: bool) -> Self {
        self.autosave = enabled;
        self
    }

    pub fn save_menu(mut self, config: impl FnOnce(SaveMenuConfig) -> SaveMenuConfig) -> Self {
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

    pub fn schema_file(mut self, file: Option<&str>) -> Self {
        self.schema_file = file.map(PathBuf::from);
        self
    }

    pub fn schema_export(&self) -> SchemaFile {
        SchemaFile {
            entry_scene: self.entry_scene.clone(),
            ..SchemaFile::new(&self.title, &self.story_dir, self.schema())
        }
    }

    pub fn schema_path(&self) -> Option<PathBuf> {
        self.schema_file.as_ref().map(|file| self.assets.join(file))
    }

    pub fn export_schema(&self) -> Result<Option<PathBuf>, AppError> {
        let Some(path) = self.schema_path() else {
            return Ok(None);
        };
        Ok(self.write_schema(&path)?.then_some(path))
    }

    fn write_schema(&self, path: &Path) -> Result<bool, AppError> {
        self.schema_export()
            .write(path)
            .map_err(|source| AppError::Schema {
                path: path.to_path_buf(),
                source,
            })
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

    pub fn start_screen(
        mut self,
        config: impl FnOnce(StartScreenConfig) -> StartScreenConfig,
    ) -> Self {
        self.start = config(self.start);
        self
    }

    pub fn main_menu(mut self, config: impl FnOnce(MainMenuConfig) -> MainMenuConfig) -> Self {
        self.menu = config(self.menu);
        self
    }

    pub fn playing(mut self, config: impl FnOnce(PlayingConfig) -> PlayingConfig) -> Self {
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

    pub fn command<A: FromArgs + 'static>(
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

    pub fn hooks(&self) -> &Hooks {
        &self.hooks
    }

    pub fn run(self) -> Result<(), AppError> {
        if std::env::args().any(|arg| arg == "--export-schema") {
            let path = self
                .schema_path()
                .unwrap_or_else(|| self.assets.join(SCHEMA_FILE_NAME));
            let written = self.write_schema(&path)?;
            let status = if written { "wrote" } else { "unchanged:" };
            println!("{} {}", status, path.display());
            return Ok(());
        }

        if cfg!(debug_assertions) {
            match self.export_schema() {
                Ok(Some(path)) => println!("Updated {}", path.display()),
                Ok(None) => {}
                Err(e) => eprintln!("⚠️ {}", e),
            }
        }

        let loader = self.loader();
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
        let settings = SettingsStore::load(saves_dir.join(SETTINGS_FILE_NAME));
        let saves = Saves::new(saves_dir, self.title.clone()).with_autosave(self.autosave);

        let factory = DefaultScreens {
            title: self.title,
            save_menu: Rc::new(self.save_menu),
            text_input: Rc::new(self.text_input),
            pause_menu: Rc::new(self.pause_menu),
            confirm_dialog: Rc::new(self.confirm_dialog),
            settings: Rc::new(self.settings),
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
            self.assets,
            story,
        )
        .map_err(AppError::Screen)?;

        manager.state = self.state;
        manager.commands = Rc::new(self.commands);
        manager.hooks = Rc::new(self.hooks);
        manager.saves = saves;
        manager.characters = self.characters;
        manager.toast_config = self.toast;
        manager.rollback = Rollback::new(self.rollback);
        manager.settings = settings;
        manager.close_confirmation = self.close_confirmation;
        manager.tooltip_config = self.tooltips;
        manager.navigation = Navigation::new(self.navigation);
        manager.audio = Audio::new(manager.resources.root().to_path_buf(), self.audio);

        for (role, file) in &self.fonts {
            manager.resources.set_font(&mut rl, &thread, *role, file);
        }

        let mut watcher = self.hot_reload.then(|| StoryWatcher::new(loader.path()));

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

            manager.update(&mut rl, &thread);

            let mut d = rl.begin_drawing(&thread);
            d.clear_background(self.clear_color);
            manager.draw(&mut d, &thread);
        }

        manager.autosave();
        Ok(())
    }
}

pub struct DefaultScreens {
    pub title: String,
    pub start: Rc<StartScreenConfig>,
    pub menu: Rc<MainMenuConfig>,
    pub playing: Rc<PlayingConfig>,
    pub save_menu: Rc<SaveMenuConfig>,
    pub text_input: Rc<TextInputConfig>,
    pub pause_menu: Rc<PauseMenuConfig>,
    pub confirm_dialog: Rc<ConfirmConfig>,
    pub settings: Rc<SettingsConfig>,
    pub overrides: HashMap<ScreenState, ScreenBuilder>,
    pub overlays: HashMap<String, OverlayBuilder>,
}

impl ScreenFactory for DefaultScreens {
    fn create_screen(&self, state: &ScreenState) -> Option<Box<dyn Screen>> {
        if let Some(build) = self.overrides.get(state) {
            return Some(build());
        }

        match state {
            ScreenState::StartScreen => Some(Box::new(StartScreen::new(self.start.clone()))),
            ScreenState::MainMenu => Some(Box::new(MainMenuScreen::new(
                self.menu.clone(),
                &self.title,
            ))),
            ScreenState::Playing => Some(Box::new(PlayingScreen::new(self.playing.clone()))),
            ScreenState::Save => Some(Box::new(SaveMenuScreen::new(
                self.save_menu.clone(),
                SaveMenuMode::Save,
            ))),
            ScreenState::Load => Some(Box::new(SaveMenuScreen::new(
                self.save_menu.clone(),
                SaveMenuMode::Load,
            ))),
            ScreenState::TextInput => Some(Box::new(TextInputScreen::new(self.text_input.clone()))),
            ScreenState::Settings => Some(Box::new(SettingsScreen::new(self.settings.clone()))),
            _ => None,
        }
    }

    fn create_overlay(&self, name: &str) -> Option<Box<dyn Overlay>> {
        if let Some(build) = self.overlays.get(name) {
            return Some(build());
        }

        match name {
            PAUSE_OVERLAY => Some(Box::new(PauseMenu::new(self.pause_menu.clone()))),
            CONFIRM_OVERLAY => Some(Box::new(ConfirmDialog::new(self.confirm_dialog.clone()))),
            SETTINGS_OVERLAY => Some(Box::new(SettingsOverlay::new(self.settings.clone()))),
            SAVE_OVERLAY => Some(Box::new(SaveMenuOverlay::new(
                self.save_menu.clone(),
                SaveMenuMode::Save,
            ))),
            LOAD_OVERLAY => Some(Box::new(SaveMenuOverlay::new(
                self.save_menu.clone(),
                SaveMenuMode::Load,
            ))),
            _ => None,
        }
    }
}

pub(crate) fn missing_art(story: &StoryVm, assets: &Path) -> Vec<Diagnostic> {
    let program = story.program();
    let mut seen = std::collections::HashSet::new();

    program
        .instructions
        .iter()
        .enumerate()
        .filter_map(|(index, instruction)| {
            let relative = match instruction {
                Instruction::Show {
                    char_id, img_id, ..
                } => crate::character_path(char_id, img_id),
                Instruction::Background { image: Some(image) } => crate::background_path(image),
                Instruction::Music { track: Some(track) }
                    if crate::music_path(assets, track).is_none() =>
                {
                    format!("music/{}.ogg", track)
                }
                Instruction::Sound { id } if crate::sound_path(assets, id).is_none() => {
                    format!("sounds/{}.ogg", id)
                }
                _ => return None,
            };
            let missing = !assets.join(&relative).exists() && seen.insert(relative.clone());
            missing.then(|| {
                Diagnostic::warning(
                    program.line(index),
                    if relative.ends_with(".png") {
                        format!("missing {} (a placeholder will be drawn)", relative)
                    } else {
                        format!(
                            "missing {} (or .mp3, .wav, .flac); it will be silent",
                            relative
                        )
                    },
                )
                .with_file(program.file(index))
            })
        })
        .collect()
}
