#include <iostream>
#include <fstream>
#include <sstream>
#include <string>
#include <vector>
#include <dlfcn.h>
#include <thread>
#include <atomic>

#include <glad/glad.h>
#include <GLFW/glfw3.h>

#include "miyabi_cxxbridge/lib.h" // cxxが生成したヘッダー

// Define the struct to hold render commands from Rust
struct RenderCommands {
    const DrawTriangleCommand* commands;
    size_t count;
};

struct SerializedWorld {
    const uint8_t* data;
    size_t len;
};

// Function prototypes
void framebuffer_size_callback(GLFWwindow* window, int width, int height);
void processInput(GLFWwindow *window);
unsigned int create_shader_program(const char* vertex_path, const char* fragment_path);
std::string read_shader_file(const char* file_path);

// Define function pointers for Rust functions
using create_world_t = World* (*)();
using destroy_world_t = void (*)(World*);
using run_logic_t = void (*)(World*);
using build_render_commands_t = RenderCommands (*)(World*);
using serialize_world_t = SerializedWorld (*)(const World*);
using deserialize_world_t = World* (*)(const uint8_t*, size_t);
using free_serialized_world_t = void (*)(SerializedWorld);


// Global flag to signal library reload
std::atomic<bool> g_reload_library(false);

void watch_for_changes() {
    // Run fswatch and wait for it to exit (which it does after the first change)
    system("fswatch -r -1 logic");
    g_reload_library = true;
}

// Settings
const unsigned int SCR_WIDTH = 800;
const unsigned int SCR_HEIGHT = 600;

