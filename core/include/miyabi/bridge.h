#pragma once

#include "rust/cxx.h"

struct Vec2;

// FFI functions for Rust to call
void play_sound(rust::Str path);
uint64_t create_dynamic_box_body(float x, float y, float width, float height);
uint64_t create_static_box_body(float x, float y, float width, float height);
Vec2 get_body_position(uint64_t id);

// FFI functions for C++ to call (implemented in the bridge)
void init_engine_systems();
void step_engine_systems();
