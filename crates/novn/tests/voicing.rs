use std::cell::RefCell;
use std::rc::Rc;

use novn::data::settings::Settings;
use novn::game::hooks::Hooks;
use novn::game::reader::Reading;
use novn::screen::{Screen, ScreenState};
use novn::screen_manager::{ScreenFactory, ScreenStateManager};
use novn::screens::keybinds::default_keybinds;
use novn::screens::playing::{PlayingConfig, PlayingScreen};
use novn::screens::settings::{SettingsConfig, SettingsPage, SettingsRow};
use novn::script::{ChoiceOption, Event, StoryVm};
use raylib::prelude::*;

fn option(text: &str, index: usize, enabled: bool) -> ChoiceOption {
    ChoiceOption {
        text: text.into(),
        index,
        enabled,
        reason: None,
        image: None,
        preview: None,
    }
}

#[test]
fn a_line_reads_as_its_speaker_names_it_without_markup() {
    let said = Event::Say {
        speaker: Some("mary".into()),
        text: "[b]Box[/b] fourteen[w=1].".into(),
    };
    let reading = Reading::of(&said, |id| format!("Mary ({id})")).unwrap();
    assert_eq!(
        reading,
        Reading::Line {
            speaker: Some("Mary (mary)".into()),
            text: "Box fourteen.".into(),
        }
    );
    assert_eq!(reading.text(), "Mary (mary): Box fourteen.");

    let narrated = Event::Say {
        speaker: None,
        text: "Rain.".into(),
    };
    assert_eq!(
        Reading::of(&narrated, |_| unreachable!()).unwrap().text(),
        "Rain."
    );
}

#[test]
fn a_choice_reads_only_the_options_a_player_can_take() {
    let choice = Event::Choice {
        options: vec![
            option("Open the box", 0, true),
            option("Burn it", 1, false),
            option("Leave?", 2, true),
        ],
    };
    let reading = Reading::of(&choice, |id| id.to_string()).unwrap();
    assert_eq!(
        reading,
        Reading::Choices {
            options: vec!["Open the box".into(), "Leave?".into()],
        }
    );
    assert_eq!(reading.text(), "Open the box. Leave?");
}

#[test]
fn stage_directions_are_not_read() {
    let hidden = Event::Hide {
        character: "mary".into(),
        transition: None,
    };
    assert_eq!(Reading::of(&hidden, |id| id.to_string()), None);
}

#[test]
fn a_game_without_a_reader_offers_no_self_voicing() {
    let mut hooks = Hooks::default();
    assert!(!hooks.has_reader());
    hooks.read(&Reading::Notice { text: "x".into() });

    let heard = Rc::new(RefCell::new(Vec::new()));
    let log = Rc::clone(&heard);
    hooks.reader(move |reading| log.borrow_mut().push(reading.text()));
    assert!(hooks.has_reader());
    hooks.read(&Reading::Notice {
        text: "Hello".into(),
    });
    assert_eq!(*heard.borrow(), ["Hello"]);
}

#[test]
fn the_setting_is_off_by_default_and_old_files_still_load() {
    assert!(!Settings::default().self_voicing);
    let old: Settings = serde_json::from_str(r#"{"text_speed": 30}"#).unwrap();
    assert!(!old.self_voicing);
}

#[test]
fn the_row_appears_only_when_the_game_gave_a_reader() {
    let config = SettingsConfig::default();
    let rows = config.page_rows(SettingsPage::Accessibility);
    assert!(!rows.contains(&SettingsRow::SelfVoicing));

    let config = SettingsConfig {
        self_voicing_row: true,
        ..Default::default()
    };
    assert_eq!(
        config.page_rows(SettingsPage::Accessibility).last(),
        Some(&SettingsRow::SelfVoicing)
    );
    let mut settings = Settings::default();
    config.step(SettingsRow::SelfVoicing, &mut settings, 1);
    assert!(settings.self_voicing);
    assert_eq!(config.value_name(SettingsRow::SelfVoicing, &settings), "On");
}

#[test]
fn the_controls_list_the_key_only_while_it_does_something() {
    let listed = |playing: &PlayingConfig| {
        default_keybinds(playing, &Default::default(), &Default::default())[0]
            .rows
            .iter()
            .any(|row| row.action == "Self-voicing on / off")
    };
    assert!(listed(&PlayingConfig::default()));
    let playing = PlayingConfig::default().keys(|keys| keys.self_voicing([]));
    assert!(!listed(&playing));
}

struct PlayingFactory;

impl ScreenFactory for PlayingFactory {
    fn create_screen(&self, _: &ScreenState) -> Option<Box<dyn Screen>> {
        Some(Box::new(PlayingScreen::new(Rc::new(
            PlayingConfig::default(),
        ))))
    }
}

#[test]
#[ignore = "opens a window; run with --ignored --test-threads=1 on a machine with a display"]
fn the_line_on_screen_is_read_once_and_again_when_voicing_comes_back() {
    let dir = std::env::temp_dir().join(format!("vn_engine_voicing_{}", std::process::id()));
    let (mut rl, thread) = raylib::init().size(64, 64).title("voicing").build();
    rl.set_trace_log(TraceLogLevel::LOG_WARNING);
    let story = StoryVm::from_source("scene a:\n  mary \"[i]One[/i].\"\n  \"Two.\"\n");
    let mut manager = ScreenStateManager::with_story(
        &mut rl,
        &thread,
        ScreenState::Playing,
        Box::new(PlayingFactory),
        dir,
        story,
    )
    .unwrap();

    let heard = Rc::new(RefCell::new(Vec::<Reading>::new()));
    let mut hooks = Hooks::default();
    let log = Rc::clone(&heard);
    hooks.reader(move |reading| log.borrow_mut().push(reading.clone()));
    manager.hooks = Rc::new(hooks);
    manager.world.settings.values.self_voicing = true;

    for _ in 0..5 {
        manager.update(&mut rl, &thread);
    }
    let line = Reading::Line {
        speaker: Some("mary".into()),
        text: "One.".into(),
    };
    assert_eq!(
        *heard.borrow(),
        std::slice::from_ref(&line),
        "read once, not every frame"
    );

    manager.world.settings.values.self_voicing = false;
    for _ in 0..3 {
        manager.update(&mut rl, &thread);
    }
    assert_eq!(heard.borrow().len(), 1, "silent while off");

    manager.world.settings.values.self_voicing = true;
    manager.update(&mut rl, &thread);
    assert_eq!(
        *heard.borrow(),
        [line.clone(), line],
        "turning it on reads the line again"
    );
}
