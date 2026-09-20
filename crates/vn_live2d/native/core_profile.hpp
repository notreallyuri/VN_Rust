#pragma once
#include <GL/glew.h>

void vn_vertex_attrib_pointer(GLuint index, GLint size, GLenum type, GLboolean normalized,
                             GLsizei stride, const void* pointer);
void vn_draw_elements(GLenum mode, GLsizei count, GLenum type, const void* indices);
void vn_buffers_destroy();

// Keep native rendering away from raylib's VAO and restore state even on error.
class VnGlScope {
public:
    VnGlScope();
    ~VnGlScope();
private:
    GLint vao, array, element, draw_fbo, read_fbo, viewport[4], program, active;
    GLint textures[3], blend[4], equations[2], front, cull_mode;
    GLboolean enabled[5], color[4], depth_mask;
    GLfloat clear_color[4];
};
