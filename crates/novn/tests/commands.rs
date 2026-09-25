use novn::game::commands::{Command, FromArgs};
use novn::prelude::*;
use novn_script::{CommandSig, ParamKind};

#[command]
fn give_item(
    _context: &mut GameContext,
    _item: String,
    _count: Option<u32>,
) -> Option<ScreenState> {
    None
}

#[command]
fn assemble(_context: &mut GameContext) -> Option<ScreenState> {
    None
}

#[command("note")]
fn write_a_note(_context: &mut GameContext, _key: String) -> Option<ScreenState> {
    None
}

#[command]
fn remember(_context: &mut GameContext, _words: Vec<String>) -> Option<ScreenState> {
    None
}

fn signature<C: Command>() -> CommandSig {
    C::Args::signature()
}

#[test]
fn a_command_is_named_after_its_function() {
    assert_eq!(give_item::NAME, "give_item");
    assert_eq!(assemble::NAME, "assemble");
}

#[test]
fn an_attribute_argument_renames_it() {
    assert_eq!(
        write_a_note::NAME,
        "note",
        "the story says `call note`, whatever the function is called"
    );
}

#[test]
fn flat_arguments_become_the_signature_the_story_is_checked_against() {
    assert_eq!(
        signature::<give_item>(),
        CommandSig {
            required: vec![ParamKind::Word],
            optional: vec![ParamKind::UInt],
            rest: None,
        }
    );
    assert_eq!(signature::<assemble>(), CommandSig::default());
    assert_eq!(
        signature::<remember>(),
        CommandSig {
            rest: Some(ParamKind::Word),
            ..CommandSig::default()
        },
        "a single Vec<String> takes the rest of the line"
    );
}

#[test]
fn the_arguments_of_a_line_are_parsed_in_order() {
    let line = ["letter".to_string(), "3".to_string()];
    assert_eq!(
        <give_item as Command>::Args::from_args(&line),
        Ok(("letter".to_string(), Some(3)))
    );
    assert_eq!(
        <give_item as Command>::Args::from_args(&line[..1]),
        Ok(("letter".to_string(), None)),
        "an Option argument may be left out"
    );
    assert!(
        <give_item as Command>::Args::from_args(&["letter".into(), "many".into()]).is_err(),
        "`many` is not a count"
    );
}

#[test]
fn a_command_is_a_value_the_builder_takes() {
    let app = VnApp::new("Test").command(give_item).command(assemble);
    let commands = app.schema().commands;

    assert!(commands.contains_key("give_item") && commands.contains_key("assemble"));
    assert!(
        !commands.contains_key("write_a_note"),
        "only what was registered"
    );
}

#[derive(StoryWord, Clone, Copy, Debug, PartialEq, Eq)]
enum Ending {
    Report,
    Silence,
    Keeper,
}

#[derive(StoryWord, Clone, Copy, Debug, PartialEq, Eq)]
enum Note {
    DebtPaid,
    #[word("folio_41")]
    Folio41,
    HotWater,
}

#[command]
fn close_case(_context: &mut GameContext, _ending: Ending) -> Option<ScreenState> {
    None
}

#[test]
fn a_story_word_is_the_variant_in_snake_case() {
    assert_eq!(Ending::Keeper.as_str(), "keeper");
    assert_eq!(Note::DebtPaid.as_str(), "debt_paid");
    assert_eq!(Note::HotWater.to_string(), "hot_water");
    assert_eq!(
        Note::Folio41.as_str(),
        "folio_41",
        "`#[word(...)]` names the ones the conversion would get wrong"
    );
    assert_eq!(Ending::ALL.len(), 3);
    assert_eq!(Ending::from_word("silence"), Some(Ending::Silence));
    assert_eq!(Ending::from_word("Silence"), None);
}

#[test]
fn a_command_taking_a_story_word_declares_its_words() {
    assert_eq!(
        signature::<close_case>(),
        CommandSig {
            required: vec![ParamKind::Choice(vec![
                "report".into(),
                "silence".into(),
                "keeper".into()
            ])],
            ..CommandSig::default()
        }
    );
}

#[test]
fn a_word_outside_the_list_is_refused_with_the_list() {
    let sig = signature::<close_case>();
    let error = sig
        .check("close_case", &["reprot".to_string()])
        .unwrap_err();

    assert!(
        error.contains("should be one of report, silence, keeper"),
        "{}",
        error
    );
    assert!(error.contains("did you mean 'report'?"), "{}", error);
    assert!(sig.check("close_case", &["keeper".to_string()]).is_ok());
}

#[test]
fn a_story_word_saves_as_the_word_the_story_writes() {
    let json = serde_json::to_string(&Note::Folio41).unwrap();
    assert_eq!(json, "\"folio_41\"");
    assert_eq!(
        serde_json::from_str::<Note>("\"hot_water\"").unwrap(),
        Note::HotWater
    );

    let gone = serde_json::from_str::<Note>("\"retired\"").unwrap_err();
    assert!(
        gone.to_string().contains("unknown Note `retired`"),
        "a save from a version that had other words says so: {}",
        gone
    );
}
