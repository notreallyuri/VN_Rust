// SDK-independent GPU regression test for the client-array compatibility layer.
#include "core_profile.hpp"
#include <GLFW/glfw3.h>
#include <cstdio>
#include <stdexcept>

static void require(bool condition, const char* message) {
    if (!condition) throw std::runtime_error(message);
}

static GLuint shader(GLenum type, const char* source) {
    GLuint shader = glCreateShader(type);
    glShaderSource(shader, 1, &source, nullptr);
    glCompileShader(shader);
    GLint compiled;
    glGetShaderiv(shader, GL_COMPILE_STATUS, &compiled);
    require(compiled, "shader failed to compile");
    return shader;
}

static void test() {
    GLuint vs = shader(GL_VERTEX_SHADER,
        "#version 330 core\nlayout(location=0) in vec2 position;\nvoid main(){gl_Position=vec4(position,0,1);}");
    GLuint fs = shader(GL_FRAGMENT_SHADER,
        "#version 330 core\nout vec4 color;\nvoid main(){color=vec4(1,0,0,1);}");
    GLuint program = glCreateProgram();
    glAttachShader(program, vs);
    glAttachShader(program, fs);
    glLinkProgram(program);
    GLint linked;
    glGetProgramiv(program, GL_LINK_STATUS, &linked);
    require(linked, "shader failed to link");
    GLuint host_vao, host_array, host_element, fbo, texture;
    glGenVertexArrays(1, &host_vao);
    glBindVertexArray(host_vao);
    glGenBuffers(1, &host_array);
    glBindBuffer(GL_ARRAY_BUFFER, host_array);
    glGenBuffers(1, &host_element);
    glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, host_element);
    glGenTextures(1, &texture);
    glBindTexture(GL_TEXTURE_2D, texture);
    glTexImage2D(GL_TEXTURE_2D, 0, GL_RGBA8, 32, 32, 0, GL_RGBA, GL_UNSIGNED_BYTE, nullptr);
    glGenFramebuffers(1, &fbo);
    glBindFramebuffer(GL_FRAMEBUFFER, fbo);
    glFramebufferTexture2D(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT0, GL_TEXTURE_2D, texture, 0);
    require(glCheckFramebufferStatus(GL_FRAMEBUFFER) == GL_FRAMEBUFFER_COMPLETE, "invalid test framebuffer");
    glViewport(0, 0, 32, 32);
    glClearColor(0, 0, 1, 1);
    glClear(GL_COLOR_BUFFER_BIT);
    glUseProgram(program);
    glEnable(GL_BLEND);
    glBlendFunc(GL_ONE, GL_ZERO);
    glActiveTexture(GL_TEXTURE2);
    const GLfloat vertices[] = {-1, -1, 3, -1, -1, 3};
    const GLushort indices[] = {0, 1, 2};
    {
        VnGlScope guard;
        // The same sequence the SDK performs before submitting CPU arrays.
        glBindBuffer(GL_ARRAY_BUFFER, 0);
        glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, 0);
        glEnableVertexAttribArray(0);
        vn_vertex_attrib_pointer(0, 2, GL_FLOAT, GL_FALSE, 8, vertices);
        vn_draw_elements(GL_TRIANGLES, 3, GL_UNSIGNED_SHORT, indices);
        glUseProgram(0);
        glActiveTexture(GL_TEXTURE0);
        glDisable(GL_BLEND);
        glViewport(1, 2, 3, 4);
        glBindFramebuffer(GL_FRAMEBUFFER, 0);
        glClearColor(1, 1, 1, 1);
    }
    const auto equals = [](GLenum key, GLint expected) {
        GLint actual;
        glGetIntegerv(key, &actual);
        require(actual == expected, "GL state not restored");
    };
    equals(GL_VERTEX_ARRAY_BINDING, host_vao);
    equals(GL_ARRAY_BUFFER_BINDING, host_array);
    equals(GL_ELEMENT_ARRAY_BUFFER_BINDING, host_element);
    equals(GL_DRAW_FRAMEBUFFER_BINDING, fbo);
    equals(GL_READ_FRAMEBUFFER_BINDING, fbo);
    equals(GL_CURRENT_PROGRAM, program);
    equals(GL_ACTIVE_TEXTURE, GL_TEXTURE2);
    require(glIsEnabled(GL_BLEND), "blend state not restored");
    GLfloat clear[4];
    glGetFloatv(GL_COLOR_CLEAR_VALUE, clear);
    require(clear[0] == 0 && clear[1] == 0 && clear[2] == 1 && clear[3] == 1, "clear color not restored");
    GLint viewport[4];
    glGetIntegerv(GL_VIEWPORT, viewport);
    require(viewport[0] == 0 && viewport[1] == 0 && viewport[2] == 32 && viewport[3] == 32, "viewport not restored");
    unsigned char pixel[4];
    glReadPixels(16, 16, 1, 1, GL_RGBA, GL_UNSIGNED_BYTE, pixel);
    require(pixel[0] == 255 && pixel[1] == 0 && pixel[2] == 0, "CPU arrays did not render through VBOs");
    require(glGetError() == GL_NO_ERROR, "OpenGL error in core-profile rendering");
    // Verify the guard restores state when the bridge catches an exception.
    try {
        VnGlScope guard;
        glUseProgram(0);
        vn_vertex_attrib_pointer(99, 2, GL_FLOAT, GL_FALSE, 8, vertices);
    } catch (const std::runtime_error&) {}
    equals(GL_CURRENT_PROGRAM, program);
    equals(GL_VERTEX_ARRAY_BINDING, host_vao);
    vn_buffers_destroy();
    glDeleteFramebuffers(1, &fbo);
    glDeleteTextures(1, &texture);
    glDeleteBuffers(1, &host_array);
    glDeleteBuffers(1, &host_element);
    glDeleteVertexArrays(1, &host_vao);
    glDeleteProgram(program);
    glDeleteShader(vs);
    glDeleteShader(fs);
}

int main() {
    glfwSetErrorCallback([](int code, const char* message) {
        std::fprintf(stderr, "GLFW %d: %s\n", code, message);
    });
#if GLFW_VERSION_MAJOR == 3 && GLFW_VERSION_MINOR >= 4
    glfwInitHint(GLFW_PLATFORM, GLFW_PLATFORM_X11);
#endif
    if (!glfwInit()) return 2;
    glfwWindowHint(GLFW_VISIBLE, GLFW_FALSE);
    glfwWindowHint(GLFW_CONTEXT_VERSION_MAJOR, 3);
    glfwWindowHint(GLFW_CONTEXT_VERSION_MINOR, 3);
    glfwWindowHint(GLFW_OPENGL_PROFILE, GLFW_OPENGL_CORE_PROFILE);
    GLFWwindow* window = glfwCreateWindow(32, 32, "vn core profile test", nullptr, nullptr);
    if (!window) { glfwTerminate(); return 2; }
    glfwMakeContextCurrent(window);
    glewExperimental = GL_TRUE;
    int result = 0;
    if (glewInit() != GLEW_OK) result = 2;
    while (glGetError() != GL_NO_ERROR) {}
    try { if (!result) { test(); test(); } }
    catch (const std::exception& error) { std::fprintf(stderr, "%s\n", error.what()); result = 1; }
    glfwDestroyWindow(window);
    glfwTerminate();
    if (!result) std::puts("core-profile drawing, GL state restoration, and resource recreation passed");
    return result;
}
