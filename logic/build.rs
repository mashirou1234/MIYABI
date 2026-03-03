use std::env;
use std::path::{Path, PathBuf};

fn find_box2d_include_dir() -> PathBuf {
    if let Ok(path) = env::var("BOX2D_INCLUDE_DIR") {
        let candidate = PathBuf::from(path);
        if candidate.is_dir() {
            return candidate;
        }
    }

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is required"));
    let repo_root = manifest_dir
        .parent()
        .expect("logic crate must be placed under repository root");

    let default_candidate = repo_root.join("build").join("_deps").join("box2d-src").join("include");
    if default_candidate.is_dir() {
        return default_candidate;
    }

    if let Ok(entries) = std::fs::read_dir(repo_root) {
        for entry in entries.flatten() {
            let candidate = entry.path().join("_deps").join("box2d-src").join("include");
            if candidate.is_dir() {
                return candidate;
            }
        }
    }

    panic!(
        "Failed to locate Box2D include directory. Set BOX2D_INCLUDE_DIR or run CMake configure first."
    );
}

fn main() {
    let box2d_include_dir = find_box2d_include_dir();

    let mut build = cxx_build::bridge("src/lib.rs");
    build
        .include("../core/include")
        .include("../core/src")
        .include(Path::new(&box2d_include_dir))
        .flag_if_supported("-std=c++17");

    if std::env::var("CARGO_FEATURE_PERFORMANCE_TEST").is_ok() {
        build.file("src/performance.cpp");
        build.define("MIYABI_PERFORMANCE_TEST", "1");
    }

    build.compile("logic");

    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-env-changed=BOX2D_INCLUDE_DIR");
    println!("cargo:rerun-if-changed=src/performance.cpp");
}
