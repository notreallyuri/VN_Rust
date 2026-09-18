use serde::{Deserialize, Serialize};
use vn_engine::script::{Event, StoryVm};
use vn_engine::{GameState, Rollback, RollbackConfig};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
struct Coins(u32);

struct Game {
    story: StoryVm,
    state: GameState,
    rollback: Rollback,
}

impl Game {
    fn new(source: &str, config: RollbackConfig) -> Self {
        let mut state = GameState::default();
        state.insert(Coins(0));
        let mut game = Self {
            story: StoryVm::from_source(source),
            state,
            rollback: Rollback::new(config),
        };
        game.next();
        game
    }

    fn next(&mut self) -> Event {
        loop {
            match self.story.advance() {
                Event::Commit => self.rollback.mark_barrier(),
                Event::Call { command, .. } => {
                    if self.rollback.blocks_command(&command) {
                        self.rollback.mark_barrier();
                    }
                    if command == "earn" {
                        self.state.get_mut::<Coins>().0 += 1;
                    }
                }
                event if event.is_blocking() => {
                    self.rollback.record(&self.story, &self.state);
                    return event;
                }
                _ => {}
            }
        }
    }

    fn choose(&mut self, index: usize) -> Event {
        self.story.choose(index).unwrap();
        if !self.rollback.config().through_choices {
            self.rollback.mark_barrier();
        }
        self.next()
    }

    fn back(&mut self) -> bool {
        self.rollback.back(&mut self.story, &mut self.state)
    }

    fn forward(&mut self) -> bool {
        self.rollback.forward(&mut self.story, &mut self.state)
    }

    fn line(&self) -> String {
        match self.story.current() {
            Some(Event::Say { text, .. }) => text.clone(),
            Some(Event::Choice { .. }) => "<choice>".into(),
            Some(Event::End) => "<end>".into(),
            other => format!("{:?}", other),
        }
    }
}

const LINEAR: &str = "scene a:\n  \"one\"\n  call earn\n  \"two\"\n  call earn\n  \"three\"\n";

const BRANCHING: &str = r#"
scene a:
  "before"
  choice:
    "Left":
      "went left"
    "Right":
      "went right"
  "after"
"#;

#[test]
fn back_and_forward_restore_story_and_state() {
    let mut game = Game::new(LINEAR, RollbackConfig::default());
    game.next();
    game.next();
    assert_eq!(game.line(), "three");
    assert_eq!(game.state.get::<Coins>().0, 2);

    assert!(game.back());
    assert_eq!(game.line(), "two");
    assert_eq!(game.state.get::<Coins>().0, 1);

    assert!(game.back());
    assert_eq!(game.line(), "one");
    assert_eq!(game.state.get::<Coins>().0, 0);
    assert!(!game.back());

    assert!(game.forward());
    assert!(game.forward());
    assert_eq!(game.line(), "three");
    assert_eq!(game.state.get::<Coins>().0, 2);
    assert!(!game.forward());
}

#[test]
fn advancing_after_rolling_back_replays_and_drops_the_future() {
    let mut game = Game::new(LINEAR, RollbackConfig::default());
    game.next();
    game.back();

    assert!(game.rollback.can_go_forward());
    assert_eq!(
        game.next(),
        Event::Say {
            speaker: None,
            text: "two".into()
        }
    );
    assert!(!game.rollback.can_go_forward());
    assert_eq!(game.state.get::<Coins>().0, 1);
}

#[test]
fn choices_can_be_undone_by_default() {
    let mut game = Game::new(BRANCHING, RollbackConfig::default());
    game.next();
    game.choose(0);
    assert_eq!(game.line(), "went left");

    assert!(game.back());
    assert_eq!(game.line(), "<choice>");
    assert_eq!(game.choose(1), narration("went right"));
}

#[test]
fn choices_can_be_made_permanent_globally() {
    let mut game = Game::new(BRANCHING, RollbackConfig::default().through_choices(false));
    game.next();
    game.choose(0);
    game.next();
    assert_eq!(game.line(), "after");

    assert!(game.back());
    assert_eq!(game.line(), "went left");
    assert!(!game.back());
}

#[test]
fn choice_final_is_permanent_whatever_the_default() {
    let source = BRANCHING.replace("  choice:", "  choice final:");
    let mut game = Game::new(&source, RollbackConfig::default());
    game.next();
    game.choose(1);

    assert_eq!(game.line(), "went right");
    assert!(!game.back());
}

#[test]
fn commit_inside_one_option_only_locks_that_option() {
    let source = r#"
scene a:
  "before"
  choice:
    "Spare him":
      "he runs"
    "Pull the trigger":
      commit
      "it is done"
  "after"
"#;

    let mut spare = Game::new(source, RollbackConfig::default());
    spare.next();
    spare.choose(0);
    assert!(spare.back());
    assert_eq!(spare.line(), "<choice>");

    let mut shoot = Game::new(source, RollbackConfig::default());
    shoot.next();
    shoot.choose(1);
    assert_eq!(shoot.line(), "it is done");
    assert!(!shoot.back());

    shoot.next();
    assert!(shoot.back());
    assert_eq!(shoot.line(), "it is done");
    assert!(!shoot.back());
}

#[test]
fn blocked_commands_are_barriers() {
    let config = RollbackConfig::default().block_command("earn");
    let mut game = Game::new(LINEAR, config);
    game.next();

    assert_eq!(game.line(), "two");
    assert!(!game.back());
    assert_eq!(game.state.get::<Coins>().0, 1);
}

#[test]
fn history_is_bounded() {
    let source = (0..20)
        .map(|i| format!("  \"line {}\"\n", i))
        .collect::<String>();
    let mut game = Game::new(
        &format!("scene a:\n{}", source),
        RollbackConfig::default().max_steps(5),
    );
    for _ in 0..19 {
        game.next();
    }

    assert_eq!(game.rollback.steps_back(), 5);
    for _ in 0..5 {
        assert!(game.back());
    }
    assert!(!game.back());
    assert_eq!(game.line(), "line 14");
}

#[test]
fn disabled_rollback_records_nothing() {
    let mut game = Game::new(LINEAR, RollbackConfig::default().enabled(false));
    game.next();
    assert!(!game.back());
    assert_eq!(game.rollback.steps_back(), 0);
}

#[test]
fn recording_the_same_line_twice_is_ignored() {
    let mut game = Game::new(LINEAR, RollbackConfig::default());
    game.rollback.record(&game.story, &game.state);
    game.rollback.record(&game.story, &game.state);
    assert_eq!(game.rollback.steps_back(), 0);
}

#[test]
fn clear_forgets_everything() {
    let mut game = Game::new(LINEAR, RollbackConfig::default());
    game.next();
    game.rollback.clear();
    assert!(!game.back());
}

fn narration(text: &str) -> Event {
    Event::Say {
        speaker: None,
        text: text.into(),
    }
}
