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
    ${FREETYPE_INCLUDE_DIRS}
    ${glm_SOURCE_DIR}
)

add_library(miyabi_runtime STATIC
    src/miyabi_bridge.cpp
    src/physics/PhysicsManager.cpp
    ../logic/src/performance.cpp
)

target_include_directories(miyabi_runtime PUBLIC
    include
    src
)

# HACK: Explicitly propagate include directories from the CXX bridge.
# For some reason, this is not happening automatically via target_link_libraries.
if(TARGET miyabi_logic_cxx)
    message(STATUS "miyabi_logic_cxx target detected. Applying bridge include workaround.")
    get_target_property(MIYABI_LOGIC_CXX_INCLUDES miyabi_logic_cxx INTERFACE_INCLUDE_DIRECTORIES)
    if(MIYABI_LOGIC_CXX_INCLUDES)
        target_include_directories(miyabi_runtime PUBLIC ${MIYABI_LOGIC_CXX_INCLUDES})
        target_include_directories(miyabi PUBLIC ${MIYABI_LOGIC_CXX_INCLUDES})
        message(STATUS "Bridge include workaround applied with INTERFACE_INCLUDE_DIRECTORIES.")
    else()
        message(WARNING "miyabi_logic_cxx exists, but INTERFACE_INCLUDE_DIRECTORIES is empty. Bridge include workaround skipped.")
    endif()
else()
    message(WARNING "miyabi_logic_cxx target not found. Bridge include workaround skipped.")
endif()

target_link_libraries(miyabi_runtime PUBLIC
    box2d
)

# Link against the Rust logic crate and system libraries.
target_link_libraries(miyabi PRIVATE 
    miyabi_logic
    miyabi_runtime
    glfw
    OpenGL::GL
    Freetype::Freetype
)

if(APPLE)
    target_link_options(miyabi PRIVATE "LINKER:-no_warn_duplicate_libraries")
endif()

# Ensure cxx bridge headers are generated before compiling core sources.
if(TARGET miyabi_logic_cxx)
    add_dependencies(miyabi miyabi_logic_cxx)
    add_dependencies(miyabi_runtime miyabi_logic_cxx)
    message(STATUS "Added miyabi_logic_cxx as a build dependency for miyabi and miyabi_runtime.")
else()
    message(WARNING "Skipping miyabi_logic_cxx build dependency because target is missing.")
endif()

# Set the rpath for the executable
set_target_properties(miyabi PROPERTIES
    BUILD_WITH_INSTALL_RPATH TRUE
    INSTALL_RPATH "@executable_path"
)
