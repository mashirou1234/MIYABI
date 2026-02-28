#include <iostream>
#include <fstream>
#include <sstream>
#include <string>
#include <vector>
#include <thread>
#include <atomic>
#include <cstdio>
#include <algorithm> // For std::sort

#include <glad/glad.h>
#include <GLFW/glfw3.h>

#include "miyabi/miyabi.h"
#include "miyabi/bridge.h"
#include "renderer/ShaderManager.hpp"
#include "renderer/MeshManager.hpp"
#include "renderer/MaterialManager.hpp"
#include "renderer/TextureManager.hpp"
#include "renderer/FontManager.hpp"
#include "renderer/TextRenderer.hpp"
#include <glm/glm.hpp>
#include <glm/gtc/matrix_transform.hpp>
#include "profiler/Profiler.hpp"

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

// Enum to mirror Rust's GameState
enum GameState {
    MainMenu,
    InGame,
};

// --- Globals ---
MiyabiVTable g_vtable;
bool g_mouse_released = true;
const unsigned int SCR_WIDTH = 800;
const unsigned int SCR_HEIGHT = 600;

// --- Function Prototypes ---
void framebuffer_size_callback(GLFWwindow* window, int width, int height);
void processInput(GLFWwindow *window, InputState& input_state);
void apply_fullscreen_mode(
    GLFWwindow* window,
    bool enable,
    bool& is_fullscreen,
    int& windowed_x,
    int& windowed_y,
    int& windowed_width,
    int& windowed_height
);

// VTable is now linked statically, we just need to get it.
extern "C" MiyabiVTable get_miyabi_vtable();

