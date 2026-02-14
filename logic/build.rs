// Include the generated paths module
include!("src/paths.rs");

fn main() {
    let mut build = cxx_build::bridge("src/lib.rs");
    build
        .include("../core/include")
        .include("../core/src")
        .include(BOX2D_INCLUDE_DIR)
        .flag_if_supported("-std=c++17");

    if std::env::var("CARGO_FEATURE_PERFORMANCE_TEST").is_ok() {
        build.file("src/performance.cpp");
        build.define("MIYABI_PERFORMANCE_TEST", "1");
    }
    
    build.compile("logic");

    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/paths.rs");
    println!("cargo:rerun-if-changed=src/performance.cpp");
}
