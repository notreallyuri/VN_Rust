use novn::ui::wrap::{is_wide, lines, units};

fn texts(paragraph: &str) -> Vec<String> {
    units(paragraph).into_iter().map(|u| u.text).collect()
}

fn by_chars(paragraph: &str, max: usize) -> Vec<String> {
    lines(paragraph, |candidate| candidate.chars().count() <= max)
}

#[test]
fn latin_breaks_on_spaces() {
    assert_eq!(texts("the lamps are"), ["the", "lamps", "are"]);
    assert_eq!(
        by_chars("the lamps are never put out", 12),
        ["the lamps", "are never", "put out"]
    );
}

#[test]
fn a_script_without_spaces_breaks_between_characters() {
    assert_eq!(texts("此処は書庫"), ["此", "処", "は", "書", "庫"]);
    assert_eq!(by_chars("此処は書庫です", 3), ["此処は", "書庫で", "す"]);
}

#[test]
fn a_line_never_starts_with_closing_punctuation() {
    let wrapped = by_chars("彼は言った。それから", 4);
    assert!(
        wrapped.iter().all(|line| !line.starts_with(['。', '、'])),
        "{:?}",
        wrapped
    );
    assert_eq!(texts("言った。"), ["言っ", "た。"]);
}

#[test]
fn a_line_never_ends_with_an_opening_bracket() {
    assert_eq!(texts("彼「はい"), ["彼", "「は", "い"]);
    let wrapped = by_chars("彼は「はい」と言った", 3);
    assert!(
        wrapped.iter().all(|line| !line.ends_with('「')),
        "{:?}",
        wrapped
    );
}

#[test]
fn small_kana_and_the_long_vowel_stay_with_what_they_follow() {
    assert_eq!(texts("きょうコーヒー"), ["きょ", "う", "コー", "ヒー"]);
}

#[test]
fn mixed_scripts_break_at_both_kinds_of_boundary() {
    assert_eq!(
        texts("これは Archive です"),
        ["こ", "れ", "は", "Archive", "で", "す"]
    );
    let wrapped = by_chars("これは Archive です", 8);
    assert_eq!(wrapped, ["これは", "Archive", "です"]);
}

#[test]
fn a_word_longer_than_the_line_is_left_alone() {
    assert_eq!(
        by_chars("supercalifragilistic", 5),
        ["supercalifragilistic"]
    );
}

#[test]
fn empty_and_blank_paragraphs_keep_a_line() {
    assert_eq!(by_chars("", 10), [""]);
    assert_eq!(by_chars("   ", 10), [""]);
}

#[test]
fn width_classes() {
    assert!(is_wide('書') && is_wide('ひ') && is_wide('ア') && is_wide('한'));
    assert!(!is_wide('a') && !is_wide('É') && !is_wide('·'));
}
