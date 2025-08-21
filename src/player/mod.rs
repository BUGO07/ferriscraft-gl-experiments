use crate::{
    App, SEA_LEVEL,
    ecs::*,
    utils::set_cursor_grab,
    world::{NoiseFunctions, mesher::terrain_noise},
};

pub mod movement;

pub fn player_plugin(app: &mut App) {
    app.add_systems(Startup, setup)
        .add_systems(Update, movement::handle_movement);
}

pub fn setup(
    mut commands: Commands,
    mut materials: NonSendMut<Materials>,
    mut window: ResMut<Window>,
    ns_window: NonSend<NSWindow>,
    noises: Res<NoiseFunctions>,
) {
    set_cursor_grab(&mut window, true);
    let (height, _biome) = terrain_noise(vec2(0.0, 0.0), &noises);
    commands.spawn((
        Camera3d {
            fov: 60.0,
            near: 0.1,
            far: 1024.0,
        },
        Transform::from_xyz(0.0, height.max(SEA_LEVEL) as f32 + 5.0, 0.0),
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 1000.0,
        },
        Transform::DEFAULT.with_rotation(
            Quat::from_rotation_x(45_f32.to_radians())
                * Quat::from_rotation_y(-30_f32.to_radians()),
        ),
    ));

    // materials[0]
    materials.add(Material::new(&ns_window.facade, "voxel", Some("atlas.png")));
}
