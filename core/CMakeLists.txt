cmake_minimum_required(VERSION 3.15)
project(miyabi_core)

# Fetch GLM for math
include(FetchContent)
set(GLM_TEST_ENABLE OFF CACHE BOOL "Disable GLM tests")
FetchContent_Declare(
    glm
    GIT_REPOSITORY https://github.com/g-truc/glm.git
    GIT_TAG 1.0.0
)
FetchContent_MakeAvailable(glm)
FetchContent_GetProperties(glm
    SOURCE_DIR glm_SOURCE_DIR
)
include_directories(SYSTEM ${glm_SOURCE_DIR})

# Find necessary packages
find_package(glfw3 3.3 REQUIRED)
find_package(OpenGL REQUIRED)
find_package(Freetype REQUIRED)

# Import the logic crate to generate the C++ headers from its cxx bridge.
corrosion_import_crate(MANIFEST_PATH ../logic/Cargo.toml)
corrosion_add_cxxbridge(miyabi_cxxbridge
    CRATE miyabi_logic
    FILES lib.rs
)

# Import the sample_game crate, which is the dynamic library we'll actually link against.
corrosion_import_crate(MANIFEST_PATH ../sample_game/Cargo.toml)

add_executable(miyabi
    src/main.cpp
    src/glad.c
    src/renderer/ShaderManager.cpp
    src/renderer/MeshManager.cpp
    src/renderer/MaterialManager.cpp
    src/renderer/TextureManager.cpp
    src/renderer/FontManager.cpp
    src/renderer/TextRenderer.cpp
)

target_include_directories(miyabi PUBLIC 
    include
    src # For renderer headers
    $<TARGET_PROPERTY:miyabi_cxxbridge,INTERFACE_INCLUDE_DIRECTORIES>
    ${FREETYPE_INCLUDE_DIRS}
    ${glm_SOURCE_DIR}
)

# Link against the sample_game target (from corrosion_import_crate)
# and other system libraries.
target_link_libraries(miyabi PRIVATE 
    sample_game # This is the target created by Corrosion for the sample_game crate
    glfw
    OpenGL::GL
    Freetype::Freetype
)

# Find the compiled Rust library to copy it next to the executable
file(GLOB RUST_LIB "${CMAKE_BINARY_DIR}/core/libsample_game.dylib")

message(STATUS "Found Rust library to copy: ${RUST_LIB}")

if(RUST_LIB)
    add_custom_command(TARGET miyabi POST_BUILD
        COMMAND ${CMAKE_COMMAND} -E copy
        ${RUST_LIB}
        $<TARGET_FILE_DIR:miyabi>
        COMMENT "Copying Rust library to executable directory"
    )
endif()

# Set the rpath for the executable
set_target_properties(miyabi PROPERTIES
    BUILD_WITH_INSTALL_RPATH TRUE
    INSTALL_RPATH "@executable_path"
)
