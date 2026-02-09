#pragma once

#include <string>
#include <unordered_map>
#include <ft2build.h>
#include FT_FREETYPE_H

// A simple struct for 2D integer vectors.
struct ivec2 {
    int x, y;
};

// A simple struct for 2D float vectors.
struct vec2 {
    float x, y;
};

// Holds all state information relevant to a character.
// This data is relative to the texture atlas.
struct Character {
    ivec2   Size;      // Size of glyph
    ivec2   Bearing;   // Offset from baseline to left/top of glyph
    unsigned int Advance;   // Horizontal offset to advance to next glyph
    vec2    TexCoordsStart; // Top-left texture coordinate in the atlas
    vec2    TexCoordsEnd;   // Bottom-right texture coordinate in the atlas
};

class FontManager {
public:
    FontManager();
    ~FontManager();

    // Loads a font, generates a texture atlas for the first 128 ASCII characters.
    // Returns true on success, false on failure.
    bool load_font(const std::string& path, unsigned int font_size);

    // Retrieves the character information for a given character.
    const Character& get_character(char c) const;

    // Gets the OpenGL ID of the texture atlas.
    unsigned int get_atlas_texture_id() const;

private:
    FT_Library m_ft;
    FT_Face m_face;
    std::unordered_map<char, Character> m_characters;
    unsigned int m_atlas_texture_id;

    void cleanup();
};
