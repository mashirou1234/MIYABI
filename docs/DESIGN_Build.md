# MIYABI Build System & Hot-Reloading Technical Design

> Status Note (2026-02-23): このドキュメントはホットリロード設計の履歴を含みます。現行実装は `staticlib` ベースで、`core/src/main.cpp` から `get_miyabi_vtable()` を静的リンクして利用します。運用上の正準手順は `docs/CODEX_MIGRATION_STATUS.md` を参照してください。

## 1. Document Purpose and Overview

This document provides an exhaustive specification for the MIYABI build system and the technical mechanics of its hot-reloading workflow. A stable, understandable, and reproducible build process is critical for a hybrid C++/Rust project.

The primary goals of this system are:
1.  **Seamless Integration:** To have `CMake` act as the single, top-level build system that seamlessly invokes `Cargo` under the hood. The developer should not need to run `cargo` manually.
2.  **Automated Code Generation:** To ensure the `cxx` bridge's C++ header files are automatically generated and made available to the C++ compiler without manual path configuration.
3.  **Robust Hot-Reloading:** To define the precise build commands and mechanisms that enable the live recompilation and reloading of the Rust `logic` crate while the main C++ application is running.

## 2. Toolchain and Dependencies

The following tools are required to build and run the project. This section serves as a checklist for setting up a new development environment.

-   **`CMake`:** Minimum version 3.15.
-   **`Rust Toolchain`:** Rust 2021 Edition or newer. This can be installed via `rustup`.
-   **`C++ Compiler`:** A compiler supporting C++14 or newer (e.g., Clang, GCC, MSVC).
-   **`fswatch` (Development Only):** A cross-platform file change monitor. This is required for the live hot-reloading feature during development. It is **not** a dependency for a final production build.

## 3. CMake as the Top-Level Build System

`CMake` is the single entry point for all build operations.

### 3.1. Rationale

Using a single, unified build command (`cmake --build .`) simplifies the entire development process. `CMake` will orchestrate the compilation of both the C++ host and the Rust library, managing dependencies between them correctly.

### 3.2. `Corrosion`: The CMake-Cargo Bridge

We use the `Corrosion` CMake toolkit to achieve the C++/Rust integration. It is the cornerstone of this system.

-   **Role:** Corrosion provides CMake functions that can parse a `Cargo.toml` file and create proper CMake build targets from it. It translates `target_link_libraries` calls in CMake into a dependency graph that forces `cargo build` to run at the correct time.

### 3.3. Root `CMakeLists.txt` Structure

The main `CMakeLists.txt` file orchestrates the build process.

```cmake
cmake_minimum_required(VERSION 3.15)
project(MIYABI LANGUAGES CXX C)

# --- Find C++ Dependencies ---
# (e.g., OpenGL, GLFW, GLAD)
# ...

# --- Bridge Rust with Corrosion ---
# Fetches Corrosion, either from a submodule or via FetchContent.
include(build/corrosion.cmake)

# This is the core command. It parses logic/Cargo.toml and creates a
# CMake target named 'logic'.
corrosion_import_crate(MANIFEST_PATH logic/Cargo.toml)

# --- Define C++ Executable ---
add_executable(miyabi
    core/src/main.cpp
    core/src/glad.c
)

# --- Link Libraries ---
# This crucial line does two things:
# 1. It links the C++ 'miyabi' executable against the Rust 'logic' library (liblogic.dylib).
# 2. It tells CMake that 'miyabi' DEPENDS ON 'logic'. CMake will now ensure
#    that the 'logic' target (i.e., `cargo build`) is successfully built
#    BEFORE it attempts to compile `main.cpp`.
target_link_libraries(miyabi PRIVATE logic)

# --- Header Management for CXX ---
# The cxx bridge generates headers inside the CMake build directory.
# We must add this directory to the include path for the C++ executable.
# ${logic_BINARY_DIR} is a variable provided by Corrosion.
target_include_directories(miyabi PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}/core/include
    ${logic_BINARY_DIR}/cxxbridge
)
```

