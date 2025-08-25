use crate::{
    App,
    ecs::*,
    render::material::{Material, MaterialOptions},
};

pub fn ui_plugin(app: &mut App) {
    app.add_systems(Startup, setup.after(crate::player::setup));
}

fn setup(mut commands: Commands, mut materials: NonSendMut<Materials>) {
    let material = materials.add(
        Material::new(
            "ui",
            MaterialOptions {
                base_texture: Some("assets/fonts/font.png"),
                base_color: Some(Vec4::new(1.0, 0.0, 0.0, 1.0)),
            },
        )
        .unwrap(),
    );

    commands.spawn(UIText::new(
        Val::Percent(0.0),
        Val::Percent(0.0),
        Val::Px(8.0 * 1.5),
        Val::Px(16.0 * 1.5),
        material,
        "PEAK GAME".to_string(),
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
    pub width: Val,
    pub height: Val,
    pub material: MeshMaterial,
    pub text: String,
}

impl UIText {
    pub fn new(
        x: Val,
        y: Val,
        width: Val,
        height: Val,
        material: MeshMaterial,
        text: String,
    ) -> Self {
        Self {
            x,
            y,
            width,
            height,
            material,
            text,
        }
    }
}
