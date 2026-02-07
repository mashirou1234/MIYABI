#pragma once

#include <cstdint>
#include <vector>
#include <unordered_map>

struct GLMesh {
    uint32_t vao;
    uint32_t vbo;
    uint32_t ebo; // Element Buffer Object
    uint32_t element_count;
};

class MeshManager {
public:
    MeshManager();
    ~MeshManager();

    // Creates a simple triangle mesh and returns its ID.
    uint32_t create_triangle_mesh();

    // Binds the VAO for the given mesh ID for drawing.
    void bind_mesh(uint32_t mesh_id) const;

    const GLMesh* get_mesh(uint32_t mesh_id) const;

private:
    uint32_t m_next_mesh_id;
    std::unordered_map<uint32_t, GLMesh> m_meshes;
};
