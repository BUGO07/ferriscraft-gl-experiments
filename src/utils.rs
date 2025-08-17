use glam::*;

use crate::{
    CHUNK_SIZE,
    ecs::Aabb,
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
    pub corners: [[i32; 3]; 4],
}

impl Quad {
    #[inline]
    pub fn from_direction(direction: Direction, pos: IVec3, size: IVec3) -> Self {
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
}

#[inline]
pub fn vec3_to_index(pos: IVec3) -> usize {
    (pos.x + pos.y * CHUNK_SIZE + pos.z * CHUNK_SIZE * CHUNK_SIZE) as usize
}

#[inline]
pub fn index_to_vec3(index: usize) -> IVec3 {
    ivec3(
        index as i32 % CHUNK_SIZE,
        (index as i32 / CHUNK_SIZE) % CHUNK_SIZE,
        index as i32 / (CHUNK_SIZE * CHUNK_SIZE),
    )
}

pub fn should_cull(frustum: &[Vec4; 6], pos: Vec3, aabb: &Aabb) -> bool {
    for plane in frustum {
        let mut n_vertex = aabb.min + pos;
        if plane.x > 0.0 {
            n_vertex.x = aabb.max.x + pos.x;
        }
        if plane.y > 0.0 {
            n_vertex.y = aabb.max.y + pos.y;
        }
        if plane.z > 0.0 {
            n_vertex.z = aabb.max.z + pos.z;
        }

        if plane.xyz().dot(n_vertex) + plane.w < 0.0 {
            return true;
        }
    }
    false
}

pub fn frustum_planes(view_proj_matrix: &Mat4) -> [Vec4; 6] {
    let mut planes: [Vec4; 6] = [Vec4::ZERO; 6];

    let row1 = view_proj_matrix.row(0);
    let row2 = view_proj_matrix.row(1);
    let row3 = view_proj_matrix.row(2);
    let row4 = view_proj_matrix.row(3);

    let left = row4 + row1;
    planes[0] = left;

    let right = row4 - row1;
    planes[1] = right;

    let bottom = row4 + row2;
    planes[2] = bottom;

    let top = row4 - row2;
    planes[3] = top;

    let near = row4 + row3;
    planes[4] = near;

    let far = row4 - row3;
    planes[5] = far;

    planes
}
