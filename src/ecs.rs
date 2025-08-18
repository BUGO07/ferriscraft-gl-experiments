use std::{collections::HashSet, time::Duration};

pub use bevy_ecs::{prelude::*, schedule::ScheduleLabel};
pub use glam::*;

use glium::{
    Display, IndexBuffer, Program, Texture2d, VertexBuffer,
    glutin::surface::WindowSurface,
    winit::{event::MouseButton, keyboard::KeyCode, window::CursorGrabMode},
};

use crate::mesher::VoxelVertex;

pub struct NSWindow {
    pub winit_window: glium::winit::window::Window,
    pub gl_context: Display<WindowSurface>,
}

#[derive(Resource)]
pub struct Window {
    pub cursor_grab: CursorGrabMode,
    pub cursor_visible: bool,
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
}

impl Time {
    pub fn delta_secs(&self) -> f32 {
        self.delta.as_secs_f32()
    }
}

#[derive(Resource, Debug, Default)]
pub struct MouseInput {
    pub just_pressesd: HashSet<MouseButton>,
    pub just_released: HashSet<MouseButton>,
    pub pressed: HashSet<MouseButton>,
    pub position: Vec2,
    pub motion: Vec2,
    pub scroll: Vec2,
}

impl MouseInput {
    pub fn just_pressed(&self, key: MouseButton) -> bool {
        self.just_pressesd.contains(&key)
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
    pub just_pressesd: HashSet<KeyCode>,
    pub just_released: HashSet<KeyCode>,
    pub pressed: HashSet<KeyCode>,
}

impl KeyboardInput {
    pub fn just_pressed(&self, key: KeyCode) -> bool {
        self.just_pressesd.contains(&key)
    }

    pub fn just_released(&self, key: KeyCode) -> bool {
        self.just_released.contains(&key)
    }

    pub fn pressed(&self, key: KeyCode) -> bool {
        self.pressed.contains(&key)
    }
}

#[derive(Debug, Default)]
pub struct Meshes(pub Vec<Mesh>);

impl Meshes {
    pub fn add(&mut self, mesh: Mesh) -> Mesh3d {
        self.0.push(mesh);
        Mesh3d(self.0.len() - 1)
    }
}

#[derive(Debug)]
pub struct Mesh {
    pub vertex_buffer: VertexBuffer<VoxelVertex>,
    pub index_buffer: IndexBuffer<u32>,
}

impl Mesh {
    pub fn new(vertex_buffer: VertexBuffer<VoxelVertex>, index_buffer: IndexBuffer<u32>) -> Self {
        Self {
            vertex_buffer,
            index_buffer,
        }
    }
}

#[derive(Debug, Default)]
pub struct Materials(pub Vec<Material>);

impl Materials {
    pub fn add(&mut self, material: Material) -> MeshMaterial {
        self.0.push(material);
        MeshMaterial(self.0.len() - 1)
    }
}

#[derive(Debug)]
pub struct Material {
    pub program: Program,
    pub texture: Texture2d,
}

impl Material {
    pub fn new(program: Program, texture: Texture2d) -> Self {
        Self { program, texture }
    }
}

#[derive(Component, Clone, Copy)]
pub struct Mesh3d(pub usize);

#[derive(Component, Clone, Copy)]
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

pub enum Val {
    Percent(f32),
    Px(f32),
}

impl Val {
    pub fn as_f32(&mut self) -> &mut f32 {
        match self {
            Val::Percent(p) => p,
            Val::Px(p) => p,
        }
    }
    pub fn calculate(&self, size: f32) -> f32 {
        match self {
            Val::Percent(p) => p / 100.0 * 2.0,
            Val::Px(p) => p / size * 2.0,
        }
    }
}

#[derive(Component)]
pub struct UIRect {
    pub x: Val,
    pub y: Val,
    pub width: Val,
    pub height: Val,
    pub color: Vec4,
}

impl UIRect {
    pub fn new(x: Val, y: Val, width: Val, height: Val, color: Vec4) -> Self {
        Self {
            x,
            y,
            width,
            height,
            color,
        }
    }
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

    #[inline]
    pub fn forward(&self) -> Vec3 {
        self.rotation * Vec3::NEG_Z
    }
    #[inline]
    pub fn right(&self) -> Vec3 {
        self.rotation * Vec3::X
    }
    #[inline]
    pub fn up(&self) -> Vec3 {
        self.rotation * Vec3::Y
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

#[derive(ScheduleLabel, Hash, PartialEq, Eq, Debug, Clone)]
pub struct PostUpdate;
