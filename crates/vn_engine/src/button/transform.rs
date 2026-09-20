use raylib::prelude::*;

use super::mix;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform {
    pub scale: Vector2,
    pub rotation: f32,
    pub skew: Vector2,
    pub offset: Vector2,
    pub origin: Vector2,
}

impl Default for Transform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Transform {
    pub const IDENTITY: Transform = Transform {
        scale: Vector2 { x: 1.0, y: 1.0 },
        rotation: 0.0,
        skew: Vector2 { x: 0.0, y: 0.0 },
        offset: Vector2 { x: 0.0, y: 0.0 },
        origin: Vector2 { x: 0.5, y: 0.5 },
    };

    pub fn scale(mut self, factor: f32) -> Self {
        self.scale = Vector2::new(factor, factor);
        self
    }

    pub fn scale_xy(mut self, x: f32, y: f32) -> Self {
        self.scale = Vector2::new(x, y);
        self
    }

    pub fn rotate(mut self, degrees: f32) -> Self {
        self.rotation = degrees;
        self
    }

    pub fn skew(mut self, x_degrees: f32, y_degrees: f32) -> Self {
        self.skew = Vector2::new(x_degrees, y_degrees);
        self
    }

    pub fn offset(mut self, x: f32, y: f32) -> Self {
        self.offset = Vector2::new(x, y);
        self
    }

    pub fn origin(mut self, x: f32, y: f32) -> Self {
        self.origin = Vector2::new(x, y);
        self
    }

    pub fn is_identity(&self) -> bool {
        *self == Self::IDENTITY
            || (self.scale == Self::IDENTITY.scale
                && self.rotation == 0.0
                && self.skew == Self::IDENTITY.skew
                && self.offset == Self::IDENTITY.offset)
    }

    pub fn lerp(&self, other: &Transform, t: f32) -> Transform {
        let v = |a: Vector2, b: Vector2| Vector2::new(mix(a.x, b.x, t), mix(a.y, b.y, t));
        Transform {
            scale: v(self.scale, other.scale),
            rotation: mix(self.rotation, other.rotation, t),
            skew: v(self.skew, other.skew),
            offset: v(self.offset, other.offset),
            origin: v(self.origin, other.origin),
        }
    }

    pub(crate) fn pivot(&self, rect: Rectangle) -> Vector2 {
        Vector2::new(
            rect.x + rect.width * self.origin.x,
            rect.y + rect.height * self.origin.y,
        )
    }

    pub(crate) fn shear(&self) -> (f32, f32) {
        (
            self.skew.x.to_radians().tan(),
            self.skew.y.to_radians().tan(),
        )
    }

    pub fn apply(&self, rect: Rectangle, point: Vector2) -> Vector2 {
        let pivot = self.pivot(rect);
        let (sx, sy) = self.shear();
        let local = Vector2::new(
            (point.x - pivot.x) * self.scale.x,
            (point.y - pivot.y) * self.scale.y,
        );
        let sheared = Vector2::new(local.x + sx * local.y, local.y + sy * local.x);
        let (sin, cos) = self.rotation.to_radians().sin_cos();
        Vector2::new(
            sheared.x * cos - sheared.y * sin + pivot.x + self.offset.x,
            sheared.x * sin + sheared.y * cos + pivot.y + self.offset.y,
        )
    }

    pub fn invert(&self, rect: Rectangle, point: Vector2) -> Option<Vector2> {
        let pivot = self.pivot(rect);
        let (sx, sy) = self.shear();
        let determinant = 1.0 - sx * sy;
        if determinant.abs() < 1e-6 || self.scale.x == 0.0 || self.scale.y == 0.0 {
            return None;
        }

        let moved = Vector2::new(
            point.x - pivot.x - self.offset.x,
            point.y - pivot.y - self.offset.y,
        );
        let (sin, cos) = self.rotation.to_radians().sin_cos();
        let unrotated = Vector2::new(
            moved.x * cos + moved.y * sin,
            -moved.x * sin + moved.y * cos,
        );
        let unsheared = Vector2::new(
            (unrotated.x - sx * unrotated.y) / determinant,
            (unrotated.y - sy * unrotated.x) / determinant,
        );
        Some(Vector2::new(
            unsheared.x / self.scale.x + pivot.x,
            unsheared.y / self.scale.y + pivot.y,
        ))
    }

    pub fn contains(&self, rect: Rectangle, point: Vector2) -> bool {
        self.invert(rect, point)
            .is_some_and(|local| rect.check_collision_point_rec(local))
    }
}
