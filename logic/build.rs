// Include the generated paths module
include!("src/paths.rs");

fn main() {
    cxx_build::bridge("src/lib.rs")
        .include("../core/include")
        .include("../core/src")
        .include(BOX2D_INCLUDE_DIR)
        .flag_if_supported("-std=c++17")
        .compile("logic");

    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/paths.rs");
}
