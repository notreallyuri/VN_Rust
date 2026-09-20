use vn_engine::data::assets::Assets;
use vn_live2d::ModelAssets;

const MANIFEST: &[u8] = br#"{
    "Version": 3,
    "FileReferences": {
        "Moc": "model.moc3", "Textures": ["textures/0.png"],
        "Physics": "physics.json", "Pose": "pose.json",
        "Expressions": [{"Name":"happy","File":"happy.exp3.json"}],
        "Motions": {"Idle":[{"File":"idle.motion3.json","FadeInTime":0.5}]}
    }
}"#;

// Deliberately not real Cubism/image data: these tests exercise asset resolution
// and manifest preflight, not native decoding or rendering.
const FILES: &[(&str, &[u8])] = &[
    ("models/mary/mary.model3.json", MANIFEST),
    ("models/mary/model.moc3", b"model bytes"),
    ("models/mary/textures/0.png", b"texture bytes"),
    ("models/mary/physics.json", b"{}"),
    ("models/mary/pose.json", b"{}"),
    ("models/mary/happy.exp3.json", b"{}"),
    ("models/mary/idle.motion3.json", b"{}"),
];

#[test]
fn embedded_bundle_resolves_every_referenced_asset() {
    let bundle =
        ModelAssets::load(&Assets::Embedded(FILES), "models/mary/mary.model3.json").unwrap();
    assert_eq!(bundle.file_count(), 6);
    assert_eq!(
        bundle.settings().file_references.motions["Idle"][0].fade_in_time,
        Some(0.5)
    );
    assert_eq!(
        bundle.settings().file_references.expressions[0].name,
        "happy"
    );
    assert_eq!(bundle.manifest_bytes(), MANIFEST);
}

#[test]
fn folder_and_embedded_assets_load_the_same_manifest() {
    let dir = std::env::temp_dir().join(format!("vn_live2d_assets_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    for (path, bytes) in FILES {
        let path = dir.join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, bytes).unwrap();
    }
    let bundle =
        ModelAssets::load(&Assets::from(dir.clone()), "models/mary/mary.model3.json").unwrap();
    assert_eq!(bundle.file_count(), 6);
    assert_eq!(bundle.manifest_bytes(), MANIFEST);
    std::fs::remove_dir_all(dir).unwrap();
}

fn failure(manifest: &[u8]) -> String {
    // Temporary folder avoids leaking dynamic fixtures into static embedded data.
    let dir = std::env::temp_dir().join(format!(
        "vn_live2d_invalid_{}_{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("test.model3.json"), manifest).unwrap();
    let result = ModelAssets::load(&Assets::from(dir.clone()), "test.model3.json");
    let message = match result {
        Ok(_) => panic!("expected an error"),
        Err(e) => e.to_string(),
    };
    std::fs::remove_dir_all(dir).unwrap();
    message
}

#[test]
fn missing_assets_report_the_resolved_path() {
    let message = failure(MANIFEST);
    assert!(message.contains("model.moc3"), "{message}");
}

#[test]
fn asset_paths_cannot_escape_the_root_or_be_absolute() {
    for path in ["../model.moc3", "/model.moc3", "C:/model.moc3", ""] {
        let manifest =
            serde_json::json!({"Version":3, "FileReferences":{"Moc":path,"Textures":["0.png"]}});
        let message = failure(&serde_json::to_vec(&manifest).unwrap());
        assert!(
            message.contains("path") || message.contains("escapes"),
            "{message}"
        );
    }
}

#[test]
fn duplicate_expressions_and_invalid_fades_are_rejected_before_loading() {
    let mut manifest: serde_json::Value = serde_json::from_slice(MANIFEST).unwrap();
    let expression = manifest["FileReferences"]["Expressions"][0].clone();
    manifest["FileReferences"]["Expressions"]
        .as_array_mut()
        .unwrap()
        .push(expression);
    assert!(failure(&serde_json::to_vec(&manifest).unwrap()).contains("duplicate"));
    let mut manifest: serde_json::Value = serde_json::from_slice(MANIFEST).unwrap();
    manifest["FileReferences"]["Motions"]["Idle"][0]["FadeInTime"] = (-2).into();
    assert!(failure(&serde_json::to_vec(&manifest).unwrap()).contains("fade"));
}

#[test]
fn incorrect_manifest_version_is_rejected() {
    let mut manifest: serde_json::Value = serde_json::from_slice(MANIFEST).unwrap();
    manifest["Version"] = 2.into();
    assert!(failure(&serde_json::to_vec(&manifest).unwrap()).contains("Version 3"));
}
