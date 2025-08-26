use crate::{
    CHUNK_SIZE,
    ecs::*,
    utils::vec3_to_index,
    world::{
        ChunkMarker, WorldData,
        mesher::{Block, Chunk, Direction},
    },
};

pub fn place_block(
    chunk: &mut Chunk,
    pos: IVec3,
    block: Block,
    // saved_chunks: &mut Option<&mut HashMap<IVec3, SavedChunk>>,
    // client: Option<ResMut<RenetClient>>,
    update: Option<(&mut Commands, Vec<(Entity, &Transform)>)>,
) {
    chunk.blocks[vec3_to_index(pos)] = block;
    // if let Some(saved_chunks) = saved_chunks {
    //     saved_chunks
    //         .entry(chunk.pos)
    //         .and_modify(|c| {
    //             c.blocks.insert(pos, block);
    //         })
    //         .or_insert(SavedChunk {
    //             blocks: HashMap::from([(pos, block)]),
    //             // entities: chunk.entities.clone(),
    //         });
    // }
    if let Some((commands, chunks)) = update {
        let mut positions = vec![chunk.pos];
        if pos.x == 0 {
            positions.push(chunk.pos - IVec3::X);
        }
        if pos.x == CHUNK_SIZE - 1 {
            positions.push(chunk.pos + IVec3::X);
        }
        if pos.y == 0 {
            positions.push(chunk.pos - IVec3::Y);
        }
        if pos.y == CHUNK_SIZE - 1 {
            positions.push(chunk.pos + IVec3::Y);
        }
        if pos.z == 0 {
            positions.push(chunk.pos - IVec3::Z);
        }
        if pos.z == CHUNK_SIZE - 1 {
            positions.push(chunk.pos + IVec3::Z);
        }
        update_chunks(commands, chunks, positions);
    }
    // ClientPacket::PlaceBlock(chunk.pos * CHUNK_SIZE + pos, block).send(client);
}

pub fn update_chunks(
    commands: &mut Commands,
    chunks: Vec<(Entity, &Transform)>,
    positions: Vec<IVec3>,
) {
    for (entity, transform) in chunks {
        if positions.contains(&(transform.translation / CHUNK_SIZE as f32).as_ivec3()) {
            commands
                .entity(entity)
                .try_remove::<ChunkMarker>()
                .try_insert(ChunkMarker);
        }
    }
}

#[derive(Debug)]
pub struct RayHit {
    pub global_position: IVec3,
    pub chunk_pos: IVec3,
    pub local_pos: IVec3,
    pub normal: Direction,
    pub _block: Block,
    pub distance: f32,
}

pub fn ray_cast(
    world_data: &WorldData,
    ray_origin: Vec3,
    ray_direction: Vec3,
    max_distance: f32,
) -> Option<RayHit> {
    let ray_direction = ray_direction.normalize();

    let mut current_block_pos = ray_origin.floor();

    let step_x = ray_direction.x.signum();
    let step_y = ray_direction.y.signum();
    let step_z = ray_direction.z.signum();

    let t_delta_x = if ray_direction.x == 0.0 {
        f32::INFINITY
    } else {
        (1.0 / ray_direction.x).abs()
    };
    let t_delta_y = if ray_direction.y == 0.0 {
        f32::INFINITY
    } else {
        (1.0 / ray_direction.y).abs()
    };
    let t_delta_z = if ray_direction.z == 0.0 {
        f32::INFINITY
    } else {
        (1.0 / ray_direction.z).abs()
    };

    let mut t_max_x = if ray_direction.x >= 0.0 {
        (current_block_pos.x + 1.0 - ray_origin.x) / ray_direction.x
    } else {
        (current_block_pos.x - ray_origin.x) / ray_direction.x
    };
    if ray_direction.x == 0.0 {
        t_max_x = f32::INFINITY;
    }

    let mut t_max_y = if ray_direction.y >= 0.0 {
        (current_block_pos.y + 1.0 - ray_origin.y) / ray_direction.y
    } else {
        (current_block_pos.y - ray_origin.y) / ray_direction.y
    };
    if ray_direction.y == 0.0 {
        t_max_y = f32::INFINITY;
    }

    let mut t_max_z = if ray_direction.z >= 0.0 {
        (current_block_pos.z + 1.0 - ray_origin.z) / ray_direction.z
    } else {
        (current_block_pos.z - ray_origin.z) / ray_direction.z
    };
    if ray_direction.z == 0.0 {
        t_max_z = f32::INFINITY;
    }

    let mut current_distance = 0.0;
    let mut normal;

    while current_distance <= max_distance {
        if t_max_x < t_max_y && t_max_x < t_max_z {
            current_block_pos.x += step_x;
            current_distance = t_max_x;
            t_max_x += t_delta_x;
            normal = if step_x.is_sign_negative() {
                Direction::Right
            } else {
                Direction::Left
            };
        } else if t_max_y < t_max_z {
            current_block_pos.y += step_y;
            current_distance = t_max_y;
            t_max_y += t_delta_y;
            normal = if step_y.is_sign_negative() {
                Direction::Top
            } else {
                Direction::Bottom
            };
        } else {
            current_block_pos.z += step_z;
            current_distance = t_max_z;
            t_max_z += t_delta_z;
            normal = if step_z.is_sign_negative() {
                Direction::Front
            } else {
                Direction::Back
            };
        }

        if current_distance > max_distance {
            break;
        }

        let chunk_pos = ivec3(
            current_block_pos.x.div_euclid(CHUNK_SIZE as f32) as i32,
            current_block_pos.y.div_euclid(CHUNK_SIZE as f32) as i32,
            current_block_pos.z.div_euclid(CHUNK_SIZE as f32) as i32,
        );

        let local_block_pos = vec3(
            current_block_pos.x.rem_euclid(CHUNK_SIZE as f32),
            current_block_pos.y.rem_euclid(CHUNK_SIZE as f32),
            current_block_pos.z.rem_euclid(CHUNK_SIZE as f32),
        )
        .as_ivec3();

        if let Some(chunk) = world_data.chunks.read().unwrap().get(&chunk_pos) {
            let block_index = vec3_to_index(local_block_pos);

            if block_index < chunk.blocks.len() {
                let block = chunk.blocks[block_index];

                if block.is_solid() {
                    return Some(RayHit {
                        global_position: current_block_pos.as_ivec3(),
                        chunk_pos,
                        local_pos: local_block_pos,
                        normal,
                        _block: block,
                        distance: current_distance,
                    });
                }
            }
        }
    }

    None
}
