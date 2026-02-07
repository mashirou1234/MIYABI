#include "renderer/MaterialManager.hpp"
#include <iostream>

MaterialManager::MaterialManager() : m_next_material_id(1) {}

MaterialManager::~MaterialManager() = default;

uint32_t MaterialManager::create_material(uint32_t shader_id) {
    uint32_t material_id = m_next_material_id++;
    m_materials[material_id] = { shader_id, 0 };
    return material_id;
}

void MaterialManager::set_texture(uint32_t material_id, uint32_t texture_id) {
    auto it = m_materials.find(material_id);
    if (it != m_materials.end()) {
        it->second.texture_id = texture_id;
    } else {
        std::cerr << "MaterialManager::set_texture - Material ID " << material_id << " not found." << std::endl;
    }
}

Material* MaterialManager::get_material(uint32_t material_id) {
    auto it = m_materials.find(material_id);
    if (it != m_materials.end()) {
        return &it->second;
    }
    std::cerr << "MaterialManager::get_material - Material ID " << material_id << " not found." << std::endl;
    return nullptr;
}
