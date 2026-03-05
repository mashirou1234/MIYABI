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
-   **Failure Log Format (minimum):**
    -   `ERROR::SHADER::READ::... path="<file-path>"` (read failure)
    -   `ERROR::SHADER::COMPILE::FAILED shader_type=<VERTEX|FRAGMENT> path="<file-path>" gl_errors=<...>`
    -   `ERROR::SHADER::LINK::FAILED vertex_path="<vs-path>" fragment_path="<fs-path>" gl_errors=<...>`

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

## 6. Frame Performance Observation (Minimum Baseline)

To detect regressions early without over-constraining iteration speed, the renderer defines a minimum set of frame-level observation metrics. These values are for trend monitoring and are treated as **provisional baselines** during active development.

### 6.1. Metrics Table

| Metric | Unit | Measurement Scope | Expected Direction | Why It Matters |
| --- | --- | --- | --- | --- |
| Draw call count | calls/frame | Total GPU draw submissions in one frame | Decrease or hold | Too many calls increase CPU driver overhead |
| Batch count | batches/frame | Number of grouped mesh+material submissions | Decrease or hold | More batches usually mean weaker grouping efficiency |
| Instance count | instances/frame | Total instances submitted via instancing | Increase per draw call, or hold total | Higher packing per call indicates better batching usage |

### 6.2. Capture Timing in the Frame

The metrics are captured at two fixed points to keep sampling consistent:

1. **Frame start:** Reset per-frame counters (`draw_call_count`, `batch_count`, `instance_count`) to zero.
2. **Frame end:** Finalize and emit the counters after all render passes for the frame complete.

This timing model aligns with the existing loop (`get_renderables` -> build draw calls -> execute draw calls -> present), and does not change render behavior.

### 6.3. Log Output Example

```text
[renderer.metrics] frame=1284 draw_calls=42 calls/frame batches=18 batches/frame instances=640 instances/frame trend=provisional
```

The `trend=provisional` marker indicates these are observation values for regression detection, not hard runtime limits.

## 7. Buffer Preconditions (Explicit Contract)

To prevent interpretation drift between Rust scene output and C++ draw submission, buffer handling follows these preconditions:

### 7.1. Creation and Ownership

- `MeshManager` owns static geometry buffers (`VAO`, `VBO`, `EBO`) for each `mesh_id`.
- The renderer owns a per-frame instance buffer (`instance_vbo`) used only for transform/material-instance payloads.
- Buffer handles are created after OpenGL context initialization and destroyed before context teardown.

### 7.2. Validity Requirements Before Draw

For each generated `DrawCall`, all of the following must be true:

1. `mesh_id` resolves to a registered mesh entry with valid `VAO` and index metadata.
2. `material_id` resolves to a registered material entry with a valid linked shader program.
3. `instance_count > 0`; zero-instance batches are skipped and not submitted.
4. The instance payload size equals `instance_count * sizeof(InstanceData)` and fits within the currently allocated instance buffer capacity.

If any requirement fails, the renderer skips that draw call and emits an error log with the failing identifier (`mesh_id` or `material_id`).

### 7.3. Update Rules Per Frame

- Per-frame counters and temporary batch data are reset at frame start.
- Static mesh buffers are immutable during the draw phase of a frame.
- The instance buffer is updated in batch units (`glBufferSubData` or mapped write) before the corresponding instanced draw call.
- Reallocation of instance buffer storage is allowed only before draw submission begins for that frame.

### 7.4. Layout Compatibility

- `InstanceData` layout must be explicitly defined and shared with shader inputs (matrix rows/columns, alignment, and stride).
- Vertex attribute pointer setup for instancing must be performed once per mesh pipeline setup and reused.
- Any change to `InstanceData` requires synchronized updates in:
  - C++ struct definition,
  - vertex attribute declarations,
  - shader input layout documentation.

## 8. Implementation Steps

1.  **Refactor FFI:** Update `cxx::bridge` in `logic/lib.rs` to include `RenderableObject` and the `get_renderables` function. Remove the old command buffer logic.
2.  **Implement Asset Managers:** Create basic `MeshManager`, `ShaderManager`, and `MaterialManager` classes in C++.
3.  **Refactor `main.cpp`:** Restructure the main loop according to the design above. Implement the sorting and batching logic.
4.  **Update Shaders:** Modify the vertex shader to accept per-instance transformation matrices from an instanced vertex buffer.
5.  **Update Rust Logic:** Modify the `World` in `logic` to generate `RenderableObject`s instead of `DrawTriangleCommand`s. Initially, all objects can share mesh_id `0` and material_id `0`.