## 4. Cargo and `build.rs` Configuration

The `logic` crate is configured to produce a C-compatible dynamic library.

### 4.1. `logic/Cargo.toml`

```toml
[package]
name = "logic"
version = "0.1.0"
edition = "2021"

[lib]
# This is ESSENTIAL. It tells cargo to produce a C-compatible dynamic
# library (.dylib on macOS, .so on Linux, .dll on Windows).
crate-type = ["cdylib"]

[dependencies]
cxx = "1.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

[build-dependencies]
# cxx-build is required to run the CXX code generator.
cxx-build = "1.0"
```

### 4.2. `logic/build.rs`: The Code Generation Script

This script is executed by Cargo before compiling the `logic` crate. Its job is to invoke the `cxx` code generator.

```rust
// logic/build.rs
fn main() {
    // 1. Tell cxx_build to parse our bridge definition in `src/lib.rs`.
    let bridge = cxx_build::bridge("src/lib.rs");

    // 2. Perform the code generation. This creates the C++ header files
    //    and a C++ object file that contains the necessary trampoline functions.
    bridge
        // We can add files here that need to be compiled alongside the bridge
        .file("src/miyabi_bridge.cpp") // (Currently empty, but still required by cxx)
        // Set the C++ standard for the generated code.
        .flag_if_supported("-std=c++14")
        // Compile the generated code into a static library (`libmiyabi_logic.a`)
        // that will be linked into the final `liblogic.dylib`.
        .compile("miyabi_logic");

    // 3. Tell Cargo to re-run this build script only if these files change.
    //    This prevents unnecessary rebuilds.
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/miyabi_bridge.cpp");
    println!("cargo:rerun-if-changed=include/miyabi.h");
}
```

## 5. Precise Hot-Reloading Build Workflow

This section details the sequence of events triggered by the `system("cmake --build build")` call inside `main.cpp`.

1.  **Invocation:** The C++ application executes `cmake --build build` as a blocking subprocess.
2.  **CMake Evaluation:** CMake starts. It already has its dependency graph from the initial configuration. It knows the `miyabi` target depends on the `logic` target.
3.  **Target `logic`:** CMake checks if the `logic` target is up-to-date.
    -   Corrosion has configured this target to execute `cargo build --manifest-path .../logic/Cargo.toml`.
    -   Cargo starts its own evaluation. It sees that a `.rs` file in `logic/src` has changed (its timestamp is newer than the last build product, `liblogic.dylib`).
    -   Cargo performs a fast, **incremental compilation** of only the changed Rust code.
    -   Cargo runs the `build.rs` script, which regenerates the `cxx` bridge code (this is also very fast).
    -   Cargo links the final `liblogic.dylib` and places it in the build output directory (`build/logic/...`).
4.  **Target `miyabi`:** CMake now checks if the `miyabi` target is up-to-date.
    -   It sees that its dependency, `logic`, was just rebuilt.
    -   However, it also sees that no C++ source files (`main.cpp`, etc.) have changed.
    -   Therefore, CMake determines that there is **no need to recompile or relink the C++ executable**.
5.  **Completion:** The `cmake --build` command finishes, having only rebuilt the Rust dylib. Control returns to the C++ application.
6.  **Load New Library:** The application can now safely `dlopen` the new `liblogic.dylib`, as it is guaranteed to be present and up-to-date.

## 6. Onboarding Workflow for New Developers

This entire system results in a very simple workflow for a new developer:

1.  **Clone:** `git clone <repository_url>`
2.  **Configure:** `cd <repository_root> && mkdir build && cd build && cmake ..`
3.  **Build:** `cmake --build .` (or `make`)
4.  **Run:** `./miyabi`
5.  **Develop:** Edit any `.rs` file in the `logic/src` directory. The running `miyabi` application will automatically detect the change, rebuild the library in the background, and hot-reload the logic. No manual intervention is required.
