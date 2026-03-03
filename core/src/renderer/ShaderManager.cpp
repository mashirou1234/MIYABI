#include "renderer/ShaderManager.hpp"
#include <glad/glad.h>
#include <iostream>
#include <fstream>
#include <sstream>
#include <iomanip>

namespace {
std::string summarize_gl_errors() {
    std::ostringstream oss;
    bool has_error = false;
    for (int i = 0; i < 8; ++i) {
        const unsigned int err = glGetError();
        if (err == GL_NO_ERROR) {
            break;
        }
        if (has_error) {
            oss << ",";
        }
        oss << "0x" << std::hex << std::uppercase << err;
        has_error = true;
    }
    return has_error ? oss.str() : "none";
}
}

ShaderManager::ShaderManager() : m_next_shader_id(1) {}

ShaderManager::~ShaderManager() {
    for (auto const& [shader_id, program_id] : m_shader_id_to_program_id) {
        glDeleteProgram(program_id);
    }
}

uint32_t ShaderManager::load_shader(const std::string& vertex_path, const std::string& fragment_path) {
    std::string vertex_source = read_file(vertex_path);
    std::string fragment_source = read_file(fragment_path);

    if (vertex_source.empty() || fragment_source.empty()) {
        std::cerr
            << "ERROR::SHADER::LOAD::READ_FAILED"
            << " vertex_path=\"" << vertex_path << "\""
            << " fragment_path=\"" << fragment_path << "\""
            << " gl_errors=" << summarize_gl_errors()
            << std::endl;
        return 0;
    }

    uint32_t vertex_shader = compile_shader(GL_VERTEX_SHADER, vertex_source, vertex_path);
    if (vertex_shader == 0) {
        return 0;
    }

    uint32_t fragment_shader = compile_shader(GL_FRAGMENT_SHADER, fragment_source, fragment_path);
    if (fragment_shader == 0) {
        glDeleteShader(vertex_shader);
        return 0;
    }

    uint32_t program_id = create_program(vertex_shader, fragment_shader, vertex_path, fragment_path);
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

std::string ShaderManager::read_file(const std::string& file_path) {
    std::ifstream file;
    std::stringstream stream;
    file.exceptions(std::ifstream::failbit | std::ifstream::badbit);
    try {
        file.open(file_path);
        stream << file.rdbuf();
        file.close();
    } catch (std::ifstream::failure& e) {
        std::cerr
            << "ERROR::SHADER::READ::FILE_OPEN_FAILED"
            << " path=\"" << file_path << "\""
            << " reason=\"" << e.what() << "\""
            << std::endl;
        return "";
    }
    return stream.str();
}

uint32_t ShaderManager::compile_shader(uint32_t type, const std::string& source, const std::string& source_path) {
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
        std::cerr
            << "ERROR::SHADER::COMPILE::FAILED"
            << " shader_type=" << shader_type_str
            << " path=\"" << source_path << "\""
            << " gl_errors=" << summarize_gl_errors()
            << "\n" << infoLog
            << std::endl;
        glDeleteShader(shader);
        return 0;
    }

    return shader;
}

uint32_t ShaderManager::create_program(
    uint32_t vertex_shader,
    uint32_t fragment_shader,
    const std::string& vertex_path,
    const std::string& fragment_path
) {
    uint32_t program = glCreateProgram();
    glAttachShader(program, vertex_shader);
    glAttachShader(program, fragment_shader);
    glLinkProgram(program);

    int success;
    char infoLog[512];
    glGetProgramiv(program, GL_LINK_STATUS, &success);
    if (!success) {
        glGetProgramInfoLog(program, 512, nullptr, infoLog);
        std::cerr
            << "ERROR::SHADER::LINK::FAILED"
            << " vertex_path=\"" << vertex_path << "\""
            << " fragment_path=\"" << fragment_path << "\""
            << " gl_errors=" << summarize_gl_errors()
            << "\n" << infoLog
            << std::endl;
        glDeleteProgram(program);
        program = 0;
    }

    glDeleteShader(vertex_shader);
    glDeleteShader(fragment_shader);

    return program;
}
