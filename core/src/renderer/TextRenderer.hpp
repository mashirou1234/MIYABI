#pragma once

#include "renderer/ShaderManager.hpp"
#include "renderer/FontManager.hpp"
#include <string>
#include <glm/glm.hpp>

class TextRenderer {
public:
    TextRenderer(ShaderManager* shader_manager, FontManager* font_manager);
    ~TextRenderer();

    // Renders a string of text
    void render_text(const std::string& text, float x, float y, float scale, glm::vec3 color);

private:
    ShaderManager* m_shader_manager;
    FontManager* m_font_manager;
    uint32_t m_text_shader_id;
    unsigned int m_vao;
    unsigned int m_vbo;
};
