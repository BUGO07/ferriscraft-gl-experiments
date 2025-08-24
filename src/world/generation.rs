use bevy_tasks::{AsyncComputeTaskPool, futures_lite::future};
use rayon::slice::ParallelSliceMut;

use crate::{
    CHUNK_SIZE,
    ecs::*,
    utils::{generate_block_at, vec3_to_index},
    world::{
        ChunkMarker, ComputeChunk, ComputeChunkMesh, NoiseFunctions, WorldData,
        mesher::{Chunk, ChunkMesh, terrain_noise},
    },
};

pub fn handle_chunk_gen(
    mut commands: Commands,
    world_data: Res<WorldData>,
    noises: Res<NoiseFunctions>,
    player: Single<&Transform, With<Camera3d>>,
) {
    let thread_pool = AsyncComputeTaskPool::get();
    let render_distance = 8;

    let mut chunks_to_load = Vec::new();
    let player_chunk = player.translation.as_ivec3() / CHUNK_SIZE;

    for chunk_y in (player_chunk.y - render_distance).max(0)..(player_chunk.y + render_distance) {
        for chunk_z in (player_chunk.z - render_distance)..(player_chunk.z + render_distance) {
            for chunk_x in (player_chunk.x - render_distance)..(player_chunk.x + render_distance) {
                let pos = ivec3(chunk_x, chunk_y, chunk_z);

                if let Ok(guard) = world_data.chunks.read() {
                    if guard.contains_key(&pos) {
                        continue;
                    }
                } else {
                    continue;
                };

                if let Ok(guard) = world_data.loading_chunks.read() {
                    if guard.contains(&pos) {
                        continue;
                    }
                } else {
                    continue;
                };

                {
                    world_data.loading_chunks.write().unwrap().insert(pos);
                }

                chunks_to_load.push(pos);

                // let chunks = world_data.chunks.clone();
                // let saved_chunks = world_data.saved_chunks.clone();
                let noises = noises.clone();

                let task = thread_pool.spawn(async move {
                    let mut chunk = Chunk::new(pos);

                    for rela_z in 0..CHUNK_SIZE {
                        for rela_x in 0..CHUNK_SIZE {
                            let hpos = vec2(
                                (rela_x + pos.x * CHUNK_SIZE) as f32,
                                (rela_z + pos.z * CHUNK_SIZE) as f32,
                            );
                            let (max_y, _biome) = terrain_noise(hpos, &noises);

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

pub fn handle_mesh_gen(
    mut commands: Commands,
    world_data: Res<WorldData>,
    noises: Res<NoiseFunctions>,
    query: Query<(Entity, &Transform), Added<ChunkMarker>>,
) {
    let thread_pool = AsyncComputeTaskPool::get();

    for (entity, transform) in query {
        let pos = transform.translation.as_ivec3() / CHUNK_SIZE;

        let chunks = world_data.chunks.clone();
        let noises = noises.clone();

        let task = thread_pool.spawn(async move {
            let guard = chunks.read().unwrap();
            #[cfg(feature = "profile")]
            let instant = std::time::Instant::now();
            let mesh = ChunkMesh::build(guard.get(&pos)?, &guard, &noises);
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
pub fn handle_chunk_despawn(
    mut commands: Commands,
    mut meshes: NonSendMut<Meshes>,
    world_data: Res<WorldData>,
    query: Query<
        (Entity, &Transform, Option<&Mesh3d>),
        Or<(
            With<ChunkMarker>,
            With<ComputeChunkMesh>,
            With<ComputeChunk>,
        )>,
    >,
    player: Single<&Transform, With<Camera3d>>,
) {
    let player_chunk = player.translation.as_ivec3() / CHUNK_SIZE;
    let render_distance = 8;

    let mut chunks = world_data.chunks.write().unwrap();
    let mut loading_chunks = world_data.loading_chunks.write().unwrap();

    for (entity, transform, mesh_id) in query {
        let chunk_pos = transform.translation.as_ivec3() / CHUNK_SIZE;

        if (chunk_pos.x + render_distance < player_chunk.x)
            || (chunk_pos.x - render_distance > player_chunk.x)
            || (chunk_pos.y + render_distance < player_chunk.y)
            || (chunk_pos.y - render_distance > player_chunk.y)
            || (chunk_pos.z + render_distance < player_chunk.z)
            || (chunk_pos.z - render_distance > player_chunk.z)
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
            if let Some(mesh_id) = mesh_id {
                meshes.0.remove(&mesh_id.0);
            }
            commands.entity(entity).try_despawn();

            loading_chunks.remove(&chunk_pos);
            chunks.remove(&chunk_pos);
        }
    }
}

pub fn process_tasks(
    mut commands: Commands,
    mut meshes: NonSendMut<Meshes>,
    player: Single<&Transform, With<Camera3d>>,
    mesh_tasks: Query<(Entity, &mut ComputeChunkMesh)>,
    spawn_tasks: Query<(Entity, &mut ComputeChunk)>,
    world_data: Res<WorldData>,
    ns_window: NonSend<NSWindow>,
) {
    // GENERATING CHUNKS
    let pt = player.translation.as_ivec3().with_y(0) / CHUNK_SIZE;

    let mut tasks = spawn_tasks.into_iter().collect::<Vec<_>>();
    tasks.par_sort_by_cached_key(|(_, x)| x.1.distance_squared(pt));

    let mut chunks = world_data.chunks.write().unwrap();
    // let mut saved_chunks = world_data
    //     .saved_chunks
    //     .as_ref()
    //     .map(|saved_chunks| saved_chunks.write().unwrap());
    let mut loading_chunks = world_data.loading_chunks.write().unwrap();

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
            //             SceneRoot(world_data.models[game_entity.kind as usize].clone()),
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
                    meshes.add(mesh_data),
                    MeshMaterial(0), // MeshMaterial3d(world_data.materials[0].clone()),
                                     // Visibility::Visible,
                ));
            }
            processed_this_frame += 1;
        }
    }
}
