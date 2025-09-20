use glfw::MouseButton;

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
        .add_systems(Update, (movement::handle_movement, handle_interactions))
        .add_systems(FixedUpdate, update_projectiles);
}

pub fn setup(mut commands: Commands, mut window: ResMut<Window>, noises: Res<NoiseFunctions>) {
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

    commands.spawn(DirectionalLight {
        illuminance: 1000.0,
    });
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
        player.rotation * Vec3::NEG_Z, // == player.forward()
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
    } else if mouse.just_pressed(MouseButton::Right) {
        let direction = player.rotation * Vec3::NEG_Z;
        let speed = 50.0; // TODO change between 35-50 depending on how long the right click was held
        commands.spawn((
            Projectile {
                direction,
                velocity: direction * speed,
                lifespan: 60.0,
            },
            Transform::from_translation(player.translation),
        ));
    }
}

pub fn update_projectiles(
    mut commands: Commands,
    mut projectiles: Query<(Entity, &mut Transform, &mut Projectile)>,
    time: Res<Time<FixedTime>>,
    world_data: Res<WorldData>,
) {
    for (entity, mut transform, mut projectile) in projectiles.iter_mut() {
        projectile.lifespan -= time.delta_secs();
        if projectile.lifespan <= 0.0 {
            commands.entity(entity).despawn();
        }
        if projectile.velocity.length_squared() > 0.0 {
            projectile.velocity *= 0.995;

            projectile.velocity.y -= time.delta_secs() * 10.0;

            let new_pos = transform.translation + projectile.velocity * time.delta_secs();

            if let Some(hit) = ray_cast(
                &world_data,
                transform.translation,
                projectile.velocity.normalize_or_zero(),
                (new_pos - transform.translation).length() + 0.25,
            ) {
                transform.translation += hit.distance * projectile.velocity.normalize_or_zero();

                projectile.velocity = Vec3::ZERO;
            } else {
                transform.translation = new_pos;

                if projectile.velocity.length_squared() > 0.001 {
                    projectile.direction = projectile.velocity.normalize();
                }
            }
        }
    }
}

#[derive(Component)]
pub struct Projectile {
    pub direction: Vec3,
    pub velocity: Vec3,
    pub lifespan: f32, // secs
}
