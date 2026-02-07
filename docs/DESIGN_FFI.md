# MIYABI FFI Technical Design (Detailed)

## 1. Document Purpose and Guiding Principles

This document provides an exhaustive technical specification for the Foreign Function Interface (FFI) between the Rust logic crate (`logic`) and the C++ host application (`core`). Its purpose is to define an unambiguous, safe, and performant boundary that explicitly supports the engine's hot-reloading architecture.

This design acknowledges that the FFI boundary is the most error-prone part of a hybrid engine. Therefore, it establishes crystal-clear rules for data ownership, memory management, and function invocation to eliminate ambiguity.

### Guiding Principles:

1.  **Minimalism:** The API surface exposed across the FFI boundary will be as small and contained as possible.
2.  **Explicit Ownership:** It must be blindingly obvious who owns any piece of data crossing the boundary and who is responsible for freeing it. There will be no exceptions.
3.  **Zero-Cost Data Sharing:** Data structures will be shared with guaranteed memory layout compatibility, avoiding any serialization or conversion costs for per-frame data transfer.
4.  **Hot-Reloading as a First-Class Citizen:** The entire design is built around enabling the hot-reloading of the Rust dylib. This justifies the choice of a dynamic C API over a static C++ binding.

## 2. The Chosen Strategy: A Hybrid C-VTable Approach

To satisfy all principles, we will adopt a formal hybrid strategy:

-   **`cxx` for Data Structures ONLY:** The `cxx` library is unparalleled for creating safe, shared data types with guaranteed layout compatibility. We will use it **exclusively** for this purpose. All structs passed across the boundary (e.g., `Vec3`, `Transform`, `RenderableObject`) will be defined in a single `#[cxx::bridge]` module. We will **not** use `cxx`'s `extern "Rust"` or `extern "C++"` blocks for functions.

-   **Dynamic C Virtual Table (`VTable`) for Functions:** All function calls will be exposed through a single, pure-C `struct` that acts as a virtual table. The C++ host will load a single function from the dylib, `get_miyabi_vtable()`, which returns this struct containing pointers to all other API functions. This approach:
    -   Centralizes all dynamic symbol loading (`dlsym`) into a single call on startup/reload.
    -   Makes the API contract explicit and versionable.
    -   Avoids any C++ name-mangling issues, as it's a pure C interface.

## 3. The FFI API Specification: `MiyabiVTable`

This is the complete definition of the API exposed by the Rust library.

### 3.1. C++ Header Definition (`core/include/miyabi/miyabi.h`)

This file will define the shared data structures (via the `cxx` generated header) and the VTable itself.

```cpp
#pragma once

// Includes the CXX-generated header for shared data types.
// This path is configured in CMake.
#include "miyabi_cxxbridge/logic/src/lib.rs.h"

// Forward-declare the opaque pointer to the Rust World.
// C++ must NEVER know its internal layout.
struct World;

// Defines a non-owning slice of renderable objects.
// This is a BORROW from Rust.
struct RenderableObjectSlice {
    const ffi::RenderableObject* ptr;
    size_t len;

    const ffi::RenderableObject* begin() const { return ptr; }
    const ffi::RenderableObject* end() const { return ptr + len; }
};

// The complete API provided by the Rust dynamic library.
struct MiyabiVTable {
    // --- Lifecycle ---
    // Creates the World. Returns an OWNED pointer.
    World* (*create_world)();
    // Destroys the World. Takes an OWNED pointer.
    void (*destroy_world)(World* world);

    // --- Hot-Reloading ---
    // Serializes the World state. Returns an OWNED C-string pointer.
    const char* (*serialize_world)(const World* world);
    // Deserializes the World state. Takes a BORROWED C-string. Returns a NEW OWNED World pointer.
    World* (*deserialize_world)(const char* json);
    // Frees the C-string returned by serialize_world. Takes an OWNED pointer.
    void (*free_serialized_string)(char* s);

    // --- Per-Frame ---
    // Runs all game logic systems for one frame.
    void (*run_logic_systems)(World* world);
    // Gets the list of objects to draw. Returns a BORROWED slice.
    RenderableObjectSlice (*get_renderables)(World* world);

    // --- Input (Future Expansion) ---
    // void (*update_input_state)(World* world, const InputState& input);
};
```

