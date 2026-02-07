#pragma once

// Includes the CXX-generated header for shared data types.
// This path is relative to the include directory provided by `corrosion_add_cxxbridge`.
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

// The complete API provided by the Rust dynamic library, exposed as a C-style VTable.
struct MiyabiVTable {
    // --- Lifecycle ---
    World* (*create_world)();
    void (*destroy_world)(World* world);

    // --- Hot-Reloading ---
    const char* (*serialize_world)(const World* world);
    World* (*deserialize_world)(const char* json);
    void (*free_serialized_string)(char* s);

    // --- Per-Frame ---
    void (*run_logic_systems)(World* world);
    RenderableObjectSlice (*get_renderables)(World* world);

    // --- Input (Future Expansion) ---
    // void (*update_input_state)(World* world, const InputState& input);
};
