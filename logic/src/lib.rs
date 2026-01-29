#[cxx::bridge]
mod ffi {
    extern "Rust" {
        fn run_logic();
    }
}

fn run_logic() {
    println!("Hello from Rust!");
}