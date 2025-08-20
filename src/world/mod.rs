use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, RwLock},
};

use bevy_ecs::{query::With, system::Single};
use bevy_tasks::{AsyncComputeTaskPool, Task, futures_lite::future};
use fastnoise2::generator::{Generator, GeneratorWrapper, simplex::Simplex};
use rayon::slice::ParallelSliceMut;

use crate::{
    App, CHUNK_SIZE,
    ecs::*,
    utils::{generate_block_at, vec3_to_index},
    world::mesher::{Chunk, ChunkMesh, VoxelVertex},
};

pub mod mesher;

pub fn world_plugin(app: &mut App) {
    app.init_resource::<GameInfo>()
        .init_resource::<NoiseFunctions>()
        .add_systems(
            Update,
            (
                handle_chunk_gen,
                handle_mesh_gen,
                handle_chunk_despawn,
                process_tasks,
            ),
        );
}

#[derive(Resource, Default)]
pub struct GameInfo {
    pub chunks: Arc<RwLock<HashMap<IVec3, Chunk>>>,
    pub loading_chunks: Arc<RwLock<HashSet<IVec3>>>,
}

#[derive(Resource, Clone, Copy, Default)]
pub struct NoiseFunctions {}

#[derive(Component)]
pub struct ComputeChunk(pub Task<Chunk>, pub IVec3);

#[derive(Component)]
pub struct ComputeChunkMesh(pub Task<Option<ChunkMesh>>, pub IVec3);

#[derive(Component)]
pub struct ChunkMarker;

fn handle_chunk_gen(
    mut commands: Commands,
    game_info: Res<GameInfo>,
    // noises: Res<NoiseFunctions>,
    player: Single<&Transform, With<Camera3d>>,
) {
    let pt = player.translation;
    let thread_pool = AsyncComputeTaskPool::get();
    let render_distance = 8;

    let mut chunks_to_load = Vec::new();

    for chunk_y in (pt.y as i32 / CHUNK_SIZE - render_distance).max(0)
        ..(pt.y as i32 / CHUNK_SIZE + render_distance)
    {
        for chunk_z in (pt.z as i32 / CHUNK_SIZE - render_distance)
            ..(pt.z as i32 / CHUNK_SIZE + render_distance)
        {
            for chunk_x in (pt.x as i32 / CHUNK_SIZE - render_distance)
                ..(pt.x as i32 / CHUNK_SIZE + render_distance)
            {
                let pos = ivec3(chunk_x, chunk_y, chunk_z);

                if let Ok(guard) = game_info.chunks.read() {
                    if guard.contains_key(&pos) {
                        continue;
                    }
                } else {
                    continue;
                };

                if let Ok(guard) = game_info.loading_chunks.read() {
                    if guard.contains(&pos) {
                        continue;
                    }
                } else {
                    continue;
                };

                {
                    game_info.loading_chunks.write().unwrap().insert(pos);
                }

                chunks_to_load.push(pos);

                // let chunks = game_info.chunks.clone();
                // let saved_chunks = game_info.saved_chunks.clone();

                let task = thread_pool.spawn(async move {
                    let mut chunk = Chunk::new(pos);

                    let noise = GeneratorWrapper(Simplex)
                        .fbm(
                            0.5, // gain (amplitude falloff per octave)
                            0.0, // weighted_strength (keep it 0.0 for smooth MC-style terrain)
                            4,   // octaves (enough detail without making it noisy)
                            2.0, // lacunarity (classic doubling frequency each octave)
                        )
                        .domain_scale(0.002) // controls feature size (lower = bigger terrain features)
                        .build();

                    for rela_z in 0..CHUNK_SIZE {
                        for rela_x in 0..CHUNK_SIZE {
                            let hpos = vec2(
                                (rela_x + pos.x * CHUNK_SIZE) as f32,
                                (rela_z + pos.z * CHUNK_SIZE) as f32,
                            );
                            let raw_noise = noise.gen_single_2d(hpos.x, hpos.y, 1337);
                            let max_y = (raw_noise * 32.0 + 64.0) as i32;

                            for rela_y in 0..CHUNK_SIZE {
                                chunk.blocks[vec3_to_index(ivec3(rela_x, rela_y, rela_z))] =
                                    generate_block_at(
                                        ivec3(
                                            hpos.x as i32,
                                            rela_y + pos.y * CHUNK_SIZE,
                                            hpos.y as i32,
                                        ),
                                        max_y,
                                    );

                                // if rela_y == max_y
                                //     && max_y > SEA_LEVEL
                                //     && biome < 0.4
                                //     && noise(noises.ferris, pos) > 0.85
                                // {
                                //     chunk.entities.push((
                                //         Entity::PLACEHOLDER,
                                //         GameEntity {
                                //             kind: GameEntityKind::Ferris,
                                //             pos: vec3(pos.x, rela_y as f32, pos.y),
                                //             rot: rand::random_range(0..360) as f32,
                                //         },
                                //     ));
                                // }
                            }

                            // let tree_probabilty = noise(noises.tree, pos);

                            // // TODO: clean up
                            // if tree_probabilty > 0.85 && max_y < 90 && max_y > SEA_LEVEL + 2 {
                            //     for (y, tree_layer) in TREE_OBJECT.iter().enumerate() {
                            //         for (z, tree_row) in tree_layer.iter().enumerate() {
                            //             for (x, &block) in tree_row.iter().enumerate() {
                            //                 let mut pos =
                            //                     ivec3(3 + x as i32, y as i32, 3 + z as i32);
                            //                 let (local_max_y, _) = terrain_noise(
                            //                     (chunk.pos * CHUNK_SIZE + pos).as_vec3().xz(),
                            //                     &noises,
                            //                 );

                            //                 pos.y += local_max_y;

                            //                 if (0..CHUNK_SIZE).contains(&pos.x)
                            //                     && (0..CHUNK_HEIGHT).contains(&pos.y)
                            //                     && (0..CHUNK_SIZE).contains(&pos.z)
                            //                 {
                            //                     chunk.blocks[vec3_to_index(pos)] = block;
                            //                 } else if let Some(relative_chunk) =
                            //                     chunk.get_relative_chunk(pos)
                            //                     && let Some(target) =
                            //                         chunks.write().unwrap().get_mut(&relative_chunk)
                            //                 {
                            //                     let block_index = vec3_to_index(
                            //                         pos - relative_chunk * CHUNK_SIZE,
                            //                     );
                            //                     if block_index < target.blocks.len() {
                            //                         target.blocks[block_index] = block;
                            //                     }
                            //                 }
                            //             }
                            //         }
                            //     }
                            // }
                        }
                    }

                    // if let Some(saved_chunks) = &saved_chunks
                    //     && let Some(saved_chunk) = saved_chunks.read().unwrap().get(&pos)
                    // {
                    //     for (&pos, &block) in &saved_chunk.blocks {
                    //         chunk.blocks[vec3_to_index(pos)] = block;
                    //     }
                    //     // chunk.entities = saved_chunk.entities.clone();
                    // }
                    chunk
                });
                commands.spawn(ComputeChunk(task, pos));
            }
        }
    }
}

fn handle_mesh_gen(
    mut commands: Commands,
    game_info: Res<GameInfo>,
    // noises: Res<NoiseFunctions>,
    query: Query<(Entity, &Transform), Added<ChunkMarker>>,
) {
    let thread_pool = AsyncComputeTaskPool::get();

    for (entity, transform) in query {
        let pos = transform.translation.as_ivec3() / CHUNK_SIZE;

        let chunks = game_info.chunks.clone();
        // let noises = *noises;

        let task = thread_pool.spawn(async move {
            let guard = chunks.read().unwrap();
            #[cfg(feature = "profile")]
            let instant = std::time::Instant::now();
            let mesh = ChunkMesh::default().build(guard.get(&pos)?, &guard);
            #[cfg(feature = "profile")]
            println!("Generated chunk in {:?}", instant.elapsed());
            mesh
        });

        commands
            .entity(entity)
            .try_insert(ComputeChunkMesh(task, pos));
    }
}

#[allow(clippy::type_complexity)]
fn handle_chunk_despawn(
    mut commands: Commands,
    game_info: Res<GameInfo>,
    query: Query<
        (Entity, &Transform),
        Or<(
            With<ChunkMarker>,
            With<ComputeChunkMesh>,
            With<ComputeChunk>,
        )>,
    >,
    player: Single<&Transform, With<Camera3d>>,
) {
    let pt = player.translation;
    let render_distance = 8;

    let mut chunks = game_info.chunks.write().unwrap();
    let mut loading_chunks = game_info.loading_chunks.write().unwrap();

    for (entity, transform) in query {
        let pos = transform.translation.as_ivec3() / CHUNK_SIZE;

        if (pos.x + render_distance < pt.x as i32 / CHUNK_SIZE)
            || (pos.x - render_distance > pt.x as i32 / CHUNK_SIZE)
            || (pos.y + render_distance < pt.y as i32 / CHUNK_SIZE)
            || (pos.y - render_distance > pt.y as i32 / CHUNK_SIZE)
            || (pos.z + render_distance < pt.z as i32 / CHUNK_SIZE)
            || (pos.z - render_distance > pt.z as i32 / CHUNK_SIZE)
        {
            {
                // if let Some(chunk_entities) = chunks.get(&pos) {
                //     for (entity, _) in &chunk_entities.entities {
                //         if *entity != Entity::PLACEHOLDER {
                //             commands.entity(*entity).try_despawn();
                //         }
                //     }
                // }
            }
            commands.entity(entity).try_despawn();

            chunks.remove(&pos);
            loading_chunks.remove(&pos);
        }
    }
}

