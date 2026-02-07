#include "renderer/TextureManager.hpp"
#include <glad/glad.h>
#include <iostream>

#define STB_IMAGE_IMPLEMENTATION
#include "vendor/stb_image.h"

TextureManager::TextureManager() : m_next_texture_id(1) {}

TextureManager::~TextureManager() {
    for (auto const& [tex_id, gl_id] : m_texture_id_to_gl_id) {
        glDeleteTextures(1, &gl_id);
    }
}

uint32_t TextureManager::load_texture(const std::string& path) {
    stbi_set_flip_vertically_on_load(true);

    int width, height, nr_channels;
    unsigned char *data = stbi_load(path.c_str(), &width, &height, &nr_channels, 0);

    if (!data) {
        std::cerr << "TextureManager::load_texture - Failed to load texture: " << path << std::endl;
        std::cerr << "stbi_failure_reason: " << stbi_failure_reason() << std::endl;
        return 0;
    }

    GLenum format;
    if (nr_channels == 1)
        format = GL_RED;
    else if (nr_channels == 3)
        format = GL_RGB;
    else if (nr_channels == 4)
        format = GL_RGBA;
    else {
        std::cerr << "TextureManager::load_texture - Unsupported number of channels: " << nr_channels << " in " << path << std::endl;
        stbi_image_free(data);
        return 0;
    }

    uint32_t gl_id;
    glGenTextures(1, &gl_id);
    glBindTexture(GL_TEXTURE_2D, gl_id);

    glTexImage2D(GL_TEXTURE_2D, 0, format, width, height, 0, format, GL_UNSIGNED_BYTE, data);
    glGenerateMipmap(GL_TEXTURE_2D);

    // Set texture wrapping and filtering options
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_REPEAT);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_REPEAT);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR_MIPMAP_LINEAR);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);

    stbi_image_free(data);

    uint32_t texture_id = m_next_texture_id++;
    m_texture_id_to_gl_id[texture_id] = gl_id;

    std::cout << "TextureManager: Loaded '" << path << "' with texture_id " << texture_id << " (gl_id " << gl_id << ")" << std::endl;

    return texture_id;
}

void TextureManager::bind_texture(uint32_t texture_id, uint32_t texture_unit) const {
    auto it = m_texture_id_to_gl_id.find(texture_id);
    if (it != m_texture_id_to_gl_id.end()) {
        glActiveTexture(texture_unit);
        glBindTexture(GL_TEXTURE_2D, it->second);
    } else {
        // Optionally bind a default texture (e.g., a white pixel)
        glActiveTexture(texture_unit);
        glBindTexture(GL_TEXTURE_2D, 0);
    }
}
