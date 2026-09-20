#pragma once
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif
typedef struct VnModel VnModel;
int vn_cubism_start(char* error, size_t capacity);
void vn_cubism_stop(void);
VnModel* vn_model_create(const uint8_t* manifest, int manifest_size,
    const uint8_t* moc, int moc_size, int textures, char* error, size_t capacity);
void vn_model_destroy(VnModel* model);
int vn_model_size(VnModel* model, float* width, float* height, char* error, size_t capacity);
int vn_model_asset(VnModel* model, int kind, const char* name, int index,
    const uint8_t* bytes, int size, char* error, size_t capacity);
int vn_model_texture(VnModel* model, unsigned int slot, unsigned int texture, char* error, size_t capacity);
int vn_model_motion(VnModel* model, const char* group, int index, int looping, char* error, size_t capacity);
int vn_model_expression(VnModel* model, const char* name, char* error, size_t capacity);
int vn_model_update(VnModel* model, float seconds, char* error, size_t capacity);
int vn_model_parameter(VnModel* model, const char* name, float value, int clear, char* error, size_t capacity);
int vn_model_draw(VnModel* model, const float* matrix, float opacity, unsigned int width,
    unsigned int height, char* error, size_t capacity);
#ifdef __cplusplus
}
#endif
