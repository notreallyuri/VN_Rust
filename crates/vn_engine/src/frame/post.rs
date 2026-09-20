use raylib::prelude::*;

use crate::frame::target::RenderTarget;

pub const GRAIN: &str = r#"#version 330
in vec2 fragTexCoord;
in vec4 fragColor;
uniform sampler2D texture0;
uniform vec4 colDiffuse;
uniform float amount;
uniform float time;
out vec4 finalColor;

float noise(vec2 at) {
    return fract(sin(dot(at, vec2(12.9898, 78.233))) * 43758.5453);
}

void main() {
    vec4 colour = texture(texture0, fragTexCoord) * colDiffuse * fragColor;
    float grain = noise(fragTexCoord * vec2(1024.0, 1024.0) + fract(time) * 17.0) - 0.5;
    finalColor = vec4(colour.rgb + grain * amount, colour.a);
}
"#;

pub const DESATURATE: &str = r#"#version 330
in vec2 fragTexCoord;
in vec4 fragColor;
uniform sampler2D texture0;
uniform vec4 colDiffuse;
uniform float amount;
out vec4 finalColor;

void main() {
    vec4 colour = texture(texture0, fragTexCoord) * colDiffuse * fragColor;
    float grey = dot(colour.rgb, vec3(0.299, 0.587, 0.114));
    finalColor = vec4(mix(colour.rgb, vec3(grey), amount), colour.a);
}
"#;

pub const BLUR: &str = r#"#version 330
in vec2 fragTexCoord;
in vec4 fragColor;
uniform sampler2D texture0;
uniform vec4 colDiffuse;
uniform float amount;
uniform vec2 pixel;
out vec4 finalColor;

void main() {
    vec4 sum = vec4(0.0);
    float weight = 0.0;
    for (int x = -2; x <= 2; x++) {
        for (int y = -2; y <= 2; y++) {
            float w = 1.0 / (1.0 + float(abs(x) + abs(y)));
            sum += texture(texture0, fragTexCoord + vec2(x, y) * pixel * amount) * w;
            weight += w;
        }
    }
    finalColor = (sum / weight) * colDiffuse * fragColor;
}
"#;

pub const FXAA: &str = r#"#version 330
in vec2 fragTexCoord;
in vec4 fragColor;
uniform sampler2D texture0;
uniform vec4 colDiffuse;
uniform float amount;
uniform vec2 pixel;
out vec4 finalColor;

float luma(vec3 colour) {
    return dot(colour, vec3(0.299, 0.587, 0.114));
}

void main() {
    vec3 here = texture(texture0, fragTexCoord).rgb;
    vec3 nw = texture(texture0, fragTexCoord + vec2(-1.0, -1.0) * pixel).rgb;
    vec3 ne = texture(texture0, fragTexCoord + vec2(1.0, -1.0) * pixel).rgb;
    vec3 sw = texture(texture0, fragTexCoord + vec2(-1.0, 1.0) * pixel).rgb;
    vec3 se = texture(texture0, fragTexCoord + vec2(1.0, 1.0) * pixel).rgb;

    float lm = luma(here);
    float lnw = luma(nw);
    float lne = luma(ne);
    float lsw = luma(sw);
    float lse = luma(se);

    float highest = max(lm, max(max(lnw, lne), max(lsw, lse)));
    float lowest = min(lm, min(min(lnw, lne), min(lsw, lse)));
    float contrast = highest - lowest;

    vec2 direction = vec2(
        -((lnw + lne) - (lsw + lse)),
        ((lnw + lsw) - (lne + lse))
    );
    float scale = min(1.0, contrast * 8.0) * amount;
    vec2 step = clamp(direction * scale, vec2(-2.0), vec2(2.0)) * pixel;

    vec3 blurred = 0.5 * (
        texture(texture0, fragTexCoord + step * 0.5).rgb +
        texture(texture0, fragTexCoord - step * 0.5).rgb
    );
    vec3 result = mix(here, blurred, min(1.0, contrast * 6.0) * amount);
    finalColor = vec4(result, texture(texture0, fragTexCoord).a) * colDiffuse * fragColor;
}
"#;

pub struct Pass {
    name: String,
    enabled: bool,
    amount: f32,
    shader: Option<Shader>,
    locations: Locations,
}

#[derive(Default, Clone, Copy)]
struct Locations {
    amount: Option<i32>,
    time: Option<i32>,
    pixel: Option<i32>,
}

impl Pass {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn amount(&self) -> f32 {
        self.amount
    }

    pub fn drawable(&self) -> bool {
        self.enabled && self.shader.is_some() && self.amount > 0.0
    }
}

#[derive(Default)]
pub struct PostChain {
    passes: Vec<Pass>,
    scratch: [RenderTarget; 2],
}

