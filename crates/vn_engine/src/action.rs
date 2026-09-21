use std::fmt;
use std::rc::Rc;

use crate::context::GameContext;
use crate::data::saves::QUICK_SLOT;
use crate::screen::ScreenState;
use crate::screens::confirm::Confirm;

type CustomAction = Rc<dyn Fn(&mut GameContext) -> Option<ScreenState>>;

#[derive(Clone)]
pub enum Action {
    NewGame,
    Continue,
    Goto(ScreenState),
    OpenOverlay(String),
    Resume,
    QuickSave,
    QuickLoad,
    ToggleAuto,
    ToggleSkip,
    Quit,
    Confirm(Box<Confirm>),
    Custom(CustomAction),
}

impl Action {
    pub fn overlay(name: impl Into<String>) -> Self {
        Action::OpenOverlay(name.into())
    }

    pub fn confirm(message: impl Into<String>, action: Action) -> Self {
        Confirm::new(message, action).into()
    }

    pub fn custom(action: impl Fn(&mut GameContext) -> Option<ScreenState> + 'static) -> Self {
        Action::Custom(Rc::new(action))
    }

    pub fn run(&self, ctx: &mut GameContext) -> Option<ScreenState> {
        match self {
            Action::NewGame => {
                ctx.override_cursor(None);
                #[cfg(feature = "character-visuals")]
                ctx.resources.visuals.reset();
                ctx.log.clear();
                *ctx.modes = crate::data::session::PlayModes::default();
                ctx.story.reset();
                ctx.state.reset();
                ctx.rollback.clear();
                Some(ScreenState::Playing)
            }
            Action::Continue => {
                if ctx.story.current().is_some() {
                    return Some(ScreenState::Playing);
                }
                let Some((slot, _)) = ctx.saves.latest() else {
                    ctx.notify("No saved game yet");
                    return None;
                };
                match ctx.load(&slot) {
                    Ok(_) => Some(ScreenState::Playing),
                    Err(e) => {
                        eprintln!("⚠️ Continue failed ({}): {}", slot, e);
                        let message = ctx.message(
                            "Could not continue: {reason}",
                            &[("reason", &e.player_message())],
                        );
                        ctx.notify_error(message);
                        None
                    }
                }
            }
            Action::Goto(state) => Some(state.clone()),
            Action::OpenOverlay(name) => {
                ctx.open_overlay(name);
                None
            }
            Action::Resume => {
                ctx.close_overlays();
                None
            }
            Action::QuickSave => {
                match ctx.save(QUICK_SLOT) {
                    Ok(()) => ctx.notify("Quick saved"),
                    Err(e) => {
                        eprintln!("⚠️ Quick save failed: {}", e);
                        let message = ctx.message(
                            "Quick save failed: {reason}",
                            &[("reason", &e.player_message())],
                        );
                        ctx.notify_error(message);
                    }
                }
                None
            }
            Action::QuickLoad => match ctx.load(QUICK_SLOT) {
                Ok(_) => {
                    ctx.notify("Quick loaded");
                    Some(ScreenState::Playing)
                }
                Err(e) => {
                    eprintln!("⚠️ Quick load failed: {}", e);
                    let message = ctx.message(
                        "Quick load failed: {reason}",
                        &[("reason", &e.player_message())],
                    );
                    ctx.notify_error(message);
                    None
                }
            },
            Action::ToggleAuto => {
                ctx.modes.auto = !ctx.modes.auto;
                None
            }
            Action::ToggleSkip => {
                ctx.modes.skip = !ctx.modes.skip;
                None
            }
            Action::Quit => Some(ScreenState::Quit),
            Action::Confirm(confirm) => {
                ctx.confirm((**confirm).clone());
                None
            }
            Action::Custom(action) => action(ctx),
        }
    }
}

impl fmt::Debug for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::NewGame => write!(f, "NewGame"),
            Action::Continue => write!(f, "Continue"),
            Action::Goto(state) => write!(f, "Goto({:?})", state),
            Action::OpenOverlay(name) => write!(f, "OpenOverlay({:?})", name),
            Action::Resume => write!(f, "Resume"),
            Action::QuickSave => write!(f, "QuickSave"),
            Action::QuickLoad => write!(f, "QuickLoad"),
            Action::ToggleAuto => write!(f, "ToggleAuto"),
            Action::ToggleSkip => write!(f, "ToggleSkip"),
            Action::Quit => write!(f, "Quit"),
            Action::Confirm(confirm) => {
                write!(f, "Confirm({:?}, {:?})", confirm.message, confirm.action)
            }
            Action::Custom(_) => write!(f, "Custom(..)"),
        }
    }
}
