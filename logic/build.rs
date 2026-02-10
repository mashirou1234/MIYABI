fn main() {
    cxx_build::bridge("src/lib.rs")
        .include("../core/include")
        .flag_if_supported("-std=c++17")
        .compile("logic");

    println!("cargo:rerun-if-changed=src/lib.rs");
}
