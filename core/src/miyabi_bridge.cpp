// This file is used by cxx to bridge C++ and Rust.
#include "miyabi/bridge.h"
#include "logic/src/lib.rs.h" // For definitions of shared types like Vec2
#include "physics/PhysicsManager.hpp"

// The miniaudio implementation must be in exactly one C++ file.
#define MINIAUDIO_IMPLEMENTATION
#include "vendor/miniaudio.h"

#include <string>

// --- Global Engine Systems ---
ma_engine g_engine;
miyabi::physics::PhysicsManager g_physics_manager;


// --- FFI Implementations ---

// Audio
void play_sound(rust::Str path) {
    std::string path_str(path);
    ma_engine_play_sound(&g_engine, path_str.c_str(), NULL);
}

// Physics
miyabi::physics::BodyId create_dynamic_box_body(float x, float y, float width, float height) {
    return g_physics_manager.create_dynamic_box(x, y, width, height);
}

miyabi::physics::BodyId create_static_box_body(float x, float y, float width, float height) {
    return g_physics_manager.create_static_box(x, y, width, height);
}

Vec2 get_body_position(miyabi::physics::BodyId id) {
    return g_physics_manager.get_body_position(id);
}

rust::Slice<const ffi::CollisionEvent> get_collision_events() {
    const auto& events = g_physics_manager.get_collision_events();
    return rust::Slice<const ffi::CollisionEvent>(
        reinterpret_cast<const ffi::CollisionEvent*>(events.data()),
        events.size()
    );
}

// --- Engine System Lifecycle ---

void init_engine_systems() {
    // Init Audio
    ma_result result = ma_engine_init(NULL, &g_engine);
    if (result != MA_SUCCESS) {
        printf("Failed to initialize audio engine.\n");
    }

    // Init Physics
    g_physics_manager.init();
}

void step_engine_systems() {
    g_physics_manager.step();
}