### 3.2. Rust VTable Creation (`logic/src/lib.rs`)

Rust will implement a single `#[no_mangle]` function to construct and return the VTable.

```rust
#[no_mangle]
pub extern "C" fn get_miyabi_vtable() -> MiyabiVTable {
    MiyabiVTable {
        create_world: rust_create_world,
        destroy_world: rust_destroy_world,
        serialize_world: rust_serialize_world,
        deserialize_world: rust_deserialize_world,
        free_serialized_string: rust_free_serialized_string,
        run_logic_systems: rust_run_logic_systems,
        get_renderables: rust_get_renderables,
    }
}

// Example bridge function (all others follow this pattern)
#[no_mangle]
extern "C" fn rust_create_world() -> *mut World {
    Box::into_raw(Box::new(World::new()))
}
```

## 4. Ownership and Memory Management: The Unbreakable Rules

Violating these rules will lead to memory leaks, use-after-free, or crashes.

1.  **`World*` Pointer:**
    -   **Ownership:** An opaque, **OWNED** pointer.
    -   **Source:** Returned by `create_world()` or `deserialize_world()`.
    -   **Responsibility:** C++ receives and holds this pointer. It **MUST NOT** attempt to `delete` or `free` it. It **MUST** eventually pass it to `destroy_world()` to be deallocated correctly on the Rust side.

2.  **`RenderableObjectSlice` Struct:**
    -   **Ownership:** A temporary, non-owning **BORROW**.
    -   **Source:** Returned by `get_renderables()`.
    -   **Responsibility:** The pointer and data within this slice are valid **ONLY** until the next call to any function in the `MiyabiVTable`. C++ **MUST NOT** store the pointer. If the data is needed longer, it must be copied into C++-owned memory.

3.  **`const char*` from `serialize_world()`:**
    -   **Ownership:** An **OWNED** C-string pointer, passed from Rust to C++.
    -   **Source:** Returned by `serialize_world()`. Rust allocates this memory.
    -   **Responsibility:** C++ receives this pointer and is now responsible for it. It **MUST** call `free_serialized_string()` on the pointer when it is no longer needed to prevent a memory leak.

4.  **`const char*` to `deserialize_world()`:**
    -   **Ownership:** A **BORROW** of a C-string owned by C++.
    -   **Source:** C++ provides this pointer (which it got from a previous `serialize_world` call).
    -   **Responsibility:** Rust will read the data but **MUST NOT** store the pointer or attempt to free it.

## 5. Precise Hot-Reloading Workflow

This workflow ensures a seamless transition between library versions with no memory leaks.

1.  **File Change Detected:** The file watcher signals a change. The main loop is flagged for reload at the end of the current frame.
2.  **Serialize State:** C++ calls `vtable.serialize_world(world)`. It receives and stores the `const char* json_state`.
3.  **Destroy Old World:** C++ calls `vtable.destroy_world(world)`. The old Rust world and all its components are now gone.
4.  **Unload Dylib:** C++ calls `dlclose(handle)`. All memory associated with the old library is now released by the OS. The `vtable` is now a dangling pointer.
5.  **Rebuild:** The C++ host triggers the command `cmake --build build` to create the new `.dylib`.
6.  **Load New Dylib:** C++ calls `dlopen("liblogic.dylib")` to get a `new_handle`.
7.  **Get New VTable:** C++ calls `dlsym(new_handle, "get_miyabi_vtable")` to get the address of the VTable factory function. It calls this function to get the `new_vtable`.
8.  **Deserialize State:** C++ calls `new_vtable.deserialize_world(json_state)`. This uses the new library's code to construct a new `World` from the old state. It returns a `new_world` pointer.
9.  **Free Serialized State:** C++ calls `new_vtable.free_serialized_string((char*)json_state)`. The temporary JSON string is now deallocated by the new library.
10. **Continue:** The main loop proceeds with the `new_world` pointer and the `new_vtable`. The hot-reload is complete.
