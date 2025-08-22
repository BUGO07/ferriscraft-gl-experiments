use glium::winit::event::MouseButton;

use crate::{
    App, CHUNK_SIZE, SEA_LEVEL,
    ecs::*,
    utils::set_cursor_grab,
    world::{
        ChunkMarker, NoiseFunctions, WorldData,
        interaction::{place_block, ray_cast},
        mesher::{Block, terrain_noise},
    },
};

pub mod movement;

pub fn player_plugin(app: &mut App) {
    app.add_systems(Startup, setup)
        .add_systems(Update, (movement::handle_movement, handle_interactions));
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

fn handle_interactions(
    mut commands: Commands,
    player: Single<&Transform, With<Camera3d>>,
    mouse: Res<MouseInput>,
    chunks: Query<(Entity, &Transform), With<ChunkMarker>>,
    world_data: Res<WorldData>,
) {
    if let Some(hit) = ray_cast(
        &world_data,
        player.translation,
        player.rotation * Vec3::NEG_Z,
        5.0,
    ) {
        let mut local_pos = hit.local_pos;
        let mut chunk_pos = hit.chunk_pos;
        if mouse.just_pressed(MouseButton::Left)
            && let Some(chunk) = world_data.chunks.write().unwrap().get_mut(&chunk_pos)
        {
            place_block(
                chunk,
                local_pos,
                Block::Air,
                Some((&mut commands, chunks.iter().collect())),
            );
        } else if mouse.just_pressed(MouseButton::Right) {
            local_pos += hit.normal.as_ivec3();

            if local_pos.x < 0 {
                local_pos.x += CHUNK_SIZE;
                chunk_pos.x -= 1;
            } else if local_pos.x >= CHUNK_SIZE {
                local_pos.x -= CHUNK_SIZE;
                chunk_pos.x += 1;
            }
            if local_pos.y < 0 {
                local_pos.y += CHUNK_SIZE;
                chunk_pos.y -= 1;
            } else if local_pos.y >= CHUNK_SIZE {
                local_pos.y -= CHUNK_SIZE;
                chunk_pos.y += 1;
            }
            if local_pos.z < 0 {
                local_pos.z += CHUNK_SIZE;
                chunk_pos.z -= 1;
            } else if local_pos.z >= CHUNK_SIZE {
                local_pos.z -= CHUNK_SIZE;
                chunk_pos.z += 1;
            }
            if let Some(chunk) = world_data.chunks.write().unwrap().get_mut(&chunk_pos) {
                place_block(
                    chunk,
                    local_pos,
                    Block::Stone,
                    Some((&mut commands, chunks.iter().collect())),
                );
            }
        }
    }
}
