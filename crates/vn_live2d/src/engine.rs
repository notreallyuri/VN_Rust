use vn_engine::data::assets::Assets;
use vn_engine::game::visuals::{CharacterVisual, CharacterVisualFactory, VisualFrame};
use vn_engine::raylib::prelude::*;

use crate::{Live2dCharacter, Model};

impl CharacterVisualFactory for Live2dCharacter {
    fn appearances(&self) -> Vec<String> {
        self.appearances.keys().cloned().collect()
    }

    fn validate(&self, assets: &Assets) -> Result<(), String> {
        self.check(assets).map_err(|e| e.to_string())
    }

    fn load(
        &self,
        appearance: &str,
        assets: &Assets,
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
    ) -> Result<Box<dyn CharacterVisual>, String> {
        let preset = self
            .appearances
            .get(appearance)
            .ok_or_else(|| format!("unknown Live2D appearance '{appearance}'"))?;
        let bundle = self.bundle(assets).map_err(|e| e.to_string())?;
        let mut model = Model::load_owned(rl, thread, &bundle).map_err(|e| e.to_string())?;
        apply(&mut model, preset)?;
        let canvas = model.canvas_size().map_err(|e| e.to_string())?;
        Ok(Box::new(Live2dVisual {
            model,
            canvas,
            size: self.size.unwrap_or(canvas),
            preset: preset.clone(),
        }))
    }
}

fn apply(model: &mut Model<'static>, preset: &crate::Appearance) -> Result<(), String> {
    if let Some(expression) = &preset.expression {
        model
            .set_expression(expression)
            .map_err(|e| e.to_string())?;
    }
    if let Some((group, index, looping)) = &preset.motion {
        model
            .start_motion(group, *index, *looping)
            .map_err(|e| e.to_string())?;
    }
    for (id, value) in &preset.parameters {
        model
            .set_parameter(id, Some(*value))
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

struct Live2dVisual {
    model: Model<'static>,
    canvas: Vector2,
    size: Vector2,
    preset: crate::Appearance,
}

impl CharacterVisual for Live2dVisual {
    fn size(&self) -> Vector2 {
        self.size
    }

    fn update(&mut self, seconds: f32) -> Result<(), String> {
        self.model.update(seconds).map_err(|e| e.to_string())
    }

    fn restart(&mut self) -> Result<(), String> {
        apply(&mut self.model, &self.preset)
    }

    fn draw(&mut self, draw: &mut RaylibDrawHandle, frame: VisualFrame) -> Result<(), String> {
        let rect = frame.rect;
        let layout = frame.layout_size;
        let scale_x = rect.width / layout.x * self.canvas.y / self.canvas.x;
        let scale_y = rect.height / layout.y;
        let x = 2.0 * (rect.x + rect.width / 2.0) / layout.x - 1.0;
        let y = 1.0 - 2.0 * (rect.y + rect.height / 2.0) / layout.y;
        let matrix = [
            scale_x, 0.0, 0.0, 0.0, 0.0, scale_y, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, x, y, 0.0, 1.0,
        ];
        self.model
            .draw(draw, matrix, frame.target_size, frame.opacity)
            .map_err(|e| e.to_string())
    }
}
