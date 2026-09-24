#include <Live2DCubismCore.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static void* aligned_read(const char* path, unsigned int* size, size_t align) {
    FILE* f = fopen(path, "rb");
    if (!f) return NULL;
    fseek(f, 0, SEEK_END);
    long n = ftell(f);
    fseek(f, 0, SEEK_SET);
    void* p = aligned_alloc(align, ((size_t)n + align - 1) / align * align);
    if (fread(p, 1, (size_t)n, f) != (size_t)n) { fclose(f); return NULL; }
    fclose(f);
    *size = (unsigned int)n;
    return p;
}

/* Which rendering paths a .moc3 actually exercises: additive and multiplicative
   blending, inverted masks, two-sided drawables. Core only; no window, no Framework.
   See the crate README for how to build and run it. */
int main(int argc, char** argv) {
    for (int a = 1; a < argc; ++a) {
        unsigned int size = 0;
        void* bytes = aligned_read(argv[a], &size, csmAlignofMoc);
        if (!bytes) { printf("%s: unreadable\n", argv[a]); continue; }
        if (!csmHasMocConsistency(bytes, size)) { printf("%s: inconsistent\n", argv[a]); continue; }
        csmMoc* moc = csmReviveMocInPlace(bytes, size);
        if (!moc) { printf("%s: not revivable\n", argv[a]); continue; }
        unsigned int model_size = csmGetSizeofModel(moc);
        void* model_memory = aligned_alloc(csmAlignofModel,
            (model_size + csmAlignofModel - 1) / csmAlignofModel * csmAlignofModel);
        csmModel* model = csmInitializeModelInPlace(moc, model_memory, model_size);
        int count = csmGetDrawableCount(model);
        const csmFlags* flags = csmGetDrawableConstantFlags(model);
        int add = 0, mul = 0, inv = 0, two = 0;
        for (int i = 0; i < count; ++i) {
            if (flags[i] & csmBlendAdditive) ++add;
            if (flags[i] & csmBlendMultiplicative) ++mul;
            if (flags[i] & csmIsInvertedMask) ++inv;
            if (flags[i] & csmIsDoubleSided) ++two;
        }
        printf("%-52s drawables %4d  additive %3d  multiply %3d  inverted-mask %3d  two-sided %3d\n",
               argv[a], count, add, mul, inv, two);
        free(model_memory);
        free(bytes);
    }
    return 0;
}
