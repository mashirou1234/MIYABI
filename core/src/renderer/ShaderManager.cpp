#include "renderer/ShaderManager.hpp"
#include <glad/glad.h>
#include <iostream>
#include <fstream>
#include <sstream>

ShaderManager::ShaderManager() : m_next_shader_id(1) {}

ShaderManager::~ShaderManager() {
    for (auto const& [shader_id, program_id] : m_shader_id_to_program_id) {
        glDeleteProgram(program_id);
    }
}

uint32_t ShaderManager::load_shader(const std::string& vertex_path, const std::string& fragment_path) {
    std::string vertex_source = read_file(vertex_path.c_str());
    std::string fragment_source = read_file(fragment_path.c_str());

    if (vertex_source.empty() || fragment_source.empty()) {
        return 0;
    }

    uint32_t vertex_shader = compile_shader(GL_VERTEX_SHADER, vertex_source);
    if (vertex_shader == 0) {
        return 0;
    }

    uint32_t fragment_shader = compile_shader(GL_FRAGMENT_SHADER, fragment_source);
    if (fragment_shader == 0) {
        glDeleteShader(vertex_shader);
        return 0;
    }

    uint32_t program_id = create_program(vertex_shader, fragment_shader);
    if (program_id == 0) {
        return 0;
    }

    uint32_t shader_id = m_next_shader_id++;
    m_shader_id_to_program_id[shader_id] = program_id;

    return shader_id;
}

void ShaderManager::use_shader(uint32_t shader_id) const {
    auto it = m_shader_id_to_program_id.find(shader_id);
    if (it != m_shader_id_to_program_id.end()) {
        glUseProgram(it->second);
    } else {
        std::cerr << "ShaderManager::use_shader - Shader ID " << shader_id << " not found." << std::endl;
        glUseProgram(0);
    }
}

uint32_t ShaderManager::get_program_id(uint32_t shader_id) const {
    auto it = m_shader_id_to_program_id.find(shader_id);
    if (it != m_shader_id_to_program_id.end()) {
        return it->second;
    }
    return 0;
}

std::string ShaderManager::read_file(const char* file_path) {
    std::ifstream file;
    std::stringstream stream;
    file.exceptions(std::ifstream::failbit | std::ifstream::badbit);
    try {
        file.open(file_path);
        stream << file.rdbuf();
        file.close();
    } catch (std::ifstream::failure& e) {
        std::cerr << "ERROR::SHADER::FILE_NOT_SUCCESSFULLY_READ: " << file_path << std::endl;
        return "";
    }
    return stream.str();
}

uint32_t ShaderManager::compile_shader(uint32_t type, const std::string& source) {
    uint32_t shader = glCreateShader(type);
    const char* src = source.c_str();
    glShaderSource(shader, 1, &src, nullptr);
    glCompileShader(shader);

    int success;
    char infoLog[512];
    glGetShaderiv(shader, GL_COMPILE_STATUS, &success);
    if (!success) {
        glGetShaderInfoLog(shader, 512, nullptr, infoLog);
        const char* shader_type_str = (type == GL_VERTEX_SHADER) ? "VERTEX" : "FRAGMENT";
        std::cerr << "ERROR::SHADER::" << shader_type_str << "::COMPILATION_FAILED\n" << infoLog << std::endl;
        glDeleteShader(shader);
        return 0;
    }

    return shader;
}

uint32_t ShaderManager::create_program(uint32_t vertex_shader, uint32_t fragment_shader) {
    uint32_t program = glCreateProgram();
    glAttachShader(program, vertex_shader);
    glAttachShader(program, fragment_shader);
    glLinkProgram(program);

    int success;
    char infoLog[512];
    glGetProgramiv(program, GL_LINK_STATUS, &success);
    if (!success) {
        glGetProgramInfoLog(program, 512, nullptr, infoLog);
        std::cerr << "ERROR::SHADER::PROGRAM::LINKING_FAILED\n" << infoLog << std::endl;
        glDeleteProgram(program);
        program = 0;
    }

    glDeleteShader(vertex_shader);
    glDeleteShader(fragment_shader);

    return program;
}
