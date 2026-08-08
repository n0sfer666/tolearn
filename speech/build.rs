use std::path::Path;

const ORDER: &[&str] = &[
    "whisper",
    "parakeet",
    "ggml",
    "ggml-metal",
    "ggml-blas",
    "ggml-cpu",
    "ggml-base",
];

fn main() {
    println!("cargo::rerun-if-changed=csrc/shim.c");
    println!("cargo::rerun-if-changed=vendor/whisper.cpp/CMakeLists.txt");
    if std::env::var_os("CARGO_FEATURE_SPEECH").is_none() {
        return;
    }

    let vendor = Path::new("vendor/whisper.cpp");
    assert!(
        vendor.join("CMakeLists.txt").is_file(),
        "нет vendor/whisper.cpp: подтяните сабмодуль (git submodule update --init --recursive)"
    );

    let built = cmake::Config::new(vendor)
        .define("BUILD_SHARED_LIBS", "OFF")
        .define("WHISPER_BUILD_TESTS", "OFF")
        .define("WHISPER_BUILD_EXAMPLES", "OFF")
        .define("WHISPER_BUILD_SERVER", "OFF")
        .define("GGML_METAL_EMBED_LIBRARY", "ON")
        .define("GGML_OPENMP", "OFF")
        .define("CMAKE_POSITION_INDEPENDENT_CODE", "ON")
        .profile("Release")
        .build();

    for dir in ["lib", "lib64"] {
        let dir = built.join(dir);
        if dir.is_dir() {
            println!("cargo::rustc-link-search=native={}", dir.display());
        }
    }
    for name in linked(&built) {
        println!("cargo::rustc-link-lib=static={name}");
    }
    system();

    cc::Build::new()
        .file("csrc/shim.c")
        .include(built.join("include"))
        .warnings(true)
        .compile("tolearn-speech-shim");
}

fn linked(built: &Path) -> Vec<String> {
    let mut found: Vec<String> = ["lib", "lib64"]
        .iter()
        .filter_map(|dir| std::fs::read_dir(built.join(dir)).ok())
        .flatten()
        .filter_map(Result::ok)
        .filter_map(|entry| stem(&entry.path()))
        .filter(|name| ORDER.contains(&name.as_str()))
        .collect();
    found.sort_by_key(|name| ORDER.iter().position(|known| known == name));
    found.dedup();
    found
}

fn stem(path: &Path) -> Option<String> {
    let name = path.file_name()?.to_str()?;
    let name = name
        .strip_suffix(".a")
        .or_else(|| name.strip_suffix(".lib"))?;
    Some(name.strip_prefix("lib").unwrap_or(name).to_owned())
}

fn system() {
    let target = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    match target.as_str() {
        "macos" | "ios" => {
            for framework in [
                "Foundation",
                "Metal",
                "MetalKit",
                "Accelerate",
                "CoreFoundation",
            ] {
                println!("cargo::rustc-link-lib=framework={framework}");
            }
            println!("cargo::rustc-link-lib=dylib=c++");
        }
        "windows" => {}
        _ => println!("cargo::rustc-link-lib=dylib=stdc++"),
    }
}
