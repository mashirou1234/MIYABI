cmake_minimum_required(VERSION 3.15)
project(MIYABI)

# Fetch Corrosion, a tool for integrating Rust into CMake projects.
include(FetchContent)
FetchContent_Declare(
    Corrosion
    GIT_REPOSITORY https://github.com/corrosion-rs/corrosion.git
    GIT_TAG v0.6.1 # Or a newer version
)
FetchContent_MakeAvailable(Corrosion)

add_subdirectory(core)