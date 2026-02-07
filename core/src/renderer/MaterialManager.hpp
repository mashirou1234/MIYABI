#pragma once

#include <cstdint>
#include <unordered_map>

struct Material {
    uint32_t shader_id;
    uint32_t texture_id = 0; // 0 means no texture
};

class MaterialManager {
public:
    MaterialManager();
    ~MaterialManager();

    // Creates a material for a given shader and returns its ID.
    uint32_t create_material(uint32_t shader_id);

    // Sets the texture for a given material.
    void set_texture(uint32_t material_id, uint32_t texture_id);

    // Returns a pointer to the material, allowing modification.
    Material* get_material(uint32_t material_id);

private:
    uint32_t m_next_material_id;
    std::unordered_map<uint32_t, Material> m_materials;
};
