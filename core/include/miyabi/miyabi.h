#pragma once

#include <cstddef>
#include <cstdint>

#define MIYABI_SDK_VERSION_MAJOR 0
#define MIYABI_SDK_VERSION_MINOR 1
#define MIYABI_SDK_VERSION_PATCH 0

#define MIYABI_ABI_VERSION_MAJOR 1
#define MIYABI_ABI_VERSION_MINOR 0
#define MIYABI_ABI_VERSION_PATCH 0
#define MIYABI_ABI_VERSION_ENCODE(major, minor, patch) \
    (((uint32_t)(major) << 16) | ((uint32_t)(minor) << 8) | (uint32_t)(patch))
#define MIYABI_ABI_VERSION \
    MIYABI_ABI_VERSION_ENCODE(MIYABI_ABI_VERSION_MAJOR, MIYABI_ABI_VERSION_MINOR, MIYABI_ABI_VERSION_PATCH)

// Includes the CXX-generated header for shared data types.
// This path is configured in CMake.
#include "miyabi_logic_cxx/lib.h"

// Forward-declare the opaque pointer to the Rust Game.
// C++ must NEVER know its internal layout.
struct Game;

// Defines a non-owning slice of renderable objects.
// This is a BORROW from Rust and is only valid for the duration of a frame.
struct RenderableObjectSlice {
    const RenderableObject* ptr;
    size_t len;

    const RenderableObject* begin() const { return ptr; }
    const RenderableObject* end() const { return ptr + len; }
};

// Defines a non-owning slice of asset commands.
struct AssetCommandSlice {
    const AssetCommand* ptr;
    size_t len;

    const AssetCommand* begin() const { return ptr; }
    const AssetCommand* end() const { return ptr + len; }
};

// Defines a non-owning slice of text commands.
struct TextCommandSlice {
    const TextCommand* ptr;
    size_t len;

    const TextCommand* begin() const { return ptr; }
    const TextCommand* end() const { return ptr + len; }
};

// The complete API provided by the Rust dynamic library, exposed as a C-style VTable.
struct MiyabiVTable {
    uint32_t abi_version;
    Game* (*create_game)();
    void (*destroy_game)(Game* game);
    const char* (*serialize_game)(const Game* game);
    Game* (*deserialize_game)(const char* json);
    void (*free_serialized_string)(char* s);
    void (*update_game)(Game* game);
    RenderableObjectSlice (*get_renderables)(Game* game);
    AssetCommandSlice (*get_asset_commands)(Game* game);
    void (*clear_asset_commands)(Game* game);
    void (*notify_asset_loaded)(Game* game, uint32_t request_id, uint32_t asset_id);
    void (*update_input_state)(Game* game, const InputState& input);
    const char* (*get_asset_command_path_cstring)(const AssetCommand* command);
    TextCommandSlice (*get_text_commands)(Game* game);
    const char* (*get_text_command_text_cstring)(const TextCommand* command);
    void (*free_cstring)(char* s);
};