fn process_tasks(
    mut commands: Commands,
    mut meshes: NonSendMut<Meshes<VoxelVertex>>,
    player: Single<&Transform, With<Camera3d>>,
    mesh_tasks: Query<(Entity, &mut ComputeChunkMesh)>,
    spawn_tasks: Query<(Entity, &mut ComputeChunk)>,
    game_info: Res<GameInfo>,
    ns_window: NonSend<NSWindow>,
) {
    // GENERATING CHUNKS
    let pt = player.translation.as_ivec3().with_y(0) / CHUNK_SIZE;

    let mut tasks = spawn_tasks.into_iter().collect::<Vec<_>>();
    tasks.par_sort_by_cached_key(|(_, x)| x.1.distance_squared(pt));

    let mut chunks = game_info.chunks.write().unwrap();
    // let mut saved_chunks = game_info
    //     .saved_chunks
    //     .as_ref()
    //     .map(|saved_chunks| saved_chunks.write().unwrap());
    let mut loading_chunks = game_info.loading_chunks.write().unwrap();

    let mut processed_this_frame = 0;
    for (entity, mut compute_task) in tasks {
        if processed_this_frame >= 15 {
            break;
        }
        if let Some(chunk) = future::block_on(future::poll_once(&mut compute_task.0)) {
            // if let Some(saved_chunks) = &mut saved_chunks {
            //     saved_chunks
            //         .entry(chunk.pos)
            //         .and_modify(|c| {
            //             if c.entities != chunk.entities {
            //                 c.entities = chunk.entities.clone();
            //             }
            //         })
            //         .or_insert(SavedChunk {
            //             entities: chunk.entities.clone(),
            //             ..default()
            //         });
            // }

            // for (e, game_entity) in &mut chunk.entities {
            //     *e = commands
            //         .spawn((
            //             *game_entity,
            //             SceneRoot(game_info.models[game_entity.kind as usize].clone()),
            //             Transform::from_translation(game_entity.pos + vec3(0.5, 0.0, 0.5))
            //                 .with_scale(Vec3::splat(2.0))
            //                 .with_rotation(Quat::from_rotation_y(game_entity.rot)),
            //         ))
            //         .id();
            // }
            commands
                .entity(entity)
                .try_insert((
                    ChunkMarker,
                    Aabb::new(
                        Vec3::ZERO,
                        vec3(CHUNK_SIZE as f32, CHUNK_SIZE as f32, CHUNK_SIZE as f32),
                    ),
                    Transform::from_translation((chunk.pos * CHUNK_SIZE).as_vec3()),
                ))
                .try_remove::<ComputeChunk>();

            loading_chunks.remove(&chunk.pos);
            chunks.insert(chunk.pos, chunk);

            processed_this_frame += 1;
        }
    }

    // GENERATING MESHES

    let mut tasks = mesh_tasks.into_iter().collect::<Vec<_>>();
    tasks.par_sort_by_cached_key(|(_, x)| x.1.distance_squared(pt));

    let mut processed_this_frame = 0;
    for (entity, mut compute_task) in tasks {
        if processed_this_frame >= 15 {
            break;
        }

        if let Some(result) = future::block_on(future::poll_once(&mut compute_task.0)) {
            commands.entity(entity).try_remove::<ComputeChunkMesh>();

            if let Some(mesh_data) = result {
                commands.entity(entity).try_insert((
                    meshes.add(
                        Mesh::new(mesh_data.vertices, mesh_data.indices),
                        &ns_window.facade,
                    ),
                    MeshMaterial(1), // MeshMaterial3d(game_info.materials[0].clone()),
                                     // Visibility::Visible,
                ));
            }
            processed_this_frame += 1;
        }
    }
}
