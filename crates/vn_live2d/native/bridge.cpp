#include "bridge.h"
#include "core_profile.hpp"
#include <CubismFramework.hpp>
#include <CubismModelSettingJson.hpp>
#include <ICubismAllocator.hpp>
#include <Id/CubismIdManager.hpp>
#include <Model/CubismUserModel.hpp>
#include <Motion/CubismMotion.hpp>
#include <Rendering/OpenGL/CubismRenderer_OpenGLES2.hpp>
#include <algorithm>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <map>
#include <memory>
#include <stdexcept>
#include <string>
#include "vn_shaders.hpp"

using namespace Live2D::Cubism::Framework;
using Renderer = Rendering::CubismRenderer_OpenGLES2;

namespace {
class Allocator final : public ICubismAllocator {
    void* Allocate(csmSizeType size) override { return std::malloc(size); }
    void Deallocate(void* memory) override { std::free(memory); }
    void* AllocateAligned(csmSizeType size, csmUint32 alignment) override {
        void* memory = nullptr;
        return posix_memalign(&memory, std::max<size_t>(alignment, sizeof(void*)), size) == 0 ? memory : nullptr;
    }
    void DeallocateAligned(void* memory) override { std::free(memory); }
} allocator;
CubismFramework::Option options{};
bool started = false;

void error_text(char* dest, size_t capacity, const char* message) noexcept {
    if (dest && capacity) std::snprintf(dest, capacity, "%s", message);
}
template<class F> int checked(char* error, size_t capacity, F&& operation) noexcept {
    try { operation(); return 1; }
    catch (const std::exception& e) { error_text(error, capacity, e.what()); }
    catch (...) { error_text(error, capacity, "unknown Cubism native exception"); }
    return 0;
}
void release_bytes(csmByte* bytes) { std::free(bytes); }
void log_message(const char* text) { std::fprintf(stderr, "Cubism: %s\n", text); }
struct DeleteMotion {
    void operator()(ACubismMotion* p) const { ACubismMotion::Delete(p); }
};
using MotionPtr = std::unique_ptr<ACubismMotion, DeleteMotion>;
}

struct VnModel final : CubismUserModel {
    std::unique_ptr<CubismModelSettingJson> settings;
    std::map<std::pair<std::string, int>, MotionPtr> motions;
    std::map<std::string, MotionPtr> expressions;
    std::map<int, float> parameters;

    ~VnModel() override {
        // Managers borrow motions owned by these maps.
        _motionManager->StopAllMotions();
        _expressionManager->StopAllMotions();
    }

    void initialize(const uint8_t* json, int json_size, const uint8_t* moc, int moc_size, int textures) {
        settings = std::make_unique<CubismModelSettingJson>(json, json_size);
        LoadModel(moc, moc_size, true);
        if (!_model) throw std::runtime_error("Cubism rejected the MOC3 model (check SDK compatibility)");
        for (int i = 0; i < _model->GetDrawableCount(); ++i) {
            int slot = _model->GetDrawableTextureIndex(i);
            if (slot < 0 || slot >= textures) throw std::runtime_error("MOC3 references a missing texture slot");
        }
        if (settings->GetEyeBlinkParameterCount() > 0) _eyeBlink = CubismEyeBlink::Create(settings.get());
        csmMap<csmString, csmFloat32> layout;
        settings->GetLayoutMap(layout);
        _modelMatrix->SetupFromLayout(layout);
        CreateRenderer(1, 1);
        GetRenderer<Renderer>()->SetIsPremultipliedAlpha(false);
        _model->Update();
    }

    void asset(int kind, const char* name, int index, const uint8_t* bytes, int size) {
        switch (kind) {
        case 0:
            LoadPhysics(bytes, size);
            if (!_physics) throw std::runtime_error("invalid physics3.json");
            break;
        case 1:
            LoadPose(bytes, size);
            if (!_pose) throw std::runtime_error("invalid pose3.json");
            break;
        case 2: {
            auto* motion = static_cast<CubismMotion*>(LoadMotion(bytes, size, name, nullptr, nullptr,
                                                               settings.get(), name, index, true));
            if (!motion) throw std::runtime_error("invalid motion3.json");
            MotionPtr owner(motion);
            csmVector<CubismIdHandle> blink, lips;
            for (int i = 0; i < settings->GetEyeBlinkParameterCount(); ++i)
                blink.PushBack(settings->GetEyeBlinkParameterId(i));
            for (int i = 0; i < settings->GetLipSyncParameterCount(); ++i)
                lips.PushBack(settings->GetLipSyncParameterId(i));
            motion->SetEffectIds(blink, lips);
            motions.emplace(std::make_pair(std::string(name), index), std::move(owner));
            break;
        }
        case 3: {
            MotionPtr expression(LoadExpression(bytes, size, name));
            if (!expression) throw std::runtime_error("invalid exp3.json");
            expressions.emplace(name, std::move(expression));
            break;
        }
        default: throw std::runtime_error("unknown model asset kind");
        }
    }

    void update(float seconds) {
        _model->LoadParameters();
        bool motion_updated = _motionManager->UpdateMotion(_model, seconds);
        _model->SaveParameters();
        if (!motion_updated && _eyeBlink) _eyeBlink->UpdateParameters(_model, seconds);
        _expressionManager->UpdateMotion(_model, seconds);
        for (const auto& parameter : parameters) _model->SetParameterValue(parameter.first, parameter.second);
        if (_physics) _physics->Evaluate(_model, seconds);
        if (_pose) _pose->UpdateParameters(_model, seconds);
        _model->Update();
    }