int main() {
    // Start the file watcher thread
    std::thread watcher_thread(watch_for_changes);

    // Load the Rust library
    void* handle = dlopen("liblogic.dylib", RTLD_LAZY);
    if (!handle) {
        std::cerr << "Cannot open library: " << dlerror() << std::endl;
        return 1;
    }

    // Load the symbols
    create_world_t create_world = (create_world_t) dlsym(handle, "create_world");
    destroy_world_t destroy_world = (destroy_world_t) dlsym(handle, "destroy_world");
    run_logic_t run_logic = (run_logic_t) dlsym(handle, "run_logic");
    build_render_commands_t build_render_commands = (build_render_commands_t) dlsym(handle, "build_render_commands");
    serialize_world_t serialize_world = (serialize_world_t) dlsym(handle, "serialize_world");
    deserialize_world_t deserialize_world = (deserialize_world_t) dlsym(handle, "deserialize_world");
    free_serialized_world_t free_serialized_world = (free_serialized_world_t) dlsym(handle, "free_serialized_world");

    const char* dlsym_error = dlerror();
    if (dlsym_error) {
        std::cerr << "Cannot load symbol: " << dlsym_error << std::endl;
        dlclose(handle);
        return 1;
    }

    // glfw: initialize and configure
    // ------------------------------
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

    // glfw window creation
    // --------------------
    GLFWwindow* window = glfwCreateWindow(SCR_WIDTH, SCR_HEIGHT, "MIYABI", NULL, NULL);
    if (window == NULL) {
        std::cerr << "Failed to create GLFW window" << std::endl;
        glfwTerminate();
        return -1;
    }
    glfwMakeContextCurrent(window);
    glfwSetFramebufferSizeCallback(window, framebuffer_size_callback);

    // glad: load all OpenGL function pointers
    // ---------------------------------------
    if (!gladLoadGLLoader((GLADloadproc)glfwGetProcAddress)) {
        std::cerr << "Failed to initialize GLAD" << std::endl;
        return -1;
    }

    // build and compile our shader program
    // ------------------------------------
    unsigned int shaderProgram = create_shader_program("core/src/shaders/triangle.vert", "core/src/shaders/triangle.frag");
    if (shaderProgram == 0) {
        std::cerr << "Failed to create shader program. Exiting." << std::endl;
        glfwTerminate();
        return -1;
    }

    // set up vertex data (and buffer(s)) and configure vertex attributes
    // ------------------------------------------------------------------
    float vertices[] = {
        -0.5f, -0.5f, 0.0f, // left
         0.5f, -0.5f, 0.0f, // right
         0.0f,  0.5f, 0.0f  // top
    };

    unsigned int VBO, VAO;
    glGenVertexArrays(1, &VAO);
    glGenBuffers(1, &VBO);
    // bind the Vertex Array Object first, then bind and set vertex buffer(s), and then configure vertex attributes(s).
    glBindVertexArray(VAO);

    glBindBuffer(GL_ARRAY_BUFFER, VBO);
    glBufferData(GL_ARRAY_BUFFER, sizeof(vertices), vertices, GL_STATIC_DRAW);

    glVertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE, 3 * sizeof(float), (void*)0);
    glEnableVertexAttribArray(0);

    // note that this is allowed, the call to glVertexAttribPointer registered VBO as the vertex attribute's bound vertex buffer object so afterwards we can safely unbind
    glBindBuffer(GL_ARRAY_BUFFER, 0);

    // You can unbind the VAO afterwards so other VAO calls won't accidentally modify this VAO, but this rarely happens. Modifying other
    // VAOs requires a call to glBindVertexArray anyways so we generally don't unbind VAOs (nor VBOs) when it's not directly necessary.
    glBindVertexArray(0);


    // uniform location
    // ----------------
    GLint translationLoc = glGetUniformLocation(shaderProgram, "u_translation");

    // RustからWorldオブジェクトの所有権付きポインタを受け取る
    // ------------------------------------------------
    auto world = create_world();

    // render loop
    // -----------
    while (!glfwWindowShouldClose(window)) {
        // Check for library reload
        if (g_reload_library) {
            g_reload_library = false;
            std::cout << "Reloading library..." << std::endl;

            // Serialize the world
            std::cout << "Serializing world..." << std::endl;
            SerializedWorld serialized_world = serialize_world(world);
            std::cout << "World serialized." << std::endl;
            
            // Rebuild the library
            std::cout << "Rebuilding library..." << std::endl;
            system("cmake --build build");
            std::cout << "Library rebuilt." << std::endl;

            // Unload the old library
            std::cout << "Unloading old library..." << std::endl;
            dlclose(handle);
            std::cout << "Old library unloaded." << std::endl;

            // Load the new library
            std::cout << "Loading new library..." << std::endl;
            handle = dlopen("liblogic.dylib", RTLD_LAZY);
            if (!handle) {
                std::cerr << "Cannot open library: " << dlerror() << std::endl;
            } else {
                std::cout << "New library loaded." << std::endl;
                // Load the new symbols
                std::cout << "Loading new symbols..." << std::endl;
                create_world = (create_world_t) dlsym(handle, "create_world");
                destroy_world = (destroy_world_t) dlsym(handle, "destroy_world");
                run_logic = (run_logic_t) dlsym(handle, "run_logic");
                build_render_commands = (build_render_commands_t) dlsym(handle, "build_render_commands");
                serialize_world = (serialize_world_t) dlsym(handle, "serialize_world");
                deserialize_world = (deserialize_world_t) dlsym(handle, "deserialize_world");
                free_serialized_world = (free_serialized_world_t) dlsym(handle, "free_serialized_world");
                dlsym_error = dlerror();
                if (dlsym_error) {
                    std::cerr << "Cannot load symbol: " << dlsym_error << std::endl;
                    dlclose(handle);
                }
                std::cout << "New symbols loaded." << std::endl;
            }

            // Recreate the world
            std::cout << "Recreating world..." << std::endl;
            destroy_world(world);
            world = deserialize_world(serialized_world.data, serialized_world.len);
            free_serialized_world(serialized_world);
            std::cout << "World recreated." << std::endl;


            // Restart the file watcher
            std::cout << "Restarting file watcher..." << std::endl;
            watcher_thread.join();
            watcher_thread = std::thread(watch_for_changes);
            std::cout << "File watcher restarted." << std::endl;
        }

        // input
        // -----
        processInput(window);

        // Rustのフレーム毎ロジックを呼び出し、シーンの状態を更新
        // ----------------------------------------------------
        run_logic(world);

        // render
        // ------
        glClearColor(0.2f, 0.3f, 0.3f, 1.0f);
        glClear(GL_COLOR_BUFFER_BIT);

        // シェーダーとVAOを有効化
        glUseProgram(shaderProgram);
        glBindVertexArray(VAO);

        // Rustからコマンドバッファを取得し、描画コマンドを実行
        // ---------------------------------------------------------
        RenderCommands render_commands = build_render_commands(world);
        for (size_t i = 0; i < render_commands.count; ++i) {
            const auto& command = render_commands.commands[i];
            // uniformに変形情報を送る
            glUniform3f(translationLoc, command.transform.position.x, command.transform.position.y, command.transform.position.z);
            // 三角形を描画
            glDrawArrays(GL_TRIANGLES, 0, 3);
        }

        // glfw: swap buffers and poll IO events (keys pressed/released, mouse moved etc.)
        // -------------------------------------------------------------------------------
        glfwSwapBuffers(window);
        glfwPollEvents();
    }

    // optional: de-allocate all resources once they've outlived their purpose:
    // ------------------------------------------------------------------------
    glDeleteVertexArrays(1, &VAO);
    glDeleteBuffers(1, &VBO);
    glDeleteProgram(shaderProgram);

    // Destroy the world
    destroy_world(world);

    // Close the library
    dlclose(handle);

    // Wait for the watcher thread to finish
    watcher_thread.join();

    // glfw: terminate, clearing all previously allocated GLFW resources.
    // ------------------------------------------------------------------
    glfwTerminate();
    return 0;
}


