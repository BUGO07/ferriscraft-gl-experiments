pub use bevy_ecs::{prelude::*, schedule::ScheduleLabel};
use glam::*;

#[derive(Component)]
pub struct ChunkEntity;

#[derive(Component)]
pub struct Camera3d {
    pub fov: f32,
    pub near: f32,
    pub far: f32,
}

#[derive(Component)]
pub struct DirectionalLight {
    pub illuminance: f32,
}

#[derive(Component, Clone, Copy)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Transform {
    pub const DEFAULT: Self = Self::from_translation(Vec3::ZERO);

    #[inline]
    pub const fn from_translation(translation: Vec3) -> Self {
        Self {
            translation,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
    #[inline]
    pub fn from_xyz(x: f32, y: f32, z: f32) -> Self {
        Self::from_translation(vec3(x, y, z))
    }
    #[inline]
    pub fn with_rotation(mut self, rotation: Quat) -> Self {
        self.rotation = rotation;
        self
    }
    #[inline]
    pub fn with_scale(mut self, scale: Vec3) -> Self {
        self.scale = scale;
        self
    }
    #[inline]
    pub fn from_mat4(mat4: Mat4) -> Self {
        let (scale, rotation, translation) = mat4.to_scale_rotation_translation();
        Self {
            translation,
            rotation,
            scale,
        }
    }
    #[inline]
    pub fn as_mat4(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }
}

#[derive(ScheduleLabel, Hash, PartialEq, Eq, Debug, Clone)]
pub struct Startup;

#[derive(ScheduleLabel, Hash, PartialEq, Eq, Debug, Clone)]
pub struct Render;

#[derive(ScheduleLabel, Hash, PartialEq, Eq, Debug, Clone)]
pub struct Update;

#[derive(ScheduleLabel, Hash, PartialEq, Eq, Debug, Clone)]
pub struct FixedUpdate;
