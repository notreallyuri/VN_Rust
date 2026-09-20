#include "core_profile.hpp"
#include <algorithm>
#include <array>
#include <stdexcept>

namespace {
struct Attribute { GLint size; GLsizei stride; GLboolean normalized; const void* data = nullptr; };
std::array<Attribute, 16> attributes;
GLuint private_vao = 0, buffers[17] = {};
const GLenum capabilities[] = {GL_BLEND, GL_CULL_FACE, GL_DEPTH_TEST, GL_SCISSOR_TEST, GL_STENCIL_TEST};
}

void vn_vertex_attrib_pointer(GLuint index, GLint size, GLenum type, GLboolean normalized,
                             GLsizei stride, const void* pointer) {
    if (index >= attributes.size() || type != GL_FLOAT || size != 2 || stride != 8 || !pointer)
        throw std::runtime_error("unsupported Cubism vertex layout (expected SDK 5-r.5)");
    attributes[index] = {size, stride, normalized, pointer};
}

void vn_draw_elements(GLenum mode, GLsizei count, GLenum type, const void* indices) {
    if (count <= 0) return;
    if (type != GL_UNSIGNED_SHORT || !indices)
        throw std::runtime_error("unsupported Cubism index layout");
    auto* index = static_cast<const GLushort*>(indices);
    const size_t vertices = static_cast<size_t>(*std::max_element(index, index + count)) + 1;
    for (GLuint i = 0; i < attributes.size(); ++i) {
        auto& a = attributes[i];
        if (!a.data) continue;
        glBindBuffer(GL_ARRAY_BUFFER, buffers[i]);
        glBufferData(GL_ARRAY_BUFFER, vertices * a.stride, a.data, GL_STREAM_DRAW);
        glVertexAttribPointer(i, a.size, GL_FLOAT, a.normalized, a.stride, nullptr);
        a.data = nullptr;
    }
    glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, buffers[16]);
    glBufferData(GL_ELEMENT_ARRAY_BUFFER, count * sizeof(GLushort), indices, GL_STREAM_DRAW);
    glDrawElements(mode, count, type, nullptr);
}

VnGlScope::VnGlScope() {
    glGetIntegerv(GL_VERTEX_ARRAY_BINDING, &vao);
    glGetIntegerv(GL_ARRAY_BUFFER_BINDING, &array);
    glGetIntegerv(GL_ELEMENT_ARRAY_BUFFER_BINDING, &element);
    glGetIntegerv(GL_DRAW_FRAMEBUFFER_BINDING, &draw_fbo);
    glGetIntegerv(GL_READ_FRAMEBUFFER_BINDING, &read_fbo);
    glGetIntegerv(GL_VIEWPORT, viewport);
    glGetIntegerv(GL_CURRENT_PROGRAM, &program);
    glGetIntegerv(GL_ACTIVE_TEXTURE, &active);
    for (int i = 0; i < 3; ++i) {
        glActiveTexture(GL_TEXTURE0 + i);
        glGetIntegerv(GL_TEXTURE_BINDING_2D, &textures[i]);
    }
    glActiveTexture(active);
    glGetIntegerv(GL_BLEND_SRC_RGB, &blend[0]);
    glGetIntegerv(GL_BLEND_DST_RGB, &blend[1]);
    glGetIntegerv(GL_BLEND_SRC_ALPHA, &blend[2]);
    glGetIntegerv(GL_BLEND_DST_ALPHA, &blend[3]);
    glGetIntegerv(GL_BLEND_EQUATION_RGB, &equations[0]);
    glGetIntegerv(GL_BLEND_EQUATION_ALPHA, &equations[1]);
    glGetIntegerv(GL_FRONT_FACE, &front);
    glGetIntegerv(GL_CULL_FACE_MODE, &cull_mode);
    glGetBooleanv(GL_COLOR_WRITEMASK, color);
    glGetBooleanv(GL_DEPTH_WRITEMASK, &depth_mask);
    glGetFloatv(GL_COLOR_CLEAR_VALUE, clear_color);
    for (int i = 0; i < 5; ++i) enabled[i] = glIsEnabled(capabilities[i]);
    if (!private_vao) {
        glGenVertexArrays(1, &private_vao);
        glGenBuffers(17, buffers);
    }
    attributes = {};
    glBindVertexArray(private_vao);
}

VnGlScope::~VnGlScope() {
    glBindVertexArray(vao);
    glBindBuffer(GL_ARRAY_BUFFER, array);
    // GL forbids changing an element binding without a VAO in core profile.
    if (vao) glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, element);
    glBindFramebuffer(GL_DRAW_FRAMEBUFFER, draw_fbo);
    glBindFramebuffer(GL_READ_FRAMEBUFFER, read_fbo);
    glViewport(viewport[0], viewport[1], viewport[2], viewport[3]);
    glUseProgram(program);
    for (int i = 0; i < 3; ++i) {
        glActiveTexture(GL_TEXTURE0 + i);
        glBindTexture(GL_TEXTURE_2D, textures[i]);
    }
    glActiveTexture(active);
    glBlendFuncSeparate(blend[0], blend[1], blend[2], blend[3]);
    glBlendEquationSeparate(equations[0], equations[1]);
    glFrontFace(front);
    glCullFace(cull_mode);
    glColorMask(color[0], color[1], color[2], color[3]);
    glDepthMask(depth_mask);
    glClearColor(clear_color[0], clear_color[1], clear_color[2], clear_color[3]);
    for (int i = 0; i < 5; ++i) {
        if (enabled[i]) glEnable(capabilities[i]); else glDisable(capabilities[i]);
    }
}

void vn_buffers_destroy() {
    glDeleteBuffers(17, buffers);
    glDeleteVertexArrays(1, &private_vao);
    private_vao = 0;
    std::fill(std::begin(buffers), std::end(buffers), 0);
    attributes = {};
}
