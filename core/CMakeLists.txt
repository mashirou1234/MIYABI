cmake_minimum_required(VERSION 3.15)
project(miyabi_core)

# Find necessary packages
find_package(glfw3 3.3 REQUIRED)
find_package(OpenGL REQUIRED)

# Import the Rust library so Corrosion is aware of it
corrosion_import_crate(MANIFEST_PATH ../logic/Cargo.toml)

# Create a specific target for the cxx bridge that holds include paths and dependencies
corrosion_add_cxxbridge(miyabi_cxxbridge
    CRATE logic
    FILES lib.rs
)

add_executable(miyabi
    src/main.cpp
    src/glad.c
)

target_include_directories(miyabi PUBLIC 
    include
)

# Link against the cxxbridge target, which brings in both the headers and the static library
target_link_libraries(miyabi PRIVATE 
    miyabi_cxxbridge
    glfw
    OpenGL::GL
)