use std::collections::HashMap;

use glam::{IVec3, Vec3, ivec3};

use crate::{
    CHUNK_HEIGHT, CHUNK_SIZE,
    mesher::{Block, Direction},
};

#[inline]
pub fn generate_block_at(pos: IVec3, max_y: i32) -> Block {
    let y = pos.y;
    if y == 0 {
        Block::Bedrock
    } else if y < max_y {
        match y {
            _ if y > 165 => Block::Snow,
            _ if y > 140 => Block::Stone,
            _ if y == max_y - 1 => Block::Grass,
            _ if y >= max_y - 4 => Block::Dirt,
            _ => Block::Stone,
        }
    } else {
        Block::Air
    }

    // let tree_probabilty = tree_noise(pos.xz().as_vec2(), seed);

    // if tree_probabilty > 0.85 && max_y < 90 && max_y > SEA_LEVEL + 2 {
    //     for (y, tree_layer) in TREE_OBJECT.iter().enumerate() {
    //         for (z, tree_row) in tree_layer.iter().enumerate() {
    //             for (x, block) in tree_row.iter().enumerate() {
    //                 let mut tree_pos = ivec3(3 + x as i32, y as i32, 3 + z as i32);
    //                 let (local_max_y, _) = terrain_noise((pos + tree_pos).as_vec3().xz(), seed);

    //                 tree_pos.y += local_max_y;

    //                 if pos == tree_pos {
    //                     return *block;
    //                 }
    //             }
    //         }
    //     }
    // }

    // terrain_block
}
pub struct Quad {
    pub corners: [[f32; 3]; 4],
}

impl Quad {
    #[inline]
    pub fn from_direction(direction: Direction, pos: Vec3, size: Vec3) -> Self {
        let corners = match direction {
            Direction::Left => [
                [pos.x, pos.y, pos.z],
                [pos.x, pos.y, pos.z + size.z],
                [pos.x, pos.y + size.y, pos.z + size.z],
                [pos.x, pos.y + size.y, pos.z],
            ],
            Direction::Right => [
                [pos.x, pos.y + size.y, pos.z],
                [pos.x, pos.y + size.y, pos.z + size.z],
                [pos.x, pos.y, pos.z + size.z],
                [pos.x, pos.y, pos.z],
            ],
            Direction::Bottom => [
                [pos.x, pos.y, pos.z],
                [pos.x + size.x, pos.y, pos.z],
                [pos.x + size.x, pos.y, pos.z + size.z],
                [pos.x, pos.y, pos.z + size.z],
            ],
            Direction::Top => [
                [pos.x, pos.y, pos.z + size.z],
                [pos.x + size.x, pos.y, pos.z + size.z],
                [pos.x + size.x, pos.y, pos.z],
                [pos.x, pos.y, pos.z],
            ],
            Direction::Back => [
                [pos.x, pos.y, pos.z],
                [pos.x, pos.y + size.y, pos.z],
                [pos.x + size.x, pos.y + size.y, pos.z],
                [pos.x + size.x, pos.y, pos.z],
            ],
            Direction::Front => [
                [pos.x + size.x, pos.y, pos.z],
                [pos.x + size.x, pos.y + size.y, pos.z],
                [pos.x, pos.y + size.y, pos.z],
                [pos.x, pos.y, pos.z],
            ],
        };

        Self { corners }
    }

    pub fn _translate(&mut self, offset: Vec3) {
        for corner in &mut self.corners {
            corner[0] += offset.x;
            corner[1] += offset.y;
            corner[2] += offset.z;
        }
    }
}

#[inline]
pub fn vec3_to_index(pos: IVec3) -> usize {
    (pos.x + pos.y * CHUNK_SIZE + pos.z * CHUNK_SIZE * CHUNK_HEIGHT) as usize
}

#[inline]
pub fn index_to_vec3(index: usize) -> IVec3 {
    ivec3(
        index as i32 % CHUNK_SIZE,
        (index as i32 / CHUNK_SIZE) % CHUNK_HEIGHT,
        index as i32 / (CHUNK_SIZE * CHUNK_HEIGHT),
    )
}
