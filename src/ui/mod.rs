use crate::{App, ecs::*, utils::Quad};

pub fn ui_plugin(app: &mut App) {
    app.add_systems(Startup, setup.after(crate::player::setup));
}

fn setup(
    mut commands: Commands,
    mut materials: NonSendMut<Materials>,
    mut ui_meshes: NonSendMut<Meshes<UIVertex>>,
    ns_window: NonSend<NSWindow>,
) {
    let ui_material = materials.add(Material::new(&ns_window.facade, "ui", None));

    commands.spawn(UIRect::new(
        Val::Percent(0.0),
        Val::Percent(0.0),
        Val::Px(80.0),
        Val::Px(80.0),
        ui_material,
    ));

    commands.spawn(UIRect::new(
        Val::Percent(50.0),
        Val::Percent(50.0),
        Val::Percent(1.0),
        Val::Percent(1.0),
        ui_material,
    ));

    let verts = Quad::DEFAULT
        .corners
        .iter()
        .map(|_| UIVertex::default())
        .collect::<Vec<_>>();

    let inds = (0..verts.len())
        .step_by(4)
        .flat_map(|i| {
            let idx = i as u32;
            [idx, idx + 1, idx + 2, idx, idx + 2, idx + 3]
        })
        .collect::<Vec<_>>();

    ui_meshes.add(Mesh::new(verts, inds), &ns_window.facade);
}

#[derive(Clone, Copy, Debug, Default)]
pub struct UIVertex {
    dummy: u8, // unneeded
}

implement_vertex!(UIVertex, dummy);

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
