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
    src/renderer/RenderBatching.cpp
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

# HACK: CXX bridge include の伝播可否を明示し、対象ターゲット不在時は警告する。
if(TARGET miyabi_logic_cxx)
    get_target_property(MIYABI_LOGIC_CXX_INCLUDES miyabi_logic_cxx INTERFACE_INCLUDE_DIRECTORIES)
    if(MIYABI_LOGIC_CXX_INCLUDES)
        message(STATUS "miyabi_logic_cxx include propagation enabled")
        target_include_directories(miyabi_runtime PUBLIC ${MIYABI_LOGIC_CXX_INCLUDES})
        target_include_directories(miyabi PUBLIC ${MIYABI_LOGIC_CXX_INCLUDES})
    else()
        message(WARNING "miyabi_logic_cxx exists but INTERFACE_INCLUDE_DIRECTORIES is empty")
    endif()
else()
    message(WARNING "miyabi_logic_cxx target not found; skip bridge include propagation and dependency wiring")
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

# CXX bridge ターゲットがある場合のみ依存を張る。
if(TARGET miyabi_logic_cxx)
    add_dependencies(miyabi miyabi_logic_cxx)
    add_dependencies(miyabi_runtime miyabi_logic_cxx)
endif()

include(CTest)
if(BUILD_TESTING)
    add_executable(render_batching_test
        tests/render_batching_test.cpp
        src/renderer/RenderBatching.cpp
    )
    target_include_directories(render_batching_test PRIVATE
        include
        src
    )
    if(MIYABI_LOGIC_CXX_INCLUDES)
        target_include_directories(
            render_batching_test PRIVATE ${MIYABI_LOGIC_CXX_INCLUDES}
        )
    endif()
    if(TARGET miyabi_logic_cxx)
        add_dependencies(render_batching_test miyabi_logic_cxx)
    endif()
    add_test(NAME render_batching_test COMMAND render_batching_test)
endif()

# Set the rpath for the executable
set_target_properties(miyabi PROPERTIES
    BUILD_WITH_INSTALL_RPATH TRUE
    INSTALL_RPATH "@executable_path"
)
