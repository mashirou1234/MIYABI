// This file is used by cxx to bridge C++ and Rust.
#include "miyabi/bridge.h"
#include "miyabi_logic_cxx/lib.h" // For definitions of shared types like Vec2
#include "physics/PhysicsManager.hpp"

// The miniaudio implementation must be in exactly one C++ file.
#define MINIAUDIO_IMPLEMENTATION
#include "vendor/miniaudio.h"

#include <algorithm>
#include <atomic>
#include <mutex>
#include <string>

// --- Global Engine Systems ---
ma_engine g_engine;
ma_sound_group g_bgm_group;
ma_sound_group g_se_group;
ma_sound g_bgm_sound;
miyabi::physics::PhysicsManager g_physics_manager;
std::atomic<bool> g_audio_ready{false};
std::atomic<bool> g_bgm_group_ready{false};
std::atomic<bool> g_se_group_ready{false};
std::atomic<bool> g_bgm_sound_ready{false};
std::atomic<bool> g_pending_fullscreen{false};
std::atomic<bool> g_requested_fullscreen{false};
std::mutex g_bgm_mutex;


// --- FFI Implementations ---

// Audio
void play_sound(rust::Str path) {
    if (!g_audio_ready.load(std::memory_order_acquire)) {
        return;
    }
    std::string path_str(path);
    ma_sound_group* group = g_se_group_ready.load(std::memory_order_acquire) ? &g_se_group : NULL;
    ma_engine_play_sound(&g_engine, path_str.c_str(), group);
}

void play_bgm(rust::Str path, bool looped) {
    if (!g_audio_ready.load(std::memory_order_acquire)) {
        return;
    }

    std::lock_guard<std::mutex> lock(g_bgm_mutex);
    if (g_bgm_sound_ready.load(std::memory_order_acquire)) {
        ma_sound_stop(&g_bgm_sound);
        ma_sound_uninit(&g_bgm_sound);
        g_bgm_sound_ready.store(false, std::memory_order_release);
    }

    std::string path_str(path);
    ma_sound_group* group = g_bgm_group_ready.load(std::memory_order_acquire) ? &g_bgm_group : NULL;
    ma_result result = ma_sound_init_from_file(
        &g_engine,
        path_str.c_str(),
        MA_SOUND_FLAG_STREAM,
        group,
        NULL,
        &g_bgm_sound
    );
    if (result != MA_SUCCESS) {
        printf("Failed to initialize BGM sound: %s\n", path_str.c_str());
        return;
    }

    ma_sound_set_looping(&g_bgm_sound, looped ? MA_TRUE : MA_FALSE);
    result = ma_sound_start(&g_bgm_sound);
    if (result != MA_SUCCESS) {
        printf("Failed to start BGM sound: %s\n", path_str.c_str());
        ma_sound_uninit(&g_bgm_sound);
        return;
    }

    g_bgm_sound_ready.store(true, std::memory_order_release);
}

void stop_bgm() {
    std::lock_guard<std::mutex> lock(g_bgm_mutex);
    if (!g_bgm_sound_ready.load(std::memory_order_acquire)) {
        return;
    }

    ma_sound_stop(&g_bgm_sound);
    ma_sound_uninit(&g_bgm_sound);
    g_bgm_sound_ready.store(false, std::memory_order_release);
}

void set_runtime_audio_settings(float master_volume, float bgm_volume, float se_volume) {
    if (!g_audio_ready.load(std::memory_order_acquire)) {
        return;
    }

    const float master = std::clamp(master_volume, 0.0f, 1.0f);
    const float bgm = std::clamp(bgm_volume, 0.0f, 1.0f);
    const float se = std::clamp(se_volume, 0.0f, 1.0f);

    ma_engine_set_volume(&g_engine, master);
    if (g_bgm_group_ready.load(std::memory_order_acquire)) {
        ma_sound_group_set_volume(&g_bgm_group, bgm);
    }
    if (g_se_group_ready.load(std::memory_order_acquire)) {
        ma_sound_group_set_volume(&g_se_group, se);
    }
}

void request_fullscreen(bool enabled) {
    g_requested_fullscreen.store(enabled, std::memory_order_release);
    g_pending_fullscreen.store(true, std::memory_order_release);
}

// Physics
miyabi::physics::PhysicsManager::BodyId create_dynamic_box_body(float x, float y, float width, float height) {
    return g_physics_manager.create_dynamic_box(x, y, width, height);
}

miyabi::physics::PhysicsManager::BodyId create_static_box_body(float x, float y, float width, float height) {
    return g_physics_manager.create_static_box(x, y, width, height);
}

Vec2 get_body_position(miyabi::physics::PhysicsManager::BodyId id) {
    return g_physics_manager.get_body_position(id);
}

rust::Slice<const CollisionEvent> get_collision_events() {
    const auto& events = g_physics_manager.get_collision_events();
    return rust::Slice<const CollisionEvent>(
        reinterpret_cast<const CollisionEvent*>(events.data()),
        events.size()
    );
}

// --- Engine System Lifecycle ---

void init_engine_systems() {
    // Init Audio
    ma_result result = ma_engine_init(NULL, &g_engine);
    if (result != MA_SUCCESS) {
        printf("Failed to initialize audio engine.\n");
    } else {
        g_audio_ready.store(true, std::memory_order_release);

        ma_result bgm_group_result = ma_sound_group_init(&g_engine, 0, NULL, &g_bgm_group);
        if (bgm_group_result == MA_SUCCESS) {
            g_bgm_group_ready.store(true, std::memory_order_release);
        } else {
            printf("Failed to initialize BGM sound group.\n");
        }

        ma_result se_group_result = ma_sound_group_init(&g_engine, 0, NULL, &g_se_group);
        if (se_group_result == MA_SUCCESS) {
            g_se_group_ready.store(true, std::memory_order_release);
        } else {
            printf("Failed to initialize SE sound group.\n");
        }
    }

    // Init Physics
    g_physics_manager.init();
}

void shutdown_engine_systems() {
    stop_bgm();

    if (g_se_group_ready.load(std::memory_order_acquire)) {
        ma_sound_group_uninit(&g_se_group);
        g_se_group_ready.store(false, std::memory_order_release);
    }
    if (g_bgm_group_ready.load(std::memory_order_acquire)) {
        ma_sound_group_uninit(&g_bgm_group);
        g_bgm_group_ready.store(false, std::memory_order_release);
    }
    if (g_audio_ready.load(std::memory_order_acquire)) {
        ma_engine_uninit(&g_engine);
        g_audio_ready.store(false, std::memory_order_release);
    }
}

void step_engine_systems() {
    g_physics_manager.step();
}

bool has_pending_fullscreen_request() {
    return g_pending_fullscreen.load(std::memory_order_acquire);
}

bool consume_pending_fullscreen_request() {
    g_pending_fullscreen.store(false, std::memory_order_release);
    return g_requested_fullscreen.load(std::memory_order_acquire);
}
