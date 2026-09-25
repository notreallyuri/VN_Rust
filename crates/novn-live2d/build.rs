use std::{env, fs, path::PathBuf, process::Command};

fn run(command: &mut Command) {
    let status = command
        .status()
        .expect("could not start CMake; install CMake and a C++ compiler");
    assert!(status.success(), "Cubism native build failed: {command:?}");
}

// The SDK ships GLSL 1.20 shaders, which a 3.3 core context cannot compile: a
// failed program leaves the host's shader bound and the model draws as a
// silhouette. Port the embedded copies; the SDK's own files are never touched.
fn core_profile_shader(name: &str, bytes: &[u8]) -> Vec<u8> {
    let source = String::from_utf8(bytes.to_vec())
        .unwrap_or_else(|_| panic!("{name} is not UTF-8; expected Framework 5-r.5 shaders"));
    assert!(
        !source.contains("#version") || source.contains("#version 120"),
        "{name}: expected a GLSL 1.20 shader from Framework 5-r.5"
    );
    let vertex = name.ends_with(".vert");
    let mut ported = source
        .replace("#version 120", "#version 330 core")
        .replace("attribute ", "in ")
        .replace("varying ", if vertex { "out " } else { "in " })
        .replace("texture2D(", "texture(")
        .replace("gl_FragColor", "vn_FragColor");
    // Blend modes arrive as version-less chunks compiled with the shader above them.
    if !vertex && ported.contains("#version 330 core") {
        ported = ported.replacen(
            "#version 330 core",
            "#version 330 core\nout vec4 vn_FragColor;",
            1,
        );
    }
    ported.into_bytes()
}

fn main() {
    println!("cargo:rerun-if-env-changed=CUBISM_SDK_ROOT");
    println!("cargo:rerun-if-changed=native");
    if env::var_os("CARGO_FEATURE_NATIVE").is_none() {
        return;
    }
    assert_eq!(
        env::var("TARGET").unwrap(),
        "x86_64-unknown-linux-gnu",
        "the first novn_live2d native backend supports Linux x86_64 only"
    );
    let sdk = PathBuf::from(env::var_os("CUBISM_SDK_ROOT").expect(
        "native requires Cubism SDK for Native 5-r.5: set CUBISM_SDK_ROOT to its extracted directory (containing Core/ and Framework/). The SDK is not downloaded by this build script.",
    )).canonicalize().expect("CUBISM_SDK_ROOT is not a directory");
    for relative in [
        "Core/include/Live2DCubismCore.h",
        "Core/lib/linux/x86_64/libLive2DCubismCore.a",
        "Framework/CMakeLists.txt",
    ] {
        assert!(
            sdk.join(relative).is_file(),
            "missing SDK file: {}",
            sdk.join(relative).display()
        );
    }
    println!("cargo:rerun-if-changed={}", sdk.display());
    let changelog = fs::read_to_string(sdk.join("Framework/CHANGELOG.md"))
        .expect("Framework/CHANGELOG.md is required to check the SDK version");
    assert!(
        changelog
            .lines()
            .find(|line| line.starts_with("## ["))
            .is_some_and(|line| line.starts_with("## [5-r.5]")),
        "novn_live2d currently targets Framework 5-r.5; other releases need compatibility validation"
    );
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    // The SDK's renderer loads shaders through an application callback. Embed them
    // at build time so the executable never depends on the SDK's working directory.
    let shaders = sdk.join("Framework/src/Rendering/OpenGL/Shaders/Standard");
    let mut paths: Vec<_> = fs::read_dir(&shaders)
        .expect("expected Framework 5-r.5 shaders")
        .map(|entry| entry.unwrap().path())
        .filter(|p| p.is_file())
        .collect();
    paths.sort();
    let mut header = String::from(
        "#pragma once\n#include <cstring>\n#include <cstdlib>\nstatic unsigned char* vn_shader(const std::string path, unsigned int* size) {\n",
    );
    for (i, path) in paths.iter().enumerate() {
        let name = path.file_name().unwrap().to_str().unwrap();
        let bytes = core_profile_shader(name, &fs::read(path).unwrap());
        let data = bytes
            .iter()
            .map(u8::to_string)
            .collect::<Vec<_>>()
            .join(",");
        header.push_str(&format!("static const unsigned char shader_{i}[] = {{{data},0}};\nif (path == \"FrameworkShaders/{name}\") {{ *size = {}; auto* data = static_cast<unsigned char*>(std::malloc(sizeof(shader_{i}))); if (!data) return nullptr; std::memcpy(data, shader_{i}, sizeof(shader_{i})); return data; }}\n", bytes.len()));
    }
    header.push_str("*size = 0; return nullptr; }\n");
    fs::write(out.join("vn_shaders.hpp"), header).unwrap();
    let build = out.join("cmake");
    run(Command::new("cmake")
        .args(["-S", "native", "-B"])
        .arg(&build)
        .arg(format!("-DCUBISM_SDK_ROOT={}", sdk.display()))
        .arg(format!("-DVN_GENERATED={}", out.display()))
        .arg("-DCMAKE_BUILD_TYPE=Release"));
    run(Command::new("cmake")
        .arg("--build")
        .arg(&build)
        .arg("--parallel")
        .arg(env::var("NUM_JOBS").unwrap_or_else(|_| "2".into())));
    println!("cargo:rustc-link-search=native={}", build.display());
    println!(
        "cargo:rustc-link-search=native={}",
        build.join("Framework").display()
    );
    println!(
        "cargo:rustc-link-search=native={}",
        sdk.join("Core/lib/linux/x86_64").display()
    );
    for lib in ["vn_cubism_bridge", "Framework", "Live2DCubismCore"] {
        println!("cargo:rustc-link-lib=static={lib}");
    }
    for lib in ["GLEW", "GL", "stdc++"] {
        println!("cargo:rustc-link-lib={lib}");
    }
}
