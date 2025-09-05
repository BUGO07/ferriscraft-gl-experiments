use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

pub use bevy_ecs::{prelude::*, schedule::ScheduleLabel};
pub use glam::*;

use gl::types::*;
use glfw::{Glfw, Key, MouseButton, PWindow};

use crate::{render::material::Material, world::mesher::ChunkMesh};

pub struct NSWindow {
    pub window: PWindow,
    pub context: Glfw,
}

#[derive(Resource)]
pub struct Window {
    pub cursor_grab: bool,
    pub cursor_visible: bool,
    pub width: GLint,
    pub height: GLint,
}

#[derive(Resource, Debug, Default)]
pub struct DebugInfo {
    pub draw_calls: usize,
    pub vertices: usize,
    pub indices: usize,
}

#[derive(Resource, Debug, Default)]
pub struct Time {
    pub delta: Duration,
    pub elapsed: f64,
}

impl Time {
    pub fn delta_secs(&self) -> f32 {
        self.delta.as_secs_f32()
    }
    pub fn delta_secs_f64(&self) -> f64 {
        self.delta.as_secs_f64()
    }
    pub fn elapsed_secs(&self) -> f32 {
        self.elapsed as f32
    }
    pub fn elapsed_secs_f64(&self) -> f64 {
        self.elapsed
    }
}

#[derive(Resource, Debug, Default)]
pub struct MouseInput {
    pub just_pressed: HashSet<MouseButton>,
    pub just_released: HashSet<MouseButton>,
    pub pressed: HashSet<MouseButton>,
    pub position: Vec2,
    pub motion: Vec2,
    pub scroll: Vec2,
}

impl MouseInput {
    pub fn just_pressed(&self, key: MouseButton) -> bool {
        self.just_pressed.contains(&key)
    }

    pub fn just_released(&self, key: MouseButton) -> bool {
        self.just_released.contains(&key)
    }

    pub fn pressed(&self, key: MouseButton) -> bool {
        self.pressed.contains(&key)
    }
}

#[derive(Resource, Debug, Default)]
pub struct KeyboardInput {
    pub just_pressed: HashSet<Key>,
    pub just_released: HashSet<Key>,
    pub pressed: HashSet<Key>,
}

impl KeyboardInput {
    pub fn just_pressed(&self, key: Key) -> bool {
        self.just_pressed.contains(&key)
    }

    pub fn just_released(&self, key: Key) -> bool {
        self.just_released.contains(&key)
    }

    pub fn pressed(&self, key: Key) -> bool {
        self.pressed.contains(&key)
    }
}

#[derive(Default)]
pub struct Meshes(pub HashMap<usize, ChunkMesh>, pub usize);

impl Meshes {
    pub fn add(&mut self, mesh: ChunkMesh) -> Mesh3d {
        self.0.insert(self.1, mesh);
        let mesh_id = Mesh3d(self.1);
        self.1 += 1;
        mesh_id
    }
}

#[derive(Default)]
pub struct Materials(pub Vec<Material>);

impl Materials {
    pub fn add(&mut self, material: Material) -> MeshMaterial {
        self.0.push(material);
        MeshMaterial(self.0.len() - 1)
    }
}

#[derive(Resource)]
pub struct Skybox {
    pub texture_id: GLuint,
    pub vao: GLuint,
    pub vbo: GLuint,
}

#[derive(Component, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Mesh3d(pub usize);

#[derive(Component, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MeshMaterial(pub usize);

#[derive(Component, Debug, Clone, Copy)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }
}

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

#[derive(Component, Clone, Copy, PartialEq)]
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

    #[inline]
    pub fn looking_at(mut self, target: Vec3, up: Vec3) -> Self {
        self.look_at(target, up);
        self
    }

    #[inline]
    pub fn look_at(&mut self, target: Vec3, up: Vec3) {
        let direction = target + self.translation;
        self.look_to(direction, up);
    }

    #[inline]
    pub fn look_to(&mut self, direction: Vec3, up: Vec3) {
        let dir = direction.try_normalize().unwrap_or(Vec3::Z);

        let mut right = up.cross(dir).try_normalize();
        if right.is_none() {
            right = Some(up.any_orthonormal_vector());
        }
        let right = right.unwrap();
        let up = dir.cross(right);

        self.rotation = Quat::from_mat3(&Mat3::from_cols(right, up, dir));
    }
}

#[derive(ScheduleLabel, Hash, PartialEq, Eq, Debug, Clone)]
pub struct Startup;

#[derive(ScheduleLabel, Hash, PartialEq, Eq, Debug, Clone)]
pub struct PreUpdate;

#[derive(ScheduleLabel, Hash, PartialEq, Eq, Debug, Clone)]
pub struct Update;

#[derive(ScheduleLabel, Hash, PartialEq, Eq, Debug, Clone)]
pub struct FixedUpdate;

#[derive(ScheduleLabel, Hash, PartialEq, Eq, Debug, Clone)]
pub struct EguiContextPass;

#[derive(ScheduleLabel, Hash, PartialEq, Eq, Debug, Clone)]
pub struct RenderUpdate;

#[derive(ScheduleLabel, Hash, PartialEq, Eq, Debug, Clone)]
pub struct PostUpdate;

#[derive(ScheduleLabel, Hash, PartialEq, Eq, Debug, Clone)]
pub struct Exiting;
