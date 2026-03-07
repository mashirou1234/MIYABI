#include "renderer/MeshManager.hpp"
#include <glad/glad.h>
#include <array>
#include <cstddef>
#include <cmath>
#include <filesystem>
#include <fstream>
#include <iostream>
#include <sstream>
#include <string>
#include <unordered_map>
#include <vector>

namespace {
namespace fs = std::filesystem;

struct ObjVertexKey {
    int position_index;
    int texcoord_index;
    int normal_index;

    bool operator==(const ObjVertexKey& other) const {
        return position_index == other.position_index &&
               texcoord_index == other.texcoord_index &&
               normal_index == other.normal_index;
    }
};

struct ObjVertexKeyHash {
    std::size_t operator()(const ObjVertexKey& key) const {
        return (static_cast<std::size_t>(key.position_index) << 32) ^
               (static_cast<std::size_t>(key.texcoord_index) << 16) ^
               static_cast<std::size_t>(key.normal_index);
    }
};

struct ObjVertex {
    std::array<float, 3> position{};
    std::array<float, 2> texcoord{};
    std::array<float, 3> normal{};
    bool has_explicit_normal = false;
};

std::size_t resolve_obj_index(int index, std::size_t available_count) {
    if (index > 0) {
        return static_cast<std::size_t>(index - 1);
    }
    return static_cast<std::size_t>(
        static_cast<std::ptrdiff_t>(available_count) + static_cast<std::ptrdiff_t>(index)
    );
}

std::array<float, 3> subtract(
    const std::array<float, 3>& lhs,
    const std::array<float, 3>& rhs
) {
    return {
        lhs[0] - rhs[0],
        lhs[1] - rhs[1],
        lhs[2] - rhs[2],
    };
}

std::array<float, 3> cross(
    const std::array<float, 3>& lhs,
    const std::array<float, 3>& rhs
) {
    return {
        lhs[1] * rhs[2] - lhs[2] * rhs[1],
        lhs[2] * rhs[0] - lhs[0] * rhs[2],
        lhs[0] * rhs[1] - lhs[1] * rhs[0],
    };
}

void accumulate_normal(
    std::array<float, 3>& target,
    const std::array<float, 3>& value
) {
    target[0] += value[0];
    target[1] += value[1];
    target[2] += value[2];
}

std::array<float, 3> normalize_or_default(const std::array<float, 3>& value) {
    const float length_sq =
        value[0] * value[0] + value[1] * value[1] + value[2] * value[2];
    if (length_sq <= 0.000001f) {
        return {0.0f, 0.0f, 1.0f};
    }

    const float inverse_length = 1.0f / std::sqrt(length_sq);
    return {
        value[0] * inverse_length,
        value[1] * inverse_length,
        value[2] * inverse_length,
    };
}

std::string resolve_mesh_path(const std::string& requested_path) {
    const fs::path requested(requested_path);
    if (fs::exists(requested)) {
        return requested.string();
    }

    const fs::path repo_relative = fs::path("assets") / "meshes" / requested.filename();
    if (fs::exists(repo_relative)) {
        return repo_relative.string();
    }

    return requested_path;
}

bool build_mesh_from_obj(
    const std::string& requested_path,
    std::vector<float>& vertices,
    std::vector<unsigned int>& indices
) {
    const std::string resolved_path = resolve_mesh_path(requested_path);
    std::ifstream file(resolved_path);
    if (!file.is_open()) {
        std::cerr << "MeshManager::load_obj_mesh - failed to open OBJ path=\""
                  << requested_path << "\" resolved_path=\"" << resolved_path << "\""
                  << std::endl;
        return false;
    }

    std::vector<std::array<float, 3>> positions;
    std::vector<std::array<float, 2>> texcoords;
    std::vector<std::array<float, 3>> normals;
    std::vector<ObjVertex> obj_vertices;
    std::unordered_map<ObjVertexKey, unsigned int, ObjVertexKeyHash> vertex_map;
    std::string line;

    auto parse_obj_vertex = [&](const std::string& token) -> unsigned int {
        std::stringstream token_stream(token);
        std::string position_part;
        std::string texcoord_part;
        std::string normal_part;
        std::getline(token_stream, position_part, '/');
        std::getline(token_stream, texcoord_part, '/');
        std::getline(token_stream, normal_part, '/');

        const int position_index = std::stoi(position_part);
        const int texcoord_index = texcoord_part.empty() ? 0 : std::stoi(texcoord_part);
        const int normal_index = normal_part.empty() ? 0 : std::stoi(normal_part);
        const ObjVertexKey key{position_index, texcoord_index, normal_index};

        auto found = vertex_map.find(key);
        if (found != vertex_map.end()) {
            return found->second;
        }

        const auto& position =
            positions.at(resolve_obj_index(position_index, positions.size()));
        const std::array<float, 2> texcoord =
            texcoord_index > 0
                ? texcoords.at(resolve_obj_index(texcoord_index, texcoords.size()))
                : std::array<float, 2>{0.0f, 0.0f};
        const std::array<float, 3> normal =
            normal_index > 0
                ? normals.at(resolve_obj_index(normal_index, normals.size()))
                : std::array<float, 3>{0.0f, 0.0f, 0.0f};

        obj_vertices.push_back(ObjVertex{
            position,
            texcoord,
            normal,
            normal_index > 0,
        });

        const unsigned int new_index = static_cast<unsigned int>(vertex_map.size());
        vertex_map.emplace(key, new_index);
        return new_index;
    };

    while (std::getline(file, line)) {
        if (line.empty() || line[0] == '#') {
            continue;
        }

        std::stringstream line_stream(line);
        std::string prefix;
        line_stream >> prefix;

        if (prefix == "v") {
            std::array<float, 3> position{};
            line_stream >> position[0] >> position[1] >> position[2];
            positions.push_back(position);
            continue;
        }

        if (prefix == "vt") {
            std::array<float, 2> texcoord{};
            line_stream >> texcoord[0] >> texcoord[1];
            texcoords.push_back(texcoord);
            continue;
        }

        if (prefix == "vn") {
            std::array<float, 3> normal{};
            line_stream >> normal[0] >> normal[1] >> normal[2];
            normals.push_back(normalize_or_default(normal));
            continue;
        }

        if (prefix != "f") {
            continue;
        }

        std::vector<unsigned int> face_indices;
        std::string token;
        while (line_stream >> token) {
            face_indices.push_back(parse_obj_vertex(token));
        }

        if (face_indices.size() < 3) {
            continue;
        }

        for (std::size_t i = 1; i + 1 < face_indices.size(); ++i) {
            const unsigned int triangle_indices[3] = {
                face_indices[0],
                face_indices[i],
                face_indices[i + 1],
            };
            indices.push_back(triangle_indices[0]);
            indices.push_back(triangle_indices[1]);
            indices.push_back(triangle_indices[2]);

            const auto& vertex_a = obj_vertices.at(triangle_indices[0]);
            const auto& vertex_b = obj_vertices.at(triangle_indices[1]);
            const auto& vertex_c = obj_vertices.at(triangle_indices[2]);
            const std::array<float, 3> edge_ab =
                subtract(vertex_b.position, vertex_a.position);
            const std::array<float, 3> edge_ac =
                subtract(vertex_c.position, vertex_a.position);
            const std::array<float, 3> face_normal =
                normalize_or_default(cross(edge_ab, edge_ac));

            for (const unsigned int triangle_index : triangle_indices) {
                ObjVertex& vertex = obj_vertices.at(triangle_index);
                if (!vertex.has_explicit_normal) {
                    accumulate_normal(vertex.normal, face_normal);
                }
            }
        }
    }

    if (obj_vertices.empty() || indices.empty()) {
        std::cerr << "MeshManager::load_obj_mesh - OBJ produced no drawable geometry path=\""
                  << requested_path << "\" resolved_path=\"" << resolved_path << "\""
                  << std::endl;
        return false;
    }

    vertices.reserve(obj_vertices.size() * 8);
    for (auto& vertex : obj_vertices) {
        const std::array<float, 3> normal = normalize_or_default(vertex.normal);
        vertices.push_back(vertex.position[0]);
        vertices.push_back(vertex.position[1]);
        vertices.push_back(vertex.position[2]);
        vertices.push_back(vertex.texcoord[0]);
        vertices.push_back(vertex.texcoord[1]);
        vertices.push_back(normal[0]);
        vertices.push_back(normal[1]);
        vertices.push_back(normal[2]);
    }

    return true;
}

GLMesh upload_mesh(
    const std::vector<float>& vertices,
    const std::vector<unsigned int>& indices
) {
    GLMesh mesh{};
    glGenVertexArrays(1, &mesh.vao);
    glGenBuffers(1, &mesh.vbo);
    glGenBuffers(1, &mesh.ebo);

    glBindVertexArray(mesh.vao);

    glBindBuffer(GL_ARRAY_BUFFER, mesh.vbo);
    glBufferData(
        GL_ARRAY_BUFFER,
        static_cast<GLsizeiptr>(vertices.size() * sizeof(float)),
        vertices.data(),
        GL_STATIC_DRAW
    );

    glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, mesh.ebo);
    glBufferData(
        GL_ELEMENT_ARRAY_BUFFER,
        static_cast<GLsizeiptr>(indices.size() * sizeof(unsigned int)),
        indices.data(),
        GL_STATIC_DRAW
    );

    const GLsizei stride = 8 * sizeof(float);
    glVertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE, stride, (void*)0);
    glEnableVertexAttribArray(0);
    glVertexAttribPointer(1, 2, GL_FLOAT, GL_FALSE, stride, (void*)(3 * sizeof(float)));
    glEnableVertexAttribArray(1);
    glVertexAttribPointer(2, 3, GL_FLOAT, GL_FALSE, stride, (void*)(5 * sizeof(float)));
    glEnableVertexAttribArray(2);

    glBindVertexArray(0);
    mesh.element_count = static_cast<uint32_t>(indices.size());
    return mesh;
}
} // namespace

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

uint32_t MeshManager::create_quad_mesh() {
    const std::vector<float> vertices = {
        0.5f, 0.5f, 0.0f, 1.0f, 1.0f, 0.0f, 0.0f, 1.0f,
        0.5f, -0.5f, 0.0f, 1.0f, 0.0f, 0.0f, 0.0f, 1.0f,
        -0.5f, -0.5f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 1.0f,
        -0.5f, 0.5f, 0.0f, 0.0f, 1.0f, 0.0f, 0.0f, 1.0f,
    };
    const std::vector<unsigned int> indices = {
        0, 1, 3,
        1, 2, 3,
    };

    GLMesh mesh = upload_mesh(vertices, indices);
    uint32_t mesh_id = m_next_mesh_id++;
    m_meshes[mesh_id] = mesh;

    return mesh_id;
}

uint32_t MeshManager::load_obj_mesh(uint32_t mesh_id, const std::string& path) {
    if (m_meshes.find(mesh_id) != m_meshes.end()) {
        std::cerr << "MeshManager::load_obj_mesh - Mesh ID " << mesh_id
                  << " already registered." << std::endl;
        return 0;
    }

    std::vector<float> vertices;
    std::vector<unsigned int> indices;
    if (!build_mesh_from_obj(path, vertices, indices)) {
        return 0;
    }

    m_meshes[mesh_id] = upload_mesh(vertices, indices);
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
