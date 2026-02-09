#pragma once

// Includes the CXX-generated header for shared data types.
// This path is configured in CMake.
#include "miyabi_cxxbridge/lib.h"

// Forward-declare the opaque pointer to the Rust World.
// C++ must NEVER know its internal layout.
struct World;

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
    World* (*create_world)();
    void (*destroy_world)(World* world);
    const char* (*serialize_world)(const World* world);
    World* (*deserialize_world)(const char* json);
    void (*free_serialized_string)(char* s);
    void (*run_logic_systems)(World* world);
    RenderableObjectSlice (*get_renderables)(World* world);
    AssetCommandSlice (*get_asset_commands)(World* world);
    void (*clear_asset_commands)(World* world);
    void (*notify_asset_loaded)(World* world, uint32_t request_id, uint32_t asset_id);
    void (*update_input_state)(World* world, const InputState& input);
    const char* (*get_asset_command_path_cstring)(const AssetCommand* command);
    TextCommandSlice (*get_text_commands)(World* world);
    const char* (*get_text_command_text_cstring)(const TextCommand* command);
    void (*free_cstring)(char* s);
};
