#include "renderer/MeshManager.hpp"
#include <glad/glad.h>
#include <vector>
#include <iostream>

MeshManager::MeshManager() : m_next_mesh_id(1) {}

MeshManager::~MeshManager() {
    for (auto const& [id, mesh] : m_meshes) {
        glDeleteVertexArrays(1, &mesh.vao);
        glDeleteBuffers(1, &mesh.vbo);
        if (mesh.ebo != 0) {
            glDeleteBuffers(1, &mesh.ebo);
        }
    }
}

uint32_t MeshManager::create_triangle_mesh() {
    // For now, hardcode a simple triangle
    float vertices[] = {
        -0.5f, -0.5f, 0.0f,
         0.5f, -0.5f, 0.0f,
         0.0f,  0.5f, 0.0f
    };

    // No indices needed for a single triangle drawn with glDrawArrays
    // but we'll use indices for good practice with glDrawElements
    unsigned int indices[] = {
        0, 1, 2
    };

    GLMesh mesh{};
    glGenVertexArrays(1, &mesh.vao);
    glGenBuffers(1, &mesh.vbo);
    glGenBuffers(1, &mesh.ebo);

    glBindVertexArray(mesh.vao);

    glBindBuffer(GL_ARRAY_BUFFER, mesh.vbo);
    glBufferData(GL_ARRAY_BUFFER, sizeof(vertices), vertices, GL_STATIC_DRAW);

    glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, mesh.ebo);
    glBufferData(GL_ELEMENT_ARRAY_BUFFER, sizeof(indices), indices, GL_STATIC_DRAW);

    // Position attribute
    glVertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE, 3 * sizeof(float), (void*)0);
    glEnableVertexAttribArray(0);

    glBindVertexArray(0); // Unbind VAO

    mesh.element_count = 3;

    uint32_t mesh_id = m_next_mesh_id++;
    m_meshes[mesh_id] = mesh;

    return mesh_id;
}

void MeshManager::bind_mesh(uint32_t mesh_id) const {
    auto it = m_meshes.find(mesh_id);
    if (it != m_meshes.end()) {
        glBindVertexArray(it->second.vao);
    } else {
        std::cerr << "MeshManager::bind_mesh - Mesh ID " << mesh_id << " not found." << std::endl;
        glBindVertexArray(0);
    }
}

const GLMesh* MeshManager::get_mesh(uint32_t mesh_id) const {
    auto it = m_meshes.find(mesh_id);
    if (it != m_meshes.end()) {
        return &it->second;
    }
    return nullptr;
}
