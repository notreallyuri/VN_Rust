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
    SaveMenuConfig, SaveMenuMode, SaveMenuOverlay, SaveMenuScreen, StartScreen, StartScreenConfig,
    TextInputConfig, TextInputScreen,
};
use crate::{
    Character, Characters, Commands, FontRole, FromArgs, GameContext, GameState, Overlay, Rollback,
    RollbackConfig, Saves, Screen, ScreenFactory, ScreenState, ScreenStateManager, ToastConfig,
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
    saves_dir: PathBuf,
    save_menu: SaveMenuConfig,
    text_input: TextInputConfig,
    pause_menu: PauseMenuConfig,
    confirm_dialog: ConfirmConfig,
    rollback: RollbackConfig,
    toast: ToastConfig,
    exit_key: Option<KeyboardKey>,
    entry_scene: Option<String>,
    variables: BTreeMap<String, VariableDef>,
    characters: Characters,
    warn_missing_art: bool,
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
            saves_dir: PathBuf::from("saves"),
            save_menu: SaveMenuConfig::default(),
            text_input: TextInputConfig::default(),
            pause_menu: PauseMenuConfig::default(),
            confirm_dialog: ConfirmConfig::default(),
            rollback: RollbackConfig::default(),
            toast: ToastConfig::default(),
            exit_key: None,
            entry_scene: None,
            variables: BTreeMap::new(),
            characters: Characters::default(),
            warn_missing_art: true,
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

    pub fn check(&self) -> Result<(StoryVm, Vec<Diagnostic>), AppError> {
        let path = self.assets.join(&self.story_dir);
        let mut story = StoryVm::from_dir(&path).map_err(|source| AppError::Story {
            path: path.clone(),
            source,
        })?;

        if story.program().files.is_empty() {
            return Err(AppError::Story {
                source: io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("no .story files in {}", path.display()),
                ),
                path,
            });
        }

        let mut diagnostics = story.prepare(self.schema(), self.entry_scene.as_deref());
        if self.warn_missing_art {
            diagnostics.extend(missing_art(&story, &self.assets));
        }

        let (errors, warnings): (Vec<_>, Vec<_>) =
            diagnostics.into_iter().partition(Diagnostic::is_error);

        if errors.is_empty() {
            Ok((story, warnings))
        } else {
            Err(AppError::Script { path, errors })
        }
    }

    pub fn saves_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.saves_dir = dir.into();
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

        let (story, warnings) = self.check()?;
        for warning in &warnings {
            eprintln!("{}", warning);
        }

        let (mut rl, thread) = raylib::init()
            .size(self.width, self.height)
            .title(&self.title)
            .build();
        rl.set_target_fps(self.target_fps);
        rl.set_exit_key(self.exit_key);

        let saves = Saves::new(self.saves_dir, self.title.clone());

        let factory = DefaultScreens {
            title: self.title,
            save_menu: Rc::new(self.save_menu),
            text_input: Rc::new(self.text_input),
            pause_menu: Rc::new(self.pause_menu),
            confirm_dialog: Rc::new(self.confirm_dialog),
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
        manager.saves = saves;
        manager.characters = self.characters;
        manager.toast_config = self.toast;
        manager.rollback = Rollback::new(self.rollback);

        for (role, file) in &self.fonts {
            manager.resources.set_font(&mut rl, &thread, *role, file);
        }

        while !rl.window_should_close() && !manager.quit_requested() {
            manager.update(&mut rl, &thread);

            let mut d = rl.begin_drawing(&thread);
            d.clear_background(self.clear_color);
            manager.draw(&mut d);
        }

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

fn missing_art(story: &StoryVm, assets: &Path) -> Vec<Diagnostic> {
    let program = story.program();
    let mut seen = std::collections::HashSet::new();

    program
        .instructions
        .iter()
        .enumerate()
        .filter_map(|(index, instruction)| match instruction {
            Instruction::Show { char_id, img_id } => {
                let relative = format!("characters/{}/{}.png", char_id, img_id).to_lowercase();
                let missing = !assets.join(&relative).exists() && seen.insert(relative.clone());
                missing.then(|| {
                    Diagnostic::warning(
                        program.line(index),
                        format!("missing {} (a placeholder will be drawn)", relative),
                    )
                    .with_file(program.file(index))
                })
            }
            _ => None,
        })
        .collect()
}
