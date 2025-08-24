use crate::{
    App,
    ecs::*,
    render::{
        material::{Material, MaterialOptions},
        mesh::Mesh,
    },
    utils::Quad,
};

pub fn ui_plugin(app: &mut App) {
    app.add_systems(Startup, setup.after(crate::player::setup));
}

fn setup(
    mut commands: Commands,
    mut materials: NonSendMut<Materials>,
    mut meshes: NonSendMut<Meshes>,
    ns_window: NonSend<NSWindow>,
) {
    let material = materials.add(Material::new("ui", MaterialOptions::default()).unwrap());

    commands.spawn(UIRect::new(
        Val::Percent(0.0),
        Val::Percent(0.0),
        Val::Px(80.0),
        Val::Px(80.0),
        material,
    ));

    commands.spawn(UIRect::new(
        Val::Percent(50.0),
        Val::Percent(50.0),
        Val::Percent(1.0),
        Val::Percent(1.0),
        material,
    ));
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
