# MIYABI Renderer Technical Design

## 1. Overview

This document details the technical design for the MIYABI rendering engine. The primary goal is to evolve the current single-triangle, multi-draw-call prototype into an efficient, data-driven, and batch-oriented rendering pipeline.

This design directly addresses the goals in `PLAN.md` (Phase 2) and provides the "precision" requested to move beyond placeholder implementations.

## 2. Core Rendering Strategy: Instanced Batching by Material

The current approach of one draw call per object (`glDrawArrays`) is highly inefficient and does not scale. We will replace this with a strategy that groups objects by their material and draws each group with a single instanced draw call.

-   **Batching:** A "batch" is a collection of objects that share the same **mesh** and **material**.
-   **Instancing:** If multiple objects share the same mesh and material but have different transforms, we will use instanced rendering (`glDrawElementsInstanced`) to render them all in one call.

## 3. Data Flow and FFI Boundary

The communication from Rust (logic) to C++ (rendering) will be refined.

### 3.1. Rust-Side (`logic` crate)

-   Rust will no longer send low-level `DrawTriangleCommand`s.
-   Instead, it will construct a scene description composed of `RenderableObject` structs.
-   The `World` will have a function `get_renderables() -> Vec<RenderableObject>` that C++ can call.

```rust
// In logic/src/lib.rs (inside the cxx::bridge)

pub struct RenderableObject {
    pub mesh_id: u32,
    pub material_id: u32,
    pub transform: Transform,
}
```

### 3.2. C++ Side (`core` crate)

1.  **Frame Start:** C++ calls `world->get_renderables()` via FFI to get a `rust::Vec<RenderableObject>`.
2.  **Sorting & Batching:** C++ iterates through the `RenderableObject` vector and sorts it by `material_id`, then by `mesh_id`. This groups all compatible objects together.
3.  **Building Draw Calls:** The sorted list is processed to create a list of `DrawCall` structs. Each `DrawCall` represents a single `glDrawElementsInstanced` operation.

```cpp
// In a new file, core/src/renderer.hpp

struct DrawCall {
    uint32_t mesh_id;
    uint32_t material_id;
    uint32_t instance_count;
    // Potentially a pointer/offset to instance data
    // (e.g., a buffer of transforms)
};
```

4.  **Execution:** The renderer iterates through the `DrawCall` list and executes the OpenGL commands.

## 4. Asset Management (C++)

To support the ID-based system (`mesh_id`, `material_id`), we need simple asset managers in C++.

### 4.1. `MeshManager`

-   **Responsibilities:** Loads vertex data from files (e.g., `.obj` initially) or procedural generation, creates OpenGL VAOs/VBOs/EBOs, and stores them.
-   **API:**
    -   `uint32_t load_mesh(const std::string& path);`
    -   `void bind_mesh(uint32_t mesh_id);`
-   **Storage:** A `std::map<uint32_t, GLHandles>` where `GLHandles` is a struct containing VAO, VBO, EBO IDs and vertex count.

### 4.2. `ShaderManager`

-   **Responsibilities:** Loads, compiles, and links vertex and fragment shaders into shader programs.
-   **API:**
    -   `uint32_t load_shader(const std::string& vs_path, const std::string& fs_path);`
    -   `void use_shader(uint32_t shader_id);`
-   **Storage:** `std::map<uint32_t, GLuint> shader_programs;`

### 4.3. `MaterialManager`

-   **Responsibilities:** Defines a material, which is a combination of a shader and its parameters (e.g., textures, colors).
-   **API:**
    -   `uint32_t create_material(uint32_t shader_id);`
    -   `void set_texture(uint32_t material_id, uint32_t texture_id);` // Future
    -   `Material& get_material(uint32_t material_id);`
-   **Storage:** `std::map<uint32_t, Material>` where `Material` is a struct containing `shader_id`, texture IDs, etc.

## 5. Revised Main Loop (`main.cpp`)

The main render loop will be significantly restructured.

```cpp
// Simplified pseudo-code for the main loop

// Initialization
// ... create managers (Mesh, Shader, Material)
// ... load default assets (e.g., mesh_id 0 = triangle, material_id 0 = default shader)

while (!glfwWindowShouldClose(window)) {
    // ...
    // Get renderables from Rust
    auto renderables = world->get_renderables();

    // Sort and batch
    auto draw_calls = build_draw_calls(renderables);

    // Execute drawing
    glClear(...);
    for (const auto& call : draw_calls) {
        // 1. Set state from material
        material_manager.use_material(call.material_id); // Binds shader, textures etc.

        // 2. Bind mesh
        mesh_manager.bind_mesh(call.mesh_id);

        // 3. Update instance buffer (if needed) with transform data for this batch
        // ... update_instance_vbo(...)

        // 4. Draw
        glDrawElementsInstanced(GL_TRIANGLES, /* vertex count */, GL_UNSIGNED_INT, 0, call.instance_count);
    }

    glfwSwapBuffers(window);
    glfwPollEvents();
}
```

## 6. Implementation Steps

1.  **Refactor FFI:** Update `cxx::bridge` in `logic/lib.rs` to include `RenderableObject` and the `get_renderables` function. Remove the old command buffer logic.
2.  **Implement Asset Managers:** Create basic `MeshManager`, `ShaderManager`, and `MaterialManager` classes in C++.
3.  **Refactor `main.cpp`:** Restructure the main loop according to the design above. Implement the sorting and batching logic.
4.  **Update Shaders:** Modify the vertex shader to accept per-instance transformation matrices from an instanced vertex buffer.
5.  **Update Rust Logic:** Modify the `World` in `logic` to generate `RenderableObject`s instead of `DrawTriangleCommand`s. Initially, all objects can share mesh_id `0` and material_id `0`.
