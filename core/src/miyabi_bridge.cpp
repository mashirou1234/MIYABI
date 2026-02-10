// This file is used by cxx to bridge C++ and Rust.
#include "miyabi/bridge.h"

// The miniaudio implementation must be in exactly one C++ file.
#define MINIAUDIO_IMPLEMENTATION
#include "vendor/miniaudio.h"

#include <string>

// Global audio engine, accessible from main.cpp via 'extern'.
ma_engine g_engine;

// FFI implementation for the function declared in Rust's cxx bridge.
void play_sound(rust::Str path) {
    std::string path_str(path);
    ma_engine_play_sound(&g_engine, path_str.c_str(), NULL);
}