    void start_motion(ACubismMotion* motion) {
        _motionManager->SetReservePriority(3);
        _motionManager->StartMotionPriority(motion, false, 3);
    }

    void start_expression(ACubismMotion* expression) {
        _expressionManager->StartMotion(expression, false);
    }
};

extern "C" int vn_cubism_start(char* error, size_t capacity) {
    return checked(error, capacity, [] {
        if (started) throw std::runtime_error("a Cubism runtime is already active");
        glewExperimental = GL_TRUE;
        if (glewInit() != GLEW_OK) throw std::runtime_error("GLEW initialization failed; a current OpenGL context is required");
        // GLEW can leave GL_INVALID_ENUM after probing a core context.
        while (glGetError() != GL_NO_ERROR) {}
        if (!GLEW_VERSION_3_3) throw std::runtime_error("Cubism adapter requires OpenGL 3.3");
        options.LogFunction = log_message;
        options.LoggingLevel = CubismFramework::Option::LogLevel_Warning;
        options.LoadFileFunction = vn_shader;
        options.ReleaseBytesFunction = release_bytes;
        if (!CubismFramework::StartUp(&allocator, &options)) throw std::runtime_error("Cubism startup failed");
        CubismFramework::Initialize();
        started = true;
    });
}

extern "C" void vn_cubism_stop() {
    // Destruction never lets an exception unwind through Rust.
    checked(nullptr, 0, [] {
        if (!started) return;
        Rendering::CubismRenderer::StaticRelease();
        vn_buffers_destroy();
        CubismFramework::Dispose();
        CubismFramework::CleanUp();
        started = false;
    });
}

extern "C" VnModel* vn_model_create(const uint8_t* manifest, int manifest_size,
    const uint8_t* moc, int moc_size, int textures, char* error, size_t capacity) {
    VnModel* result = nullptr;
    if (!checked(error, capacity, [&] {
        VnGlScope gl;
        auto model = std::make_unique<VnModel>();
        model->initialize(manifest, manifest_size, moc, moc_size, textures);
        result = model.release();
    })) return nullptr;
    return result;
}
extern "C" void vn_model_destroy(VnModel* model) {
    checked(nullptr, 0, [&] { VnGlScope gl; delete model; });
}
extern "C" int vn_model_size(VnModel* model, float* width, float* height, char* error, size_t capacity) {
    return checked(error, capacity, [&] {
        *width = model->GetModel()->GetCanvasWidthPixel();
        *height = model->GetModel()->GetCanvasHeightPixel();
    });
}
extern "C" int vn_model_asset(VnModel* model, int kind, const char* name, int index,
    const uint8_t* bytes, int size, char* error, size_t capacity) {
    return checked(error, capacity, [&] { model->asset(kind, name, index, bytes, size); });
}
extern "C" int vn_model_texture(VnModel* model, unsigned slot, unsigned texture, char* error, size_t capacity) {
    return checked(error, capacity, [&] { model->GetRenderer<Renderer>()->BindTexture(slot, texture); });
}
extern "C" int vn_model_motion(VnModel* model, const char* group, int index, int looping, char* error, size_t capacity) {
    return checked(error, capacity, [&] {
        auto found = model->motions.find({group, index});
        if (found == model->motions.end()) throw std::runtime_error("unknown motion group/index");
        found->second->SetLoop(looping != 0);
        model->GetModel()->LoadParameters();
        // A force priority makes explicit game requests replace the current motion.
        // Exposed through a member below, since the SDK manager is protected.
        model->start_motion(found->second.get());
    });
}
extern "C" int vn_model_expression(VnModel* model, const char* name, char* error, size_t capacity) {
    return checked(error, capacity, [&] {
        auto found = model->expressions.find(name);
        if (found == model->expressions.end()) throw std::runtime_error("unknown expression");
        model->start_expression(found->second.get());
    });
}
extern "C" int vn_model_update(VnModel* model, float seconds, char* error, size_t capacity) {
    return checked(error, capacity, [&] { model->update(seconds); });
}
extern "C" int vn_model_parameter(VnModel* model, const char* name, float value, int clear,
    char* error, size_t capacity) {
    return checked(error, capacity, [&] {
        auto id = CubismFramework::GetIdManager()->GetId(name);
        auto* data = model->GetModel();
        for (int i = 0; i < data->GetParameterCount(); ++i) {
            if (data->GetParameterId(i) != id) continue;
            if (clear) model->parameters.erase(i); else model->parameters[i] = value;
            return;
        }
        throw std::runtime_error("unknown model parameter");
    });
}
extern "C" int vn_model_draw(VnModel* model, const float* matrix, float opacity, unsigned width,
    unsigned height, char* error, size_t capacity) {
    return checked(error, capacity, [&] {
        VnGlScope gl;
        model->SetRenderTargetSize(width, height);
        CubismMatrix44 transform;
        transform.SetMatrix(const_cast<float*>(matrix));
        transform.MultiplyByMatrix(model->GetModelMatrix());
        auto* renderer = model->GetRenderer<Renderer>();
        renderer->SetMvpMatrix(&transform);
        renderer->SetModelColor(1, 1, 1, opacity * model->GetModel()->GetModelOpacity());
        renderer->DrawModel();
        GLenum error = glGetError();
        if (error != GL_NO_ERROR) throw std::runtime_error("OpenGL error during Cubism drawing: " + std::to_string(error));
    });
}
