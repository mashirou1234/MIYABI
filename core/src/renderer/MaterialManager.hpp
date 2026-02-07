#pragma once

#include <cstdint>
#include <unordered_map>

struct Material {
    uint32_t shader_id;
    // In the future, this could hold texture IDs, colors, etc.
};

class MaterialManager {
public:
    MaterialManager();
    ~MaterialManager();

    // Creates a material for a given shader and returns its ID.
    uint32_t create_material(uint32_t shader_id);

    const Material* get_material(uint32_t material_id) const;

private:
    uint32_t m_next_material_id;
    std::unordered_map<uint32_t, Material> m_materials;
};