impl PostChain {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn declare(&mut self, name: impl Into<String>, amount: f32) {
        self.passes.push(Pass {
            name: name.into(),
            enabled: false,
            amount,
            shader: None,
            locations: Locations::default(),
        });
    }

    pub fn load(
        &mut self,
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        name: impl Into<String>,
        fragment: &str,
        amount: f32,
    ) {
        let name = name.into();
        let shader = rl.load_shader_from_memory(thread, None, Some(fragment));
        if !shader.is_shader_valid() {
            eprintln!("⚠️ Shader '{}' did not compile; it will be skipped", name);
            self.passes.push(Pass {
                name,
                enabled: false,
                amount,
                shader: None,
                locations: Locations::default(),
            });
            return;
        }
        let locations = Locations {
            amount: location(&shader, "amount"),
            time: location(&shader, "time"),
            pixel: location(&shader, "pixel"),
        };
        self.passes.push(Pass {
            name,
            enabled: false,
            amount,
            shader: Some(shader),
            locations,
        });
    }

    pub fn passes(&self) -> &[Pass] {
        &self.passes
    }

    pub fn set_enabled(&mut self, name: &str, enabled: bool) -> bool {
        match self.passes.iter_mut().find(|pass| pass.name == name) {
            Some(pass) => {
                pass.enabled = enabled;
                true
            }
            None => false,
        }
    }

    pub fn set_amount(&mut self, name: &str, amount: f32) -> bool {
        match self.passes.iter_mut().find(|pass| pass.name == name) {
            Some(pass) => {
                pass.amount = amount.max(0.0);
                true
            }
            None => false,
        }
    }

    pub fn clear(&mut self) {
        for pass in &mut self.passes {
            pass.enabled = false;
        }
    }

    pub fn active(&self) -> Vec<&str> {
        self.passes
            .iter()
            .filter(|pass| pass.enabled && pass.amount > 0.0)
            .map(|pass| pass.name.as_str())
            .collect()
    }

    pub fn busy(&self) -> bool {
        self.passes.iter().any(Pass::drawable)
    }

    pub fn resize(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread, size: (i32, i32)) {
        if !self.busy() {
            return;
        }
        for scratch in &mut self.scratch {
            scratch.resize(rl, thread, size);
        }
    }

    pub fn apply<'a>(
        &'a mut self,
        d: &mut RaylibDrawHandle,
        thread: &RaylibThread,
        frame: &'a RenderTexture2D,
        size: (i32, i32),
        now: f64,
    ) -> &'a RenderTexture2D {
        let drawn: Vec<usize> = (0..self.passes.len())
            .filter(|&i| self.passes[i].drawable())
            .collect();
        if drawn.is_empty() || self.scratch.iter().any(|target| target.frame().is_none()) {
            return frame;
        }

        let whole = Rectangle::new(0.0, 0.0, size.0 as f32, size.1 as f32);
        let flipped = Rectangle::new(0.0, 0.0, size.0 as f32, -(size.1 as f32));
        let pixel = [1.0 / size.0 as f32, 1.0 / size.1 as f32];
        let mut read: Option<usize> = None;

        for (step, &index) in drawn.iter().enumerate() {
            let write = step % 2;
            let (left, right) = self.scratch.split_at_mut(1);
            let (previous, into) = if write == 0 {
                (&right[0], &mut left[0])
            } else {
                (&left[0], &mut right[0])
            };

            let pass = &mut self.passes[index];
            let Some(shader) = &mut pass.shader else {
                continue;
            };

            if let Some(location) = pass.locations.amount {
                shader.set_shader_value(location, pass.amount);
            }
            if let Some(location) = pass.locations.time {
                shader.set_shader_value(location, now as f32);
            }
            if let Some(location) = pass.locations.pixel {
                shader.set_shader_value(location, pixel);
            }

            let source = match read {
                Some(_) => previous.frame().unwrap().texture(),
                None => frame.texture(),
            };
            let Some(into) = into.frame_mut() else {
                continue;
            };

            let mut texture_mode = d.begin_texture_mode(thread, into);
            texture_mode.clear_background(Color::BLACK);
            {
                let mut shaded = texture_mode.begin_shader_mode(shader);
                shaded.draw_texture_pro(source, flipped, whole, Vector2::zero(), 0.0, Color::WHITE);
            }
            drop(texture_mode);

            read = Some(write);
        }

        match read {
            Some(index) => self.scratch[index].frame().unwrap(),
            None => frame,
        }
    }
}

fn location(shader: &Shader, name: &str) -> Option<i32> {
    let location = shader.get_shader_location(name);
    (location >= 0).then_some(location)
}