int main() {
    g_vtable = get_miyabi_vtable();
    if (g_vtable.abi_version != MIYABI_ABI_VERSION) {
        std::cerr
            << "ABI version mismatch. expected="
            << MIYABI_ABI_VERSION
            << " actual="
            << g_vtable.abi_version
            << std::endl;
        return -1;
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

    // --- Engine Systems Setup (Audio, Physics, etc.) ---
    init_engine_systems();

    // Enable alpha blending
    glEnable(GL_BLEND);
    glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);

    // --- Renderer Infrastructure Setup ---
    ShaderManager shader_manager;
    MeshManager mesh_manager;
    MaterialManager material_manager;
    TextureManager texture_manager;
    FontManager font_manager;
    font_manager.load_font("assets/MPLUS1p-Regular.ttf", 48);
    TextRenderer text_renderer(&shader_manager, &font_manager);

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

    Game* miyabi_game = g_vtable.create_game();
    bool is_fullscreen = false;
    int windowed_x = 100;
    int windowed_y = 100;
    int windowed_width = SCR_WIDTH;
    int windowed_height = SCR_HEIGHT;
    glfwGetWindowPos(window, &windowed_x, &windowed_y);
    glfwGetWindowSize(window, &windowed_width, &windowed_height);

    if (has_pending_fullscreen_request()) {
        bool requested_fullscreen = consume_pending_fullscreen_request();
        apply_fullscreen_mode(
            window,
            requested_fullscreen,
            is_fullscreen,
            windowed_x,
            windowed_y,
            windowed_width,
            windowed_height
        );
    }

    // Process any initial asset load commands
    AssetCommandSlice asset_commands = g_vtable.get_asset_commands(miyabi_game);
    for (const auto& command : asset_commands) {
        const char* c_path = g_vtable.get_asset_command_path_cstring(&command);
        std::string path(c_path);
        g_vtable.free_cstring((char*)c_path);

        uint32_t loaded_texture_id = 0;
        switch (command.type_) {
            case AssetCommandType::LoadTexture:
                loaded_texture_id = texture_manager.load_texture(path);
                break;
            case AssetCommandType::ReloadTexture:
                loaded_texture_id = texture_manager.reload_texture(path);
                break;
            default:
                std::cerr
                    << "Warning: Unknown AssetCommandType received in initial asset processing. "
                    << "request_id=" << command.request_id
                    << ", type=" << static_cast<int>(command.type_)
                    << ", path=" << path
                    << std::endl;
                break;
        }
        g_vtable.notify_asset_loaded(miyabi_game, command.request_id, loaded_texture_id);
    }
    g_vtable.clear_asset_commands(miyabi_game);

    InputState input_state;

#ifdef MIYABI_PROFILE
    // Variables for performance monitoring
    double lastTime = glfwGetTime();
    int nbFrames = 0;
#endif

    // --- Render Loop ---
    while (!glfwWindowShouldClose(window)) {
        MIYABI_PROFILE_SCOPE("Frame");
#ifdef MIYABI_PROFILE
        // Measure time
        double currentTime = glfwGetTime();
        nbFrames++;
        if (currentTime - lastTime >= 1.0) { // If last print was more than 1 sec ago
            double msPerFrame = 1000.0 / double(nbFrames);
            std::string title = "MIYABI Engine - " + std::to_string(nbFrames) + " FPS (" + std::to_string(msPerFrame) + " ms/frame)";
            glfwSetWindowTitle(window, title.c_str());

            nbFrames = 0;
            lastTime += 1.0;
        }
#endif
        {
            MIYABI_PROFILE_SCOPE("PhysicsStep");
            step_engine_systems();
        }

        {
            MIYABI_PROFILE_SCOPE("InputProcessing");
            processInput(window, input_state);
            g_vtable.update_input_state(miyabi_game, input_state);
        }
        
        {
            MIYABI_PROFILE_SCOPE("RustLogicUpdate");
            g_vtable.update_game(miyabi_game);
        }

        if (has_pending_fullscreen_request()) {
            bool requested_fullscreen = consume_pending_fullscreen_request();
            apply_fullscreen_mode(
                window,
                requested_fullscreen,
                is_fullscreen,
                windowed_x,
                windowed_y,
                windowed_width,
                windowed_height
            );
        }

        {
            MIYABI_PROFILE_SCOPE("AssetProcessing");
            // Process asset commands from Rust
            asset_commands = g_vtable.get_asset_commands(miyabi_game);
            for (const auto& command : asset_commands) {
                const char* c_path = g_vtable.get_asset_command_path_cstring(&command);
                std::string path(c_path);
                g_vtable.free_cstring((char*)c_path);

                uint32_t loaded_texture_id = 0;
                switch (command.type_) {
                    case AssetCommandType::LoadTexture:
                        loaded_texture_id = texture_manager.load_texture(path);
                        break;
                    case AssetCommandType::ReloadTexture:
                        loaded_texture_id = texture_manager.reload_texture(path);
                        break;
                    default:
                        std::cerr
                            << "Warning: Unknown AssetCommandType received during frame asset processing. "
                            << "request_id=" << command.request_id
                            << ", type=" << static_cast<int>(command.type_)
                            << ", path=" << path
                            << std::endl;
                        break;
                }
                g_vtable.notify_asset_loaded(miyabi_game, command.request_id, loaded_texture_id);
            }
            if (asset_commands.len > 0) {
                g_vtable.clear_asset_commands(miyabi_game);
            }
        }

        {
            MIYABI_PROFILE_SCOPE("Render");
            glClearColor(0.2f, 0.3f, 0.3f, 1.0f);
            glClear(GL_COLOR_BUFFER_BIT);

            // --- New Rendering Logic ---
            RenderableObjectSlice renderables_slice = g_vtable.get_renderables(miyabi_game);
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
            glm::mat4 projection = glm::ortho(0.0f, (float)SCR_WIDTH, 0.0f, (float)SCR_HEIGHT, -1.0f, 1.0f);
            glm::mat4 view = glm::mat4(1.0f);
            glUniformMatrix4fv(glGetUniformLocation(program_id, "u_projection"), 1, GL_FALSE, &projection[0][0]);
            glUniformMatrix4fv(glGetUniformLocation(program_id, "u_view"), 1, GL_FALSE, &view[0][0]);
            glUniform1i(glGetUniformLocation(program_id, "u_texture"), 0); // Set texture sampler to unit 0

            mesh_manager.bind_mesh(quad_mesh_id);

            for (auto const& [texture_id, batch] : textured_batches) {
                if (batch.empty()) continue;

                std::vector<glm::mat4> model_matrices;
                model_matrices.reserve(batch.size());
                for (const auto& obj : batch) {
                    glm::mat4 model = glm::mat4(1.0f);
                    model = glm::translate(model, glm::vec3(obj.transform.position.x, obj.transform.position.y, obj.transform.position.z));
                    // Add rotation and scale later
                    model = glm::scale(model, glm::vec3(obj.transform.scale.x, obj.transform.scale.y, obj.transform.scale.z));
                    model_matrices.push_back(model);
                }

                // Update instance VBO
                glBindBuffer(GL_ARRAY_BUFFER, instance_vbo);
                glBufferData(GL_ARRAY_BUFFER, batch.size() * sizeof(glm::mat4), model_matrices.data(), GL_DYNAMIC_DRAW);
                
                // Bind texture for this batch
                texture_manager.bind_texture(texture_id, GL_TEXTURE0);

                // Draw instanced
                glDrawElementsInstanced(GL_TRIANGLES, quad_mesh->element_count, GL_UNSIGNED_INT, 0, batch.size());
            }
            
            glBindVertexArray(0);

            // Render text from commands
            TextCommandSlice text_commands_slice = g_vtable.get_text_commands(miyabi_game);
            for (const auto& command : text_commands_slice) {
                const char* c_text = g_vtable.get_text_command_text_cstring(&command);
                std::string text(c_text);
                g_vtable.free_cstring((char*)c_text);

                float scale = command.font_size / 48.0f; // Font atlas was loaded with size 48

                text_renderer.render_text(
                    text,
                    command.position.x,
                    command.position.y,
                    scale,
                    glm::vec3(command.color.x, command.color.y, command.color.z)
                );
            }
        }

        glfwSwapBuffers(window);
        glfwPollEvents();
    }

    // --- Cleanup ---
    glDeleteBuffers(1, &instance_vbo);
    g_vtable.destroy_game(miyabi_game);
    shutdown_engine_systems();

    glfwTerminate();
    return 0;
}

