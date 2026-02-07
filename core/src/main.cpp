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
void processInput(GLFWwindow *window);
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

    uint32_t instanced_shader_id = shader_manager.load_shader("core/src/shaders/instanced.vert", "core/src/shaders/instanced.frag");
    if (instanced_shader_id == 0) {
        glfwTerminate();
        return -1;
    }
    uint32_t triangle_mesh_id = mesh_manager.create_triangle_mesh();
    uint32_t basic_material_id = material_manager.create_material(instanced_shader_id);

    const GLMesh* triangle_mesh = mesh_manager.get_mesh(triangle_mesh_id);
    if (!triangle_mesh) {
        glfwTerminate();
        return -1;
    }

    // --- Instancing Setup ---
    unsigned int instance_vbo;
    glGenBuffers(1, &instance_vbo);
    glBindVertexArray(triangle_mesh->vao);
    glBindBuffer(GL_ARRAY_BUFFER, instance_vbo);
    // Set up attribute pointers for instance model matrix
    // A mat4 is 4 vec4s.
    glEnableVertexAttribArray(1);
    glVertexAttribPointer(1, 4, GL_FLOAT, GL_FALSE, sizeof(Mat4), (void*)0);
    glEnableVertexAttribArray(2);
    glVertexAttribPointer(2, 4, GL_FLOAT, GL_FALSE, sizeof(Mat4), (void*)(sizeof(float) * 4));
    glEnableVertexAttribArray(3);
    glVertexAttribPointer(3, 4, GL_FLOAT, GL_FALSE, sizeof(Mat4), (void*)(sizeof(float) * 8));
    glEnableVertexAttribArray(4);
    glVertexAttribPointer(4, 4, GL_FLOAT, GL_FALSE, sizeof(Mat4), (void*)(sizeof(float) * 12));
    // Tell OpenGL this is an instanced vertex attribute.
    glVertexAttribDivisor(1, 1);
    glVertexAttribDivisor(2, 1);
    glVertexAttribDivisor(3, 1);
    glVertexAttribDivisor(4, 1);
    glBindVertexArray(0);

    World* world = g_vtable.create_world();

    // --- Render Loop ---
    while (!glfwWindowShouldClose(window)) {
        if (g_reload_library) {
            // ... hot reloading logic ...
        }

        processInput(window);
        g_vtable.run_logic_systems(world);

        glClearColor(0.2f, 0.3f, 0.3f, 1.0f);
        glClear(GL_COLOR_BUFFER_BIT);

        // --- New Rendering Logic ---
        RenderableObjectSlice renderables = g_vtable.get_renderables(world);
        
        // For now, we only have one material and one mesh, so we can treat everything
        // as a single batch.
        if (renderables.len > 0) {
            std::vector<Mat4> model_matrices;
            model_matrices.reserve(renderables.len);
            for (const auto& obj : renderables) {
                // NOTE: A full transform would include rotation and scale.
                // For now, only translation is implemented.
                model_matrices.push_back(Mat4::translation(obj.transform.position.x, obj.transform.position.y, obj.transform.position.z));
            }

            // Update instance VBO
            glBindBuffer(GL_ARRAY_BUFFER, instance_vbo);
            glBufferData(GL_ARRAY_BUFFER, renderables.len * sizeof(Mat4), model_matrices.data(), GL_DYNAMIC_DRAW);
            
            // Get material and shader info
            const Material* material = material_manager.get_material(basic_material_id); // Assuming all objects use this
            shader_manager.use_shader(material->shader_id);
            uint32_t program_id = shader_manager.get_program_id(material->shader_id);

            // Set uniforms (identity matrices for now)
            Mat4 projection, view;
            glUniformMatrix4fv(glGetUniformLocation(program_id, "u_projection"), 1, GL_FALSE, projection.data);
            glUniformMatrix4fv(glGetUniformLocation(program_id, "u_view"), 1, GL_FALSE, view.data);

            // Bind mesh and draw instanced
            mesh_manager.bind_mesh(triangle_mesh_id);
            glDrawElementsInstanced(GL_TRIANGLES, triangle_mesh->element_count, GL_UNSIGNED_INT, 0, renderables.len);
            glBindVertexArray(0);
        }

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
void processInput(GLFWwindow *window) {
    if (glfwGetKey(window, GLFW_KEY_ESCAPE) == GLFW_PRESS)
        glfwSetWindowShouldClose(window, true);
}

void framebuffer_size_callback(GLFWwindow* window, int width, int height) {
    glViewport(0, 0, width, height);
}
