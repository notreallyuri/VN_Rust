use std::fmt;
use std::rc::Rc;

use crate::saves::QUICK_SLOT;
use crate::screens::Confirm;
use crate::{GameContext, ScreenState};

type CustomAction = Rc<dyn Fn(&mut GameContext) -> Option<ScreenState>>;

#[derive(Clone)]
pub enum Action {
    NewGame,
    Goto(ScreenState),
    OpenOverlay(String),
    Resume,
    QuickSave,
    QuickLoad,
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
                ctx.story.reset();
                ctx.state.reset();
                ctx.rollback.clear();
                Some(ScreenState::Playing)
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
                        ctx.notify_error(format!("Quick save failed: {}", e.player_message()));
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
                    ctx.notify_error(format!("Quick load failed: {}", e.player_message()));
                    None
                }
            },
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
            Action::Goto(state) => write!(f, "Goto({:?})", state),
            Action::OpenOverlay(name) => write!(f, "OpenOverlay({:?})", name),
            Action::Resume => write!(f, "Resume"),
            Action::QuickSave => write!(f, "QuickSave"),
            Action::QuickLoad => write!(f, "QuickLoad"),
            Action::Quit => write!(f, "Quit"),
            Action::Confirm(confirm) => {
                write!(f, "Confirm({:?}, {:?})", confirm.message, confirm.action)
            }
            Action::Custom(_) => write!(f, "Custom(..)"),
        }
    }
}
