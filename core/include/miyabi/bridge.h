#pragma once

#include <rust/cxx.h>

// Declaration for the C++ function that will be called from Rust.
// The implementation is in miyabi_bridge.cpp.
void play_sound(rust::Str path);
