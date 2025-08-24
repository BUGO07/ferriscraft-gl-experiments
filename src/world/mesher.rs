use std::collections::HashMap;

use gl::types::GLuint;
use glam::*;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{
    CHUNK_SIZE, SEA_LEVEL,
    utils::{Quad, generate_block_at, index_to_vec3, vec3_to_index},
    world::NoiseFunctions,
};

#[derive(Clone)]
pub struct Chunk {
    pub pos: IVec3,
    pub blocks: Vec<Block>,
}

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum Block {
    #[default]
    Air,
    Stone,
    Dirt,
    Grass,
    Plank,
    Bedrock,
    Water,
    Sand,
    Wood,
    Leaf,
    Snow,
}

impl Block {
    pub fn is_air(&self) -> bool {
        matches!(self, Block::Air)
    }
    pub fn is_solid(&self) -> bool {
        !matches!(self, Block::Air | Block::Water)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum Direction {
    Left,
    Right,
    Bottom,
    #[default]
    Top,
    Back,
    Front,
}

impl Direction {
    const NORMALS: [[i32; 3]; 6] = [
        [-1, 0, 0],
        [1, 0, 0],
        [0, -1, 0],
        [0, 1, 0],
        [0, 0, -1],
        [0, 0, 1],
    ];
    pub fn as_ivec3(self) -> IVec3 {
        IVec3::from(Self::NORMALS[self as usize])
    }
}

#[derive(Debug, Default)]
pub struct ChunkMesh {
    pub vertices: Vec<GLuint>,
    pub indices: Vec<GLuint>,
}

impl ChunkMesh {
    pub fn build(
        chunk: &Chunk,
        chunks: &HashMap<IVec3, Chunk>,
        noises: &NoiseFunctions,
    ) -> Option<Self> {
        let chunk_pos = chunk.pos;

        let left_chunk = chunks.get(&(chunk_pos + IVec3::new(-1, 0, 0)));
        let back_chunk = chunks.get(&(chunk_pos + IVec3::new(0, 0, -1)));
        let down_chunk = chunks.get(&(chunk_pos + IVec3::new(0, -1, 0)));

        // parallelized (thanks rayon)
        let mesh_parts = (0..CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE)
            .into_par_iter()
            .filter_map(|i| {
                let mut local_mesh = ChunkMesh::default();

                let pos = index_to_vec3(i as usize);

                let current = *unsafe { chunk.blocks.get_unchecked(i as usize) };

                let (back, left, down) =
                    chunk.get_adjacent_blocks(pos, left_chunk, back_chunk, down_chunk, noises);

                // TODO fix this so water works properly
                if !current.is_air() {
                    if left.is_air() {
                        local_mesh.push_face(chunk, Direction::Left, pos, current, noises);
                    }
                    if back.is_air() {
                        local_mesh.push_face(chunk, Direction::Back, pos, current, noises);
                    }
                    if down.is_air() {
                        local_mesh.push_face(chunk, Direction::Bottom, pos, current, noises);
                    }
                } else {
                    if !left.is_air() {
                        local_mesh.push_face(chunk, Direction::Right, pos, left, noises);
                    }
                    if !back.is_air() {
                        local_mesh.push_face(chunk, Direction::Front, pos, back, noises);
                    }
                    if !down.is_air() {
                        local_mesh.push_face(chunk, Direction::Top, pos, down, noises);
                    }
                }

                if local_mesh.vertices.is_empty() {
                    None
                } else {
                    Some(local_mesh)
                }
            })
            .collect::<Vec<_>>();

        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        for part in mesh_parts {
            vertices.extend(part.vertices);
        }

        if vertices.is_empty() {
            None
        } else {
            vertices.shrink_to_fit();
            indices.extend((0..vertices.len()).step_by(4).flat_map(|i| {
                let idx = i as u32;
                [idx, idx + 1, idx + 2, idx, idx + 2, idx + 3]
            }));

            Some(Self { vertices, indices })
        }
    }

    #[inline(always)]
    pub fn push_face(
        &mut self,
        chunk: &Chunk,
        dir: Direction,
        pos: IVec3,
        block: Block,
        noises: &NoiseFunctions,
    ) {
        let ambient_corners = chunk.ambient_corner_voxels(dir, pos, noises);
        for (i, pos) in Quad::from_direction(dir, pos.as_vec3(), Vec3::ONE)
            .corners
            .iter()
            .enumerate()
        {
            let index = i * 2;

            let side_1 = ambient_corners[index] as u8;
            let side_2 = ambient_corners[(index + 2) % 8] as u8;
            let side_corner = ambient_corners[(index + 1) % 8] as u8;
            let mut ao_count = side_1 + side_2 + side_corner;
            if side_1 == 1 && side_2 == 1 {
                ao_count = 3;
            }

            self.vertices.push(
                pos[0] as u32
                    | (pos[1] as u32) << 6
                    | (pos[2] as u32) << 12
                    | (dir as u32) << 18
                    | (ao_count as u32) << 21
                    | (block as u32) << 23,
            );
        }
    }
}

impl Chunk {
    #[inline]
    pub fn new(pos: IVec3) -> Self {
        Chunk {
            pos,
            blocks: vec![Block::Air; (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) as usize],
        }
    }

    fn get_relative_block(
        &self,
        relative_pos: IVec3,
        fallback: Option<&Chunk>,
        noises: &NoiseFunctions,
    ) -> Block {
        let (nx, ny, nz) = (relative_pos.x, relative_pos.y, relative_pos.z);
        if (0..CHUNK_SIZE).contains(&nx)
            && (0..CHUNK_SIZE).contains(&ny)
            && (0..CHUNK_SIZE).contains(&nz)
        {
            return *unsafe {
                self.blocks
                    .get_unchecked(vec3_to_index(IVec3::new(nx, ny, nz)))
            };
        }

        let mut chunk_x = self.pos.x;
        let mut chunk_y = self.pos.y;
        let mut chunk_z = self.pos.z;
        let mut lx = nx;
        let mut ly = ny;
        let mut lz = nz;

        if nx < 0 {
            lx += CHUNK_SIZE;
            chunk_x -= 1;
        } else if nx >= CHUNK_SIZE {
            lx -= CHUNK_SIZE;
            chunk_x += 1;
        }

        if ny < 0 {
            ly += CHUNK_SIZE;
            chunk_y -= 1;
        } else if ny >= CHUNK_SIZE {
            ly -= CHUNK_SIZE;
            chunk_y += 1;
        }

        if nz < 0 {
            lz += CHUNK_SIZE;
            chunk_z -= 1;
        } else if nz >= CHUNK_SIZE {
            lz -= CHUNK_SIZE;
            chunk_z += 1;
        }

        if let Some(chunk) = fallback {
            return *unsafe {
                chunk
                    .blocks
                    .get_unchecked(vec3_to_index(IVec3::new(lx, ly, lz)))
            };
        }

        let world_pos = IVec3::new(
            chunk_x * CHUNK_SIZE + lx,
            chunk_y * CHUNK_SIZE + ly,
            chunk_z * CHUNK_SIZE + lz,
        );
        let (max_y, _biome) = terrain_noise(world_pos.xz().as_vec2(), noises);
        generate_block_at(world_pos, max_y)
    }

    #[inline(always)]
    pub fn get_adjacent_blocks(
        &self,
        pos: IVec3,
        left_chunk: Option<&Chunk>,
        back_chunk: Option<&Chunk>,
        down_chunk: Option<&Chunk>,
        noises: &NoiseFunctions,
    ) -> (Block, Block, Block) {
        let back = self.get_relative_block(pos - IVec3::Z, back_chunk, noises);
        let left = self.get_relative_block(pos - IVec3::X, left_chunk, noises);
        let down = self.get_relative_block(pos - IVec3::Y, down_chunk, noises);

        (back, left, down)
    }

    pub fn ambient_corner_voxels(
        &self,
        dir: Direction,
        pos: IVec3,
        noises: &NoiseFunctions,
    ) -> [bool; 8] {
        #[rustfmt::skip]
        let positions = match dir {
            Direction::Left => [ivec3(-1,0,-1),ivec3(-1,-1,-1),ivec3(-1,-1,0),ivec3(-1,-1,1),ivec3(-1,0,1),ivec3(-1,1,1),ivec3(-1, 1, 0),ivec3(-1,1,-1),],
            Direction::Bottom => [ivec3(-1, -1, 0),ivec3(-1, -1, -1),ivec3(0, -1, -1), ivec3(1,-1,-1),ivec3(1,-1,0),ivec3(1, -1, 1),ivec3(0,-1,1),ivec3(-1,-1,1),],
            Direction::Back => [ivec3(0,-1,-1),ivec3(-1,-1,-1),ivec3(-1,0,-1),ivec3(-1,1,-1), ivec3(0,1,-1), ivec3(1,1,-1),ivec3(1,0,-1), ivec3(1,-1,-1)],

            Direction::Right => [ivec3(0,0,-1), ivec3(0,1,-1), ivec3(0,1,0), ivec3(0,1,1),ivec3(0,0,1),ivec3(0,-1,1),ivec3(0,-1,0),ivec3(0,-1,-1)],
            Direction::Top => [ivec3(-1,0,0),ivec3(-1,0,1),ivec3(0,0,1),ivec3(1,0,1),ivec3(1,0,0),ivec3(1,0,-1),ivec3(0,0,-1),ivec3(-1,0,-1),],
            Direction::Front => [ivec3(0,-1,0),ivec3(1,-1,0),ivec3(1,0,0),ivec3(1,1,0),ivec3(0,1,0),ivec3(-1,1,0),ivec3(-1,0,0),ivec3(-1,-1,0),],
        };

        let mut result = [false; 8];
        for i in 0..8 {
            result[i] = !self
                .get_relative_block(pos + positions[i], None, noises)
                .is_air();
        }
        result
    }
}

const OCEAN_MIN_HEIGHT: f32 = SEA_LEVEL as f32 - 40.0;
const OCEAN_MAX_HEIGHT: f32 = SEA_LEVEL as f32 + 5.0;
const OCEAN_FLATTENING_EXPONENT: f32 = 4.0;
const PLAINS_MIN_HEIGHT: f32 = SEA_LEVEL as f32 + 10.0;
const PLAINS_MAX_HEIGHT: f32 = SEA_LEVEL as f32 + 40.0;
const PLAINS_FLATTENING_EXPONENT: f32 = 3.0;
const MOUNTAIN_MIN_HEIGHT: f32 = SEA_LEVEL as f32 + 50.0;
const MOUNTAIN_MAX_HEIGHT: f32 = SEA_LEVEL as f32 + 180.0;
const MOUNTAIN_FLATTENING_EXPONENT: f32 = 1.5;
const OCEAN_PLAINS_THRESHOLD: f32 = 0.4;
const PLAINS_MOUNTAIN_THRESHOLD: f32 = 0.6;

// TODO make this better
#[inline]
// max_y, biome
pub fn terrain_noise(pos: Vec2, noises: &NoiseFunctions) -> (i32, f32) {
    let terrain_fbm = (noises.terrain.gen_single_2d(pos.x, pos.y, noises.seed) + 1.0) / 2.0;
    let biome_fbm = (noises.biome.gen_single_2d(pos.x, pos.y, noises.seed + 1) + 1.0) / 2.0;

    let min_height: f32;
    let max_height: f32;
    let flattening_exp: f32;

    if biome_fbm < OCEAN_PLAINS_THRESHOLD {
        let t = biome_fbm / OCEAN_PLAINS_THRESHOLD;
        min_height = OCEAN_MIN_HEIGHT.lerp(PLAINS_MIN_HEIGHT, t);
        max_height = OCEAN_MAX_HEIGHT.lerp(PLAINS_MAX_HEIGHT, t);
        flattening_exp = OCEAN_FLATTENING_EXPONENT.lerp(PLAINS_FLATTENING_EXPONENT, t);
    } else if biome_fbm < PLAINS_MOUNTAIN_THRESHOLD {
        let t = (biome_fbm - OCEAN_PLAINS_THRESHOLD)
            / (PLAINS_MOUNTAIN_THRESHOLD - OCEAN_PLAINS_THRESHOLD);
        min_height = PLAINS_MIN_HEIGHT.lerp(MOUNTAIN_MIN_HEIGHT, t);
        max_height = PLAINS_MAX_HEIGHT.lerp(MOUNTAIN_MAX_HEIGHT, t);
        flattening_exp = PLAINS_FLATTENING_EXPONENT.lerp(MOUNTAIN_FLATTENING_EXPONENT, t);
    } else {
        min_height = MOUNTAIN_MIN_HEIGHT;
        max_height = MOUNTAIN_MAX_HEIGHT;
        flattening_exp = MOUNTAIN_FLATTENING_EXPONENT;
    }

    let height = min_height + terrain_fbm.powf(flattening_exp) * (max_height - min_height);

    (height as i32, biome_fbm)
}
