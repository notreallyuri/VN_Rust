use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use vn_engine::data::persistent::{PERSISTENT_FILE_NAME, Persistent};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
struct Achievements {
    unlocked: BTreeSet<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
struct Endings {
    reached: u32,
}

fn file(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "vn_engine_persistent_{}_{}",
        name,
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    dir.join(PERSISTENT_FILE_NAME)
}

fn open(path: &PathBuf) -> Persistent {
    let mut store = Persistent::in_memory();
    store.insert(Achievements::default());
    store.insert(Endings::default());
    store.load(path);
    store
}

#[test]
fn values_outlive_the_session_that_wrote_them() {
    let path = file("survive");
    let mut store = open(&path);
    store
        .get_mut::<Achievements>()
        .unlocked
        .insert("ending_report".into());
    store.get_mut::<Endings>().reached = 2;
    store.save();

    let store = open(&path);
    assert!(
        store
            .get::<Achievements>()
            .unlocked
            .contains("ending_report")
    );
    assert_eq!(store.get::<Endings>().reached, 2);
    assert!(!path.with_extension("json.tmp").exists());
}

#[test]
fn nothing_is_written_until_a_value_is_borrowed_mutably() {
    let path = file("lazy");
    let mut store = open(&path);
    let _ = store.get::<Endings>();
    store.save();
    assert!(!path.exists());
}

#[test]
fn values_from_types_the_game_no_longer_registers_are_kept() {
    let path = file("unknown");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        r#"{"version":1,"values":{"Endings":{"reached":1},"Retired":{"kept":true}}}"#,
    )
    .unwrap();

    let mut store = open(&path);
    store.get_mut::<Endings>().reached = 3;
    store.save();

    let json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(json["values"]["Retired"]["kept"], true);
    assert_eq!(json["values"]["Endings"]["reached"], 3);
}

#[test]
fn a_value_that_no_longer_fits_its_type_falls_back_alone_and_is_backed_up() {
    let path = file("reshaped");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let old = r#"{"version":1,"values":{"Endings":{"reached":"many"},"Achievements":{"unlocked":["thorough_reader"]}}}"#;
    fs::write(&path, old).unwrap();

    let store = open(&path);
    assert_eq!(store.get::<Endings>(), &Endings::default());
    assert!(
        store
            .get::<Achievements>()
            .unlocked
            .contains("thorough_reader")
    );
    assert_eq!(
        fs::read_to_string(path.with_extension("json.bak")).unwrap(),
        old
    );
}

#[test]
fn a_file_from_a_newer_version_is_read_but_never_overwritten() {
    let path = file("newer");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let newer = r#"{"version":99,"values":{"Endings":{"reached":7}}}"#;
    fs::write(&path, newer).unwrap();

    let mut store = open(&path);
    assert_eq!(store.get::<Endings>().reached, 7);
    store.get_mut::<Endings>().reached = 8;
    store.save();
    assert_eq!(fs::read_to_string(&path).unwrap(), newer);
}

#[test]
fn a_damaged_file_is_backed_up_and_replaced_on_the_next_write() {
    let path = file("damaged");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, "{ not json").unwrap();

    let mut store = open(&path);
    assert_eq!(store.get::<Endings>(), &Endings::default());
    assert_eq!(
        fs::read_to_string(path.with_extension("json.bak")).unwrap(),
        "{ not json"
    );
    store.get_mut::<Endings>().reached = 1;
    store.save();
    assert_eq!(open(&path).get::<Endings>().reached, 1);
}
