#pragma once

#include <string>
#include <cstdint>
#include <unordered_map>

class TextureManager {
public:
    TextureManager();
    ~TextureManager();

    // Loads a texture from a file path.
    // Returns a texture_id, or 0 if loading fails.
    uint32_t load_texture(const std::string& path);

    // Binds the specified texture to the given texture unit (e.g., GL_TEXTURE0).
    void bind_texture(uint32_t texture_id, uint32_t texture_unit) const;

private:
    uint32_t m_next_texture_id;
    std::unordered_map<uint32_t, uint32_t> m_texture_id_to_gl_id;
};
