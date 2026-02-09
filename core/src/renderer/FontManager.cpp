#include "renderer/FontManager.hpp"
#include <iostream>
#include <glad/glad.h>
#include <numeric>

FontManager::FontManager() : m_ft(nullptr), m_face(nullptr), m_atlas_texture_id(0) {
    if (FT_Init_FreeType(&m_ft)) {
        std::cerr << "ERROR::FREETYPE: Could not init FreeType Library" << std::endl;
    }
}

FontManager::~FontManager() {
    cleanup();
}

void FontManager::cleanup() {
    if (m_atlas_texture_id != 0) {
        glDeleteTextures(1, &m_atlas_texture_id);
        m_atlas_texture_id = 0;
    }
    if (m_face) {
        FT_Done_Face(m_face);
        m_face = nullptr;
    }
    if (m_ft) {
        FT_Done_FreeType(m_ft);
        m_ft = nullptr;
    }
}

bool FontManager::load_font(const std::string& path, unsigned int font_size) {
    if (!m_ft) {
        std::cerr << "ERROR::FREETYPE: Library not initialized." << std::endl;
        return false;
    }
    if (FT_New_Face(m_ft, path.c_str(), 0, &m_face)) {
        std::cerr << "ERROR::FREETYPE: Failed to load font: " << path << std::endl;
        return false;
    }

    FT_Set_Pixel_Sizes(m_face, 0, font_size);
    glPixelStorei(GL_UNPACK_ALIGNMENT, 1);

    unsigned int atlas_width = 0;
    unsigned int atlas_height = 0;

    // First pass: calculate atlas dimensions
    for (unsigned char c = 0; c < 128; c++) {
        if (FT_Load_Char(m_face, c, FT_LOAD_RENDER)) {
            std::cerr << "Warning::FREETYPE: Failed to load Glyph for character: " << c << std::endl;
            continue;
        }
        atlas_width += m_face->glyph->bitmap.width;
        atlas_height = std::max(atlas_height, m_face->glyph->bitmap.rows);
    }

    // Create the texture atlas
    glGenTextures(1, &m_atlas_texture_id);
    glBindTexture(GL_TEXTURE_2D, m_atlas_texture_id);
    glTexImage2D(GL_TEXTURE_2D, 0, GL_RED, atlas_width, atlas_height, 0, GL_RED, GL_UNSIGNED_BYTE, nullptr);

    // Set texture options
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);

    // Second pass: fill the texture atlas
    int x_offset = 0;
    for (unsigned char c = 0; c < 128; c++) {
        if (FT_Load_Char(m_face, c, FT_LOAD_RENDER)) {
            continue; // Warning already issued in first pass
        }

        if (m_face->glyph->bitmap.width > 0 && m_face->glyph->bitmap.rows > 0) {
            glTexSubImage2D(GL_TEXTURE_2D, 0, x_offset, 0, m_face->glyph->bitmap.width, m_face->glyph->bitmap.rows, GL_RED, GL_UNSIGNED_BYTE, m_face->glyph->bitmap.buffer);
        }

        // Calculate texture coordinates
        vec2 tex_coords_start = {
            (float)x_offset / (float)atlas_width,
            0.0f
        };
        vec2 tex_coords_end = {
            (float)(x_offset + m_face->glyph->bitmap.width) / (float)atlas_width,
            (float)m_face->glyph->bitmap.rows / (float)atlas_height
        };

        // Store character
        Character character = {
            { (int)m_face->glyph->bitmap.width, (int)m_face->glyph->bitmap.rows },
            { static_cast<int>(m_face->glyph->metrics.horiBearingX / 64), static_cast<int>(m_face->glyph->metrics.horiBearingY / 64) },
            (unsigned int)(m_face->glyph->advance.x / 64),
            tex_coords_start,
            tex_coords_end
        };
        m_characters.insert(std::pair<char, Character>(c, character));

        x_offset += m_face->glyph->bitmap.width;
    }

    glBindTexture(GL_TEXTURE_2D, 0);
    // FT_Done_Face(m_face); // Keep face loaded for now

    return true;
}

const Character& FontManager::get_character(char c) const {
    return m_characters.at(c);
}

unsigned int FontManager::get_atlas_texture_id() const {
    return m_atlas_texture_id;
}