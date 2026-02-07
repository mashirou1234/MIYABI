#include <iostream>
#include <fstream>
#include <sstream>
#include <string>
#include <vector>
#include <dlfcn.h>
#include <thread>
#include <atomic>
#include <cstdio>
#include <algorithm> // For std::sort

#include <glad/glad.h>
#include <GLFW/glfw3.h>

#include "miyabi/miyabi.h"
#include "renderer/ShaderManager.hpp"
#include "renderer/MeshManager.hpp"
#include "renderer/MaterialManager.hpp"
#include "renderer/TextureManager.hpp"

// Temporary minimal Mat4 struct until a math library is added
struct Mat4 {
    float data[16] = {
        1.f, 0.f, 0.f, 0.f,
        0.f, 1.f, 0.f, 0.f,
        0.f, 0.f, 1.f, 0.f,
        0.f, 0.f, 0.f, 1.f
    };

    static Mat4 translation(float x, float y, float z) {
        Mat4 m;
        m.data[12] = x;
        m.data[13] = y;
        m.data[14] = z;
        return m;
    }
};

// --- Globals ---
MiyabiVTable g_vtable;
std::atomic<bool> g_reload_library(false);
const unsigned int SCR_WIDTH = 800;
const unsigned int SCR_HEIGHT = 600;

// --- Function Prototypes ---
void framebuffer_size_callback(GLFWwindow* window, int width, int height);
void processInput(GLFWwindow *window, InputState& input_state);
bool load_vtable(void* handle);


// --- File Watcher ---
void watch_for_changes() {
    std::cout << "Watcher thread started." << std::endl;
    FILE* pipe = popen("fswatch -1 -r -l 0.1 logic/src", "r");
    if (!pipe) {
        std::cerr << "popen() failed!" << std::endl;
        return;
    }
    char buffer[128];
    if (fgets(buffer, sizeof(buffer), pipe) != NULL) {
        g_reload_library = true;
    }
    pclose(pipe);
    std::cout << "Watcher thread finished." << std::endl;
}

// --- VTable Loader ---
bool load_vtable(void* handle) {
    if (!handle) {
        std::cerr << "Cannot open library: " << dlerror() << std::endl;
        return false;
    }
    using get_vtable_t = MiyabiVTable (*)();
    get_vtable_t get_vtable = (get_vtable_t) dlsym(handle, "get_miyabi_vtable");
    const char* dlsym_error = dlerror();
    if (dlsym_error) {
        std::cerr << "Cannot load symbol 'get_miyabi_vtable': " << dlsym_error << std::endl;
        dlclose(handle);
        return false;
    }
    g_vtable = get_vtable();
    return true;
}

int main() {
    std::thread watcher_thread(watch_for_changes);

    void* handle = dlopen("liblogic.dylib", RTLD_LAZY);
    if (!load_vtable(handle)) {
        return 1;
    }

    if (!glfwInit()) {
        std::cerr << "Failed to initialize GLFW" << std::endl;
        return -1;
    }
    glfwWindowHint(GLFW_CONTEXT_VERSION_MAJOR, 3);
    glfwWindowHint(GLFW_CONTEXT_VERSION_MINOR, 3);
    glfwWindowHint(GLFW_OPENGL_PROFILE, GLFW_OPENGL_CORE_PROFILE);
#ifdef __APPLE__
    glfwWindowHint(GLFW_OPENGL_FORWARD_COMPAT, GL_TRUE);
#endif

    GLFWwindow* window = glfwCreateWindow(SCR_WIDTH, SCR_HEIGHT, "MIYABI Engine", NULL, NULL);
    if (window == NULL) {
        std::cerr << "Failed to create GLFW window" << std::endl;
        glfwTerminate();
        return -1;
    }
    glfwMakeContextCurrent(window);
    glfwSetFramebufferSizeCallback(window, framebuffer_size_callback);

    if (!gladLoadGLLoader((GLADloadproc)glfwGetProcAddress)) {
        std::cerr << "Failed to initialize GLAD" << std::endl;
        return -1;
    }

    // --- Renderer Infrastructure Setup ---
    ShaderManager shader_manager;
    MeshManager mesh_manager;
    MaterialManager material_manager;
    TextureManager texture_manager;

    uint32_t textured_shader_id = shader_manager.load_shader("core/src/shaders/textured.vert", "core/src/shaders/textured.frag");
    if (textured_shader_id == 0) {
        glfwTerminate();
        return -1;
    }
    uint32_t quad_mesh_id = mesh_manager.create_quad_mesh();
    
    uint32_t textured_material_id = material_manager.create_material(textured_shader_id);

    const GLMesh* quad_mesh = mesh_manager.get_mesh(quad_mesh_id);
    if (!quad_mesh) {
        glfwTerminate();
        return -1;
    }

    // --- Instancing Setup ---
    unsigned int instance_vbo;
    glGenBuffers(1, &instance_vbo);
    glBindVertexArray(quad_mesh->vao);
    glBindBuffer(GL_ARRAY_BUFFER, instance_vbo);
    // Set up attribute pointers for instance model matrix (a_modelMatrix)
    // It's at location 2 because pos=0, texcoord=1. A mat4 is 4 vec4s.
    glEnableVertexAttribArray(2);
    glVertexAttribPointer(2, 4, GL_FLOAT, GL_FALSE, sizeof(Mat4), (void*)0);
    glEnableVertexAttribArray(3);
    glVertexAttribPointer(3, 4, GL_FLOAT, GL_FALSE, sizeof(Mat4), (void*)(sizeof(float) * 4));
    glEnableVertexAttribArray(4);
    glVertexAttribPointer(4, 4, GL_FLOAT, GL_FALSE, sizeof(Mat4), (void*)(sizeof(float) * 8));
    glEnableVertexAttribArray(5);
    glVertexAttribPointer(5, 4, GL_FLOAT, GL_FALSE, sizeof(Mat4), (void*)(sizeof(float) * 12));
    // Tell OpenGL this is an instanced vertex attribute.
    glVertexAttribDivisor(2, 1);
    glVertexAttribDivisor(3, 1);
    glVertexAttribDivisor(4, 1);
    glVertexAttribDivisor(5, 1);
    glBindVertexArray(0);

    World* world = g_vtable.create_world();

    // Process any initial asset load commands
    AssetCommandSlice asset_commands = g_vtable.get_asset_commands(world);
    for (const auto& command : asset_commands) {
        if (command.type_ == AssetCommandType::LoadTexture) {
            const char* c_path = g_vtable.get_asset_command_path_cstring(&command);
            std::string path(c_path);
            g_vtable.free_cstring((char*)c_path);

            uint32_t loaded_texture_id = texture_manager.load_texture(path);
            if (loaded_texture_id != 0) {
                g_vtable.notify_asset_loaded(world, command.request_id, loaded_texture_id);
            }
        }
    }
    g_vtable.clear_asset_commands(world);


    InputState input_state;

    // --- Render Loop ---
    while (!glfwWindowShouldClose(window)) {
        if (g_reload_library) {
            // ... hot reloading logic ...
        }

        processInput(window, input_state);
        g_vtable.update_input_state(world, input_state);
        g_vtable.run_logic_systems(world);

        // Process asset commands from Rust
        asset_commands = g_vtable.get_asset_commands(world);
        for (const auto& command : asset_commands) {
            if (command.type_ == AssetCommandType::LoadTexture) {
                const char* c_path = g_vtable.get_asset_command_path_cstring(&command);
                std::string path(c_path);
                g_vtable.free_cstring((char*)c_path);

                uint32_t loaded_texture_id = texture_manager.load_texture(path);
                 if (loaded_texture_id != 0) {
                    g_vtable.notify_asset_loaded(world, command.request_id, loaded_texture_id);
                }
            }
        }
        if (asset_commands.len > 0) {
            g_vtable.clear_asset_commands(world);
        }


        glClearColor(0.2f, 0.3f, 0.3f, 1.0f);
        glClear(GL_COLOR_BUFFER_BIT);

        // --- New Rendering Logic ---
        RenderableObjectSlice renderables_slice = g_vtable.get_renderables(world);
        std::vector<RenderableObject> renderables(renderables_slice.ptr, renderables_slice.ptr + renderables_slice.len);

        // Group renderables by texture
        std::unordered_map<uint32_t, std::vector<RenderableObject>> textured_batches;
        for (const auto& obj : renderables) {
            textured_batches[obj.texture_id].push_back(obj);
        }
        
        // For now, we only have one material and one mesh
        Material* material = material_manager.get_material(textured_material_id);
        shader_manager.use_shader(material->shader_id);
        uint32_t program_id = shader_manager.get_program_id(material->shader_id);

        // Set uniforms that are the same for all batches
        Mat4 projection, view; // Keep as identity for now
        glUniformMatrix4fv(glGetUniformLocation(program_id, "u_projection"), 1, GL_FALSE, projection.data);
        glUniformMatrix4fv(glGetUniformLocation(program_id, "u_view"), 1, GL_FALSE, view.data);
        glUniform1i(glGetUniformLocation(program_id, "u_texture"), 0); // Set texture sampler to unit 0

        mesh_manager.bind_mesh(quad_mesh_id);

        for (auto const& [texture_id, batch] : textured_batches) {
            if (batch.empty()) continue;

            std::vector<Mat4> model_matrices;
            model_matrices.reserve(batch.size());
            for (const auto& obj : batch) {
                model_matrices.push_back(Mat4::translation(obj.transform.position.x, obj.transform.position.y, obj.transform.position.z));
            }

            // Update instance VBO
            glBindBuffer(GL_ARRAY_BUFFER, instance_vbo);
            glBufferData(GL_ARRAY_BUFFER, batch.size() * sizeof(Mat4), model_matrices.data(), GL_DYNAMIC_DRAW);
            
            // Bind texture for this batch
            texture_manager.bind_texture(texture_id, GL_TEXTURE0);

            // Draw instanced
            glDrawElementsInstanced(GL_TRIANGLES, quad_mesh->element_count, GL_UNSIGNED_INT, 0, batch.size());
        }
        
        glBindVertexArray(0);

        glfwSwapBuffers(window);
        glfwPollEvents();
    }

    // --- Cleanup ---
    glDeleteBuffers(1, &instance_vbo);
    g_vtable.destroy_world(world);
    dlclose(handle);
    if(watcher_thread.joinable()) {
        watcher_thread.join();
    }
    glfwTerminate();
    return 0;
}

// --- Utility Functions ---
void processInput(GLFWwindow *window, InputState& input_state) {
    if (glfwGetKey(window, GLFW_KEY_ESCAPE) == GLFW_PRESS)
        glfwSetWindowShouldClose(window, true);

    input_state.up = glfwGetKey(window, GLFW_KEY_UP) == GLFW_PRESS;
    input_state.down = glfwGetKey(window, GLFW_KEY_DOWN) == GLFW_PRESS;
    input_state.left = glfwGetKey(window, GLFW_KEY_LEFT) == GLFW_PRESS;
    input_state.right = glfwGetKey(window, GLFW_KEY_RIGHT) == GLFW_PRESS;
}

void framebuffer_size_callback(GLFWwindow* window, int width, int height) {
    glViewport(0, 0, width, height);
}
