cmake_minimum_required(VERSION 3.25)

# 【変更点1】 project() よりも前に定義を配置し、拡張機能(gnu++17など)をOFFにする
set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED TRUE)
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED TRUE)
set(CMAKE_CXX_EXTENSIONS OFF)

project(MIYABI LANGUAGES CXX C)

# Option to enable performance profiling
option(MIYABI_PROFILE "Enable performance profiling instrumentation" OFF)
if(MIYABI_PROFILE)
    message(STATUS "Performance profiling enabled.")
    add_compile_definitions(MIYABI_PROFILE)
endif()

# Option to enable performance testing
option(MIYABI_PERFORMANCE_TEST "Enable performance testing" OFF)
if(MIYABI_PERFORMANCE_TEST)
    message(STATUS "Performance testing enabled.")
    add_compile_definitions(MIYABI_PERFORMANCE_TEST)
endif()

# 【変更点2】 GLMライブラリに対して「C++17を使ってビルドせよ」と明示的に指示するマクロ
# これがないと、コンパイラ設定が正しくてもGLM内部でC++98判定されることがあります
add_compile_definitions(GLM_FORCE_CXX17)
add_compile_options(-Wno-c++98-compat -Wno-c++98-compat-pedantic -Wno-poison-system-directories)

# 【変更点3】 Windows (MSVC) を使用している場合の特効薬
# MSVCは標準では __cplusplus マクロが 199711L (C++98相当) を返し続ける仕様があるため、それを修正します
if(MSVC)
    add_compile_options(/Zc:__cplusplus)
endif()

# Fetch Corrosion, a tool for integrating Rust into CMake projects.
include(FetchContent)
FetchContent_Declare(
    Corrosion
    GIT_REPOSITORY https://github.com/corrosion-rs/corrosion.git
    GIT_TAG v0.6.1 # Or a newer version
)
FetchContent_MakeAvailable(Corrosion)

FetchContent_Declare(
    box2d
    GIT_REPOSITORY https://github.com/erincatto/box2d.git
    GIT_TAG v2.4.1
)
set(BOX2D_BUILD_UNIT_TESTS OFF CACHE BOOL "Disable Box2D unit tests")
set(BOX2D_BUILD_TESTBED OFF CACHE BOOL "Disable Box2D testbed")
FetchContent_MakeAvailable(box2d)
FetchContent_GetProperties(box2d
    SOURCE_DIR box2d_SOURCE_DIR
)
set(BOX2D_INCLUDE_DIR ${box2d_SOURCE_DIR}/include)
configure_file(logic/src/paths.rs.in ${CMAKE_CURRENT_SOURCE_DIR}/logic/src/paths.rs @ONLY)

add_subdirectory(logic)
add_subdirectory(core)
