#pragma once

#include <string>
#include <vector>
#include <cstdint>
#include <unordered_map>

class ShaderManager {
public:
    ShaderManager();
    ~ShaderManager();

    // Loads a shader program from vertex and fragment shader files.
    // Returns a shader_id, or 0 if loading fails.
    uint32_t load_shader(const std::string& vertex_path, const std::string& fragment_path);

    // Uses the specified shader program.
    void use_shader(uint32_t shader_id) const;

    // Gets the OpenGL program ID from a shader_id.
    // Returns 0 if not found.
    uint32_t get_program_id(uint32_t shader_id) const;

private:
    std::string read_file(const char* file_path);
    uint32_t compile_shader(uint32_t type, const std::string& source);
    uint32_t create_program(uint32_t vertex_shader, uint32_t fragment_shader);

    uint32_t m_next_shader_id;
    std::unordered_map<uint32_t, uint32_t> m_shader_id_to_program_id;
};
