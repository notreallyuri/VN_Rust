use novn::data::rollback::{Rollback, RollbackConfig, VisualParameters};
use novn::data::state::GameState;
use novn::script::{Event, StoryVm};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
struct Coins(u32);

struct Game {
    story: StoryVm,
    state: GameState,
    rollback: Rollback,
    pushed: VisualParameters,
}

impl Game {
    fn new(source: &str, config: RollbackConfig) -> Self {
        let mut state = GameState::default();
        state.insert(Coins(0));
        let mut game = Self {
            story: StoryVm::from_source(source),
            state,
            rollback: Rollback::new(config),
            pushed: VisualParameters::new(),
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
                    self.rollback.record_with_log(
                        &self.story,
                        &self.state,
                        None,
                        self.pushed.clone(),
                    );
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

    fn push(&mut self, character: &str, id: &str, value: f32) {
        self.pushed
            .entry(character.to_string())
            .or_default()
            .insert(id.to_string(), value);
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

fn reload(game: &Game, source: &str, config: RollbackConfig) -> Game {
    let mut story = StoryVm::from_source(source);
    story.restore(&game.story.snapshot()).unwrap();
    let mut state = GameState::default();
    state.insert(Coins(0));
    let pending = state.prepare_load(&game.state.to_json().unwrap()).unwrap();
    state.apply(pending);

    let mut rollback = Rollback::new(config);
    let json = serde_json::to_string(&game.rollback.history()).unwrap();
    rollback.restore_history(serde_json::from_str(&json).unwrap(), &story);
    Game {
        story,
        state,
        rollback,
        pushed: game.pushed.clone(),
    }
}

#[test]
fn history_survives_a_save_and_load() {
    let mut game = Game::new(LINEAR, RollbackConfig::default());
    game.next();
    game.next();

    let mut loaded = reload(&game, LINEAR, RollbackConfig::default());
    assert_eq!(loaded.line(), "three");
    assert!(loaded.back());
    assert_eq!(loaded.line(), "two");
    assert_eq!(loaded.state.get::<Coins>().0, 1);
    assert!(loaded.back());
    assert_eq!(loaded.line(), "one");
    assert!(!loaded.back());
}

#[test]
fn barriers_survive_a_save_and_load() {
    let source = "scene a:\n  \"one\"\n  commit\n  \"two\"\n  \"three\"\n";
    let mut game = Game::new(source, RollbackConfig::default());
    game.next();
    game.next();

    let mut loaded = reload(&game, source, RollbackConfig::default());
    assert!(loaded.back());
    assert_eq!(loaded.line(), "two");
    assert!(!loaded.back());
}

#[test]
fn history_stops_at_an_edited_scene() {
    let source = "scene a:\n  \"a1\"\n  \"a2\"\n  jump b\nscene b:\n  \"b1\"\n  \"b2\"\n";
    let edited = "scene a:\n  \"a1 (edited)\"\n  \"a2\"\n  jump b\nscene b:\n  \"b1\"\n  \"b2\"\n";
    let mut game = Game::new(source, RollbackConfig::default());
    game.next();
    game.next();
    game.next();
    assert_eq!(game.line(), "b2");

    let mut loaded = reload(&game, edited, RollbackConfig::default());
    assert_eq!(loaded.rollback.steps_back(), 1);
    assert!(loaded.back());
    assert_eq!(loaded.line(), "b1");
    assert!(!loaded.back());
}

#[test]
fn history_is_trimmed_to_the_configured_length() {
    let mut game = Game::new(LINEAR, RollbackConfig::default());
    game.next();
    game.next();

    let loaded = reload(&game, LINEAR, RollbackConfig::default().max_steps(1));
    assert_eq!(loaded.rollback.steps_back(), 1);
}

#[test]
fn history_can_be_left_out_of_saves() {
    let mut game = Game::new(LINEAR, RollbackConfig::default().save_history(false));
    game.next();
    assert!(game.rollback.history().is_empty());
    assert!(game.rollback.can_go_back(), "rollback itself still works");

    let disabled = Game::new(LINEAR, RollbackConfig::default().enabled(false));
    assert!(disabled.rollback.history().is_empty());
}

#[test]
fn a_step_remembers_what_the_game_pushed_onto_its_characters() {
    let mut game = Game::new(LINEAR, RollbackConfig::default());
    assert!(game.rollback.visuals().is_empty(), "nothing pushed yet");

    game.push("mary", "blush", 1.0);
    game.next();
    assert_eq!(game.rollback.visuals()["mary"]["blush"], 1.0);

    game.push("mary", "blush", 0.25);
    game.next();
    assert_eq!(game.rollback.visuals()["mary"]["blush"], 0.25);

    assert!(game.back());
    assert_eq!(
        game.rollback.visuals()["mary"]["blush"],
        1.0,
        "a step back is the value that line was shown with"
    );
    assert!(game.back());
    assert!(
        game.rollback.visuals().is_empty(),
        "and before the push there was nothing"
    );

    assert!(game.forward());
    assert_eq!(game.rollback.visuals()["mary"]["blush"], 1.0);
}

#[test]
fn a_line_repeated_with_a_different_push_is_its_own_step() {
    let mut game = Game::new(LINEAR, RollbackConfig::default());
    let steps = |game: &Game| game.rollback.steps_back();

    let before = steps(&game);
    game.rollback
        .record_with_log(&game.story, &game.state, None, VisualParameters::new());
    assert_eq!(
        steps(&game),
        before,
        "the same line and the same push is one step"
    );

    game.push("mary", "blush", 1.0);
    let pushed = game.pushed.clone();
    game.rollback
        .record_with_log(&game.story, &game.state, None, pushed);
    assert_eq!(
        steps(&game),
        before + 1,
        "the same line with a new push is a step of its own, so it can be undone"
    );
    assert!(game.back());
    assert!(game.rollback.visuals().is_empty());
}