// --- Utility Functions ---
void processInput(GLFWwindow *window, InputState& input_state) {
    input_state.up = glfwGetKey(window, GLFW_KEY_UP) == GLFW_PRESS;
    input_state.down = glfwGetKey(window, GLFW_KEY_DOWN) == GLFW_PRESS;
    input_state.left = glfwGetKey(window, GLFW_KEY_LEFT) == GLFW_PRESS;
    input_state.right = glfwGetKey(window, GLFW_KEY_RIGHT) == GLFW_PRESS;
    input_state.esc_key = glfwGetKey(window, GLFW_KEY_ESCAPE) == GLFW_PRESS;

    input_state.s_key = glfwGetKey(window, GLFW_KEY_S) == GLFW_PRESS;
    input_state.p_key = glfwGetKey(window, GLFW_KEY_P) == GLFW_PRESS;
    input_state.u_key = glfwGetKey(window, GLFW_KEY_U) == GLFW_PRESS;

    // Mouse position
    double xpos, ypos;
    glfwGetCursorPos(window, &xpos, &ypos);
    input_state.mouse_pos.x = (float)xpos;
    input_state.mouse_pos.y = (float)ypos;

    // Mouse click (handle state to register only on click, not hold)
    if (glfwGetMouseButton(window, GLFW_MOUSE_BUTTON_LEFT) == GLFW_PRESS) {
        input_state.mouse_clicked = false;
        g_mouse_released = false;
    } else if (glfwGetMouseButton(window, GLFW_MOUSE_BUTTON_LEFT) == GLFW_RELEASE) {
        if (!g_mouse_released) {
            input_state.mouse_clicked = true;
            g_mouse_released = true;
        } else {
            input_state.mouse_clicked = false;
        }
    } else {
        input_state.mouse_clicked = false;
    }
}

void framebuffer_size_callback(GLFWwindow* window, int width, int height) {
    glViewport(0, 0, width, height);
}

void apply_fullscreen_mode(
    GLFWwindow* window,
    bool enable,
    bool& is_fullscreen,
    int& windowed_x,
    int& windowed_y,
    int& windowed_width,
    int& windowed_height
) {
    if (enable == is_fullscreen) {
        return;
    }

    if (enable) {
        glfwGetWindowPos(window, &windowed_x, &windowed_y);
        glfwGetWindowSize(window, &windowed_width, &windowed_height);

        GLFWmonitor* monitor = glfwGetPrimaryMonitor();
        if (monitor == NULL) {
            std::cerr << "Failed to get primary monitor for fullscreen." << std::endl;
            return;
        }

        const GLFWvidmode* mode = glfwGetVideoMode(monitor);
        if (mode == NULL) {
            std::cerr << "Failed to get video mode for fullscreen." << std::endl;
            return;
        }

        glfwSetWindowMonitor(
            window,
            monitor,
            0,
            0,
            mode->width,
            mode->height,
            mode->refreshRate
        );
        is_fullscreen = true;
    } else {
        glfwSetWindowMonitor(
            window,
            NULL,
            windowed_x,
            windowed_y,
            windowed_width,
            windowed_height,
            0
        );
        is_fullscreen = false;
    }

    int fb_width = 0;
    int fb_height = 0;
    glfwGetFramebufferSize(window, &fb_width, &fb_height);
    glViewport(0, 0, fb_width, fb_height);
}