// process all input: query GLFW whether relevant keys are pressed/released this frame and react accordingly
// ---------------------------------------------------------------------------------------------------------
void processInput(GLFWwindow *window) {
    if (glfwGetKey(window, GLFW_KEY_ESCAPE) == GLFW_PRESS)
        glfwSetWindowShouldClose(window, true);
}

// glfw: whenever the window size changed (by OS or user resize) this callback function executes
// -----------------------------------------------------------------------------
void framebuffer_size_callback(GLFWwindow* window, int width, int height) {
    // make sure the viewport matches the new window dimensions; note that width and
    // height will be significantly larger than specified on retina displays.
    glViewport(0, 0, width, height);
}

// utility function for reading shader file
// ----------------------------------------
std::string read_shader_file(const char* file_path) {
    std::ifstream shader_file;
    std::stringstream shader_stream;
    shader_file.exceptions(std::ifstream::failbit | std::ifstream::badbit);
    try {
        shader_file.open(file_path);
        shader_stream << shader_file.rdbuf();
        shader_file.close();
    } catch (std::ifstream::failure& e) {
        std::cerr << "ERROR::SHADER::FILE_NOT_SUCCESSFULLY_READ: " << file_path << std::endl;
    }
    return shader_stream.str();
}

// utility function for creating shader program
// --------------------------------------------
unsigned int create_shader_program(const char* vertex_path, const char* fragment_path) {
    std::string vertex_code_str = read_shader_file(vertex_path);
    std::string fragment_code_str = read_shader_file(fragment_path);
    const char* v_shader_code = vertex_code_str.c_str();
    const char* f_shader_code = fragment_code_str.c_str();

    // 2. compile shaders
    unsigned int vertex, fragment;
    int success;
    char infoLog[512];

    // vertex shader
    vertex = glCreateShader(GL_VERTEX_SHADER);
    glShaderSource(vertex, 1, &v_shader_code, NULL);
    glCompileShader(vertex);
    glGetShaderiv(vertex, GL_COMPILE_STATUS, &success);
    if (!success) {
        glGetShaderInfoLog(vertex, 512, NULL, infoLog);
        std::cerr << "ERROR::SHADER::VERTEX::COMPILATION_FAILED\n" << infoLog << std::endl;
    }

    // fragment shader
    fragment = glCreateShader(GL_FRAGMENT_SHADER);
    glShaderSource(fragment, 1, &f_shader_code, NULL);
    glCompileShader(fragment);
    glGetShaderiv(fragment, GL_COMPILE_STATUS, &success);
    if (!success) {
        glGetShaderInfoLog(fragment, 512, NULL, infoLog);
        std::cerr << "ERROR::SHADER::FRAGMENT::COMPILATION_FAILED\n" << infoLog << std::endl;
    }

    // shader program
    unsigned int ID = glCreateProgram();
    glAttachShader(ID, vertex);
    glAttachShader(ID, fragment);
    glLinkProgram(ID);
    glGetProgramiv(ID, GL_LINK_STATUS, &success);
    if (!success) {
        glGetProgramInfoLog(ID, 512, NULL, infoLog);
        std::cerr << "ERROR::SHADER::PROGRAM::LINKING_FAILED\n" << infoLog << std::endl;
    }

    glDeleteShader(vertex);
    glDeleteShader(fragment);
    return ID;
}
