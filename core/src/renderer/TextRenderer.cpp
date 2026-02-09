#include "renderer/TextRenderer.hpp"
#include <iostream>
#include <glad/glad.h>
#include <glm/gtc/matrix_transform.hpp>

TextRenderer::TextRenderer(ShaderManager* shader_manager, FontManager* font_manager)
    : m_shader_manager(shader_manager), m_font_manager(font_manager), m_text_shader_id(0) {

    // Load shader
    m_text_shader_id = m_shader_manager->load_shader("core/src/shaders/text.vert", "core/src/shaders/text.frag");
    
    // Configure VAO/VBO for texture quads
    glGenVertexArrays(1, &m_vao);
    glGenBuffers(1, &m_vbo);
    
    glBindVertexArray(m_vao);
    glBindBuffer(GL_ARRAY_BUFFER, m_vbo);
    
    // The VBO will be filled with data for each character, 6 vertices per character.
    // We'll use glBufferData with GL_DYNAMIC_DRAW since it will be updated frequently.
    glBufferData(GL_ARRAY_BUFFER, sizeof(float) * 6 * 4, NULL, GL_DYNAMIC_DRAW);
    
    // Vertex attribute is a vec4 (pos.x, pos.y, tex.x, tex.y)
    glEnableVertexAttribArray(0);
    glVertexAttribPointer(0, 4, GL_FLOAT, GL_FALSE, 4 * sizeof(float), 0);
    
    glBindBuffer(GL_ARRAY_BUFFER, 0);
    glBindVertexArray(0);
}

TextRenderer::~TextRenderer() {
    glDeleteVertexArrays(1, &m_vao);
    glDeleteBuffers(1, &m_vbo);
}

void TextRenderer::render_text(const std::string& text, float x, float y, float scale, glm::vec3 color) {
    // Activate corresponding render state	
    m_shader_manager->use_shader(m_text_shader_id);
    uint32_t program_id = m_shader_manager->get_program_id(m_text_shader_id);
    if (program_id == 0) {
        std::cerr << "TextRenderer::render_text: Could not find shader program for text rendering" << std::endl;
        return;
    }

    // Set uniforms
    glUniform3f(glGetUniformLocation(program_id, "u_textColor"), color.x, color.y, color.z);
    
    // Assuming the window dimensions are known (e.g., 800x600).
    // This should ideally come from a window or context manager.
    glm::mat4 projection = glm::ortho(0.0f, 800.0f, 0.0f, 600.0f);
    glUniformMatrix4fv(glGetUniformLocation(program_id, "u_projection"), 1, GL_FALSE, &projection[0][0]);
    
    glActiveTexture(GL_TEXTURE0);
    glBindTexture(GL_TEXTURE_2D, m_font_manager->get_atlas_texture_id());
    glUniform1i(glGetUniformLocation(program_id, "u_text"), 0);

    glBindVertexArray(m_vao);

    // Iterate through all characters
    for (const char& c : text) {
        const Character& ch = m_font_manager->get_character(c);

        float xpos = x + ch.Bearing.x * scale;
        float ypos = y - (ch.Size.y - ch.Bearing.y) * scale;

        float w = ch.Size.x * scale;
        float h = ch.Size.y * scale;

        // Define the 2 triangles that form the character's quad
        float vertices[6][4] = {
            { xpos,     ypos + h,   ch.TexCoordsStart.x, ch.TexCoordsStart.y },
            { xpos,     ypos,       ch.TexCoordsStart.x, ch.TexCoordsEnd.y   },
            { xpos + w, ypos,       ch.TexCoordsEnd.x,   ch.TexCoordsEnd.y   },

            { xpos,     ypos + h,   ch.TexCoordsStart.x, ch.TexCoordsStart.y },
            { xpos + w, ypos,       ch.TexCoordsEnd.x,   ch.TexCoordsEnd.y   },
            { xpos + w, ypos + h,   ch.TexCoordsEnd.x,   ch.TexCoordsStart.y }
        };

        // Update content of VBO
        glBindBuffer(GL_ARRAY_BUFFER, m_vbo);
        glBufferSubData(GL_ARRAY_BUFFER, 0, sizeof(vertices), vertices); 
        glBindBuffer(GL_ARRAY_BUFFER, 0);

        // Render quad
        glDrawArrays(GL_TRIANGLES, 0, 6);

        // Now advance cursors for next glyph
        x += ch.Advance * scale;
    }

    glBindVertexArray(0);
    glBindTexture(GL_TEXTURE_2D, 0);
}

