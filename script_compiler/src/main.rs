use schemars::schema_for;
use serde::Deserialize;
use serde_json::Value;
use std::{fs, path::Path};

use vn_core::parser::parse_vn_script;
use vn_core::types::Scene;

#[derive(Deserialize)]
struct ProjectConfig {
    game: GameConfig,
}

#[derive(Deserialize)]
struct GameConfig {
    characters: Vec<String>,
    expressions: Vec<String>,
}

fn main() {
    let config_str =
        fs::read_to_string("assets/project.toml").expect("❌ Missing assets/project.toml");
    let config: ProjectConfig =
        toml::from_str(&config_str).expect("❌ Failed to parse project.toml");

    let schema = schema_for!(Scene);
    let mut schema_val = serde_json::to_value(schema).unwrap();

    inject_enums(
        &mut schema_val,
        &config.game.characters,
        &config.game.expressions,
    );

    let schema_json_final = serde_json::to_string_pretty(&schema_val).unwrap();

    fs::create_dir_all("assets/scripts").unwrap();
    fs::write("assets/scripts/schema.json", schema_json_final).expect("Failed to write schema");
    println!("✅ JSON Schema updated and validated against project.toml");

    let scripts_dir = Path::new("assets/scripts");

    if let Ok(entries) = fs::read_dir(scripts_dir) {
        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("vn") {
                let content = fs::read_to_string(&path).expect("Failed to read script");
                let scene = parse_vn_script(&content);

                for (i, line) in scene.lines.iter().enumerate() {
                    if let Some(ref char_name) = line.character {
                        if !config.game.characters.contains(char_name) {
                            panic!(
                                "❌ Error in {:?} line {}: Character '{}' not in project.toml",
                                path,
                                i + 1,
                                char_name
                            );
                        }

                        if let Some(ref expr) = line.expression {
                            if !config.game.expressions.contains(expr) {
                                panic!(
                                    "❌ Error in {:?} line {}: Expression '{}' not in project.toml",
                                    path,
                                    i + 1,
                                    expr
                                );
                            }

                            let asset_path =
                                format!("assets/characters/{}/{}.png", char_name, expr)
                                    .to_lowercase();
                            if !Path::new(&asset_path).exists() {
                                panic!(
                                    "❌ Error in {:?} line {}: Missing asset file '{}'",
                                    path,
                                    i + 1,
                                    asset_path
                                );
                            }
                        }
                    }
                }

                let bin_path = path.with_extension("bin");
                let encoded: Vec<u8> = postcard::to_allocvec(&scene)
                    .expect("❌ Failed to serialize to postcard binary");

                fs::write(bin_path, encoded).expect("❌ Failed to write binary");

                let json_path = path.with_extension("json");
                fs::write(json_path, serde_json::to_string_pretty(&scene).unwrap()).unwrap();
                println!("📖 Compiled and Validated: {:?}", path.file_name().unwrap());
            }
        }
    } else {
        println!("⚠️  Warning: 'assets/scripts' directory not found or inaccessible.");
    }
}

fn inject_enums(schema: &mut Value, characters: &[String], expressions: &[String]) {
    let defs = if schema.get("definitions").is_some() {
        schema.get_mut("definitions")
    } else {
        schema.get_mut("$defs")
    };

    if let Some(properties) = defs
        .and_then(|d| d.get_mut("Dialogue"))
        .and_then(|diag| diag.get_mut("properties"))
    {
        if let Some(char_field) = properties.get_mut("character") {
            *char_field = serde_json::json!({
                "anyOf": [{ "enum": characters }, { "type": "null" }]
            });
        }
        if let Some(expr_field) = properties.get_mut("expression") {
            *expr_field = serde_json::json!({
                "anyOf": [{ "enum": expressions }, { "type": "null" }]
            });
        }
    }
}
