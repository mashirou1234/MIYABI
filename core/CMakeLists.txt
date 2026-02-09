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

# Link against the cxxbridge target, which brings in both the headers and the static library
target_link_libraries(miyabi PRIVATE 
    glfw
    OpenGL::GL
    Freetype::Freetype
)



# Find the compiled Rust library

file(GLOB RUST_LIB "${CMAKE_BINARY_DIR}/core/liblogic.dylib")

message(STATUS "Found Rust library: ${RUST_LIB}")



# Add a custom command to copy the Rust library to the executable's directory



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




