#![allow(dead_code)]

use gl::types::*;

use crate::{
    App,
    ecs::*,
    render::{
        material::{Material, MaterialOptions},
        mesh::Vertex,
    },
};

pub mod update;

pub fn ui_plugin(app: &mut App) {
    app.add_systems(Startup, setup.after(crate::player::setup))
        .add_systems(Update, update::update_ui);
}

#[derive(Component)]
pub struct DebugText;

fn setup(mut commands: Commands, mut materials: NonSendMut<Materials>) {
    let material = materials.add(
        Material::new(
            "text",
            MaterialOptions {
                base_texture: Some("assets/fonts/minogram_6x10.png"),
                base_color: Some(Vec4::new(1.0, 0.0, 0.0, 1.0)),
            },
        )
        .unwrap(),
    );

    commands.spawn((
        UIText::new(
            Val::Percent(0.0),
            Val::Percent(0.0),
            Val::Px(6.0 * 3.0),
            Val::Px(10.0 * 3.0),
            material,
            "f3 or something".to_string(),
        ),
        DebugText,
    ));

    // commands.spawn(UIRect::new(
    //     Val::Percent(50.0),
    //     Val::Percent(50.0),
    //     Val::Percent(1.0),
    //     Val::Percent(1.0),
    //     material,
    // ));
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
    pub material: MeshMaterial,
}

impl UIRect {
    pub fn new(x: Val, y: Val, width: Val, height: Val, material: MeshMaterial) -> Self {
        Self {
            x,
            y,
            width,
            height,
            material,
        }
    }
}

#[derive(Component)]
pub struct UIText {
    pub x: Val,
    pub y: Val,
    pub font_size: Val,
    pub font_height: Val,
    pub text: String,
    pub material: MeshMaterial,
}

impl UIText {
    pub fn new(
        x: Val,
        y: Val,
        font_size: Val,
        font_height: Val,
        material: MeshMaterial,
        text: String,
    ) -> Self {
        Self {
            x,
            y,
            font_size,
            font_height,
            text,
            material,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct TextVertex {
    pub position: [f32; 2],
    pub char_id: u32,
    // size: [f32; 2],
}

impl Vertex for TextVertex {
    fn attributes() -> &'static [(GLuint, GLint, GLenum, GLboolean, usize)] {
        &[
            (0, 2, gl::FLOAT, gl::FALSE, 0),
            (1, 1, gl::UNSIGNED_INT, gl::FALSE, size_of::<[f32; 2]>()),
            // (2, 2, gl::FLOAT, gl::FALSE, size_of::<[f32; 2]>()),
        ]
    }
}
