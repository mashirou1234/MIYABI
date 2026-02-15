# Performance Test Guide

This document outlines how to enable and run the performance test suite for the MIYABI engine.

## 1. Purpose

The performance test is designed to benchmark the engine's capabilities under heavy load, such as rendering a large number of sprites. This helps identify performance bottlenecks and measure the impact of optimizations.

It is implemented using a compile-time feature flag to ensure that the performance testing code does not interfere with regular development builds.

## 2. How to Enable the Performance Test

The performance test is enabled via the `performance_test` feature in the `miyabi_logic` crate. To enable it, you need to configure the build with a specific CMake command.

1.  **Clean the build directory (optional but recommended):**
    If you have an existing build, it's best to clean it to ensure a fresh configuration.
    ```bash
    rm -rf build
    ```

2.  **Run CMake with the feature enabled:**
    From the project root directory, run the following commands to configure and build the project with the performance test enabled. The `-D` flag for corrosion is not directly used, but we pass the feature name to `corrosion_add_cxxbridge`. The `logic/CMakeLists.txt` is already configured to use this feature.

    ```bash
    cmake -S . -B build -DCMAKE_BUILD_TYPE=Release
    ```
    *Note: The `logic/CMakeLists.txt` is currently hardcoded to enable the "performance_test" feature. In the future, this could be controlled by a CMake variable.*

## 3. How to Run

1.  **Build the project:**
    ```bash
    cmake --build build
    ```

2.  **Run the executable:**
    ```bash
    ./build/core/miyabi
    ```

## 4. Expected Results

When you run the application with the performance test enabled, you will see the following:

- The application window will open.
- The window's title bar will display the current Frames Per Second (FPS).
- A large number of sprites (e.g., 10,000) will be rendered on the screen, which is used to stress the rendering system.

## 5. Implementation Details

- **CMake & Corrosion:** The `logic/CMakeLists.txt` file uses `corrosion_add_cxxbridge` to build the Rust `miyabi_logic` crate with the `performance_test` feature.
- **Rust (`lib.rs`):** When the `performance_test` feature is enabled, Rust code will call the C++ function `get_performance_test_sprite_count()` to determine how many sprites to render.
- **C++ (`performance.cpp`):** This file contains the `get_performance_test_sprite_count()` function, which defines the number of sprites for the test. This allows easy modification of the load for performance testing.