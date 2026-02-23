#pragma once

#include "rust/cxx.h"

// Forward-declare types defined in Rust.
struct Vec2;
struct CollisionEvent;

// FFI functions for Rust to call
void play_sound(rust::Str path);
void set_runtime_audio_settings(float master_volume, float bgm_volume, float se_volume);
void request_fullscreen(bool enabled);
uint64_t create_dynamic_box_body(float x, float y, float width, float height);
uint64_t create_static_box_body(float x, float y, float width, float height);
Vec2 get_body_position(uint64_t id);
rust::Slice<const CollisionEvent> get_collision_events();

#if defined(MIYABI_PERFORMANCE_TEST)
uint32_t get_performance_test_sprite_count();
#endif

// FFI functions for C++ to call (implemented in the bridge)
void init_engine_systems();
void step_engine_systems();
bool has_pending_fullscreen_request();
bool consume_pending_fullscreen_request();
