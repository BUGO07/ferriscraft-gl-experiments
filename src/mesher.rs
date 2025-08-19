use std::collections::HashMap;

use glam::*;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{
    CHUNK_SIZE,
    utils::{Quad, index_to_vec3, vec3_to_index},
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
    fn is_air(&self) -> bool {
        matches!(self, Block::Air)
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

#[derive(Default)]
pub struct ChunkMesh {
    pub vertices: Vec<VoxelVertex>,
    pub indices: Vec<u32>,
}

#[derive(Clone, Copy, Debug)]
pub struct VoxelVertex {
    pub vertex_data: u32,
}

implement_vertex!(VoxelVertex, vertex_data);

#[derive(Clone, Copy, Debug)]
pub struct UIVertex {
    pub pos: [f32; 2],
}

implement_vertex!(UIVertex, pos);

impl ChunkMesh {
    pub fn build(mut self, chunk: &Chunk, chunks: &HashMap<IVec3, Chunk>) -> Option<Self> {
        let chunk_pos = chunk.pos;

        let left_chunk = chunks.get(&(chunk_pos + IVec3::new(-1, 0, 0)));
        let back_chunk = chunks.get(&(chunk_pos + IVec3::new(0, 0, -1)));
        let down_chunk = chunks.get(&(chunk_pos + IVec3::new(0, -1, 0)));

        // parallelized (thanks rayon)
        let mesh_parts: Vec<ChunkMesh> = (0..CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE)
            .into_par_iter()
            .filter_map(|i| {
                let mut local_mesh = ChunkMesh::default();

                let pos = index_to_vec3(i as usize);

                let current = *unsafe { chunk.blocks.get_unchecked(i as usize) };

                let (back, left, down) =
                    chunk.get_adjacent_blocks(pos, left_chunk, back_chunk, down_chunk);

                if !current.is_air() {
                    if left.is_air() {
                        local_mesh.push_face(Direction::Left, pos, current);
                    }
                    if back.is_air() {
                        local_mesh.push_face(Direction::Back, pos, current);
                    }
                    if down.is_air() {
                        local_mesh.push_face(Direction::Bottom, pos, current);
                    }
                } else {
                    if !left.is_air() {
                        local_mesh.push_face(Direction::Right, pos, left);
                    }
                    if !back.is_air() {
                        local_mesh.push_face(Direction::Front, pos, back);
                    }
                    if !down.is_air() {
                        local_mesh.push_face(Direction::Top, pos, down);
                    }
                }

                if local_mesh.vertices.is_empty() {
                    None
                } else {
                    Some(local_mesh)
                }
            })
            .collect();

        for part in mesh_parts {
            self.vertices.extend(part.vertices);
        }

        if self.vertices.is_empty() {
            None
        } else {
            self.vertices.shrink_to_fit();
            self.indices
                .extend((0..self.vertices.len() / 4).flat_map(|i| {
                    let idx = i as u32 * 4;
                    [idx, idx + 1, idx + 2, idx, idx + 2, idx + 3]
                }));

            Some(self)
        }
    }

    #[inline(always)]
    pub fn push_face(&mut self, dir: Direction, pos: IVec3, block: Block) {
        for (corner, pos) in Quad::from_direction(dir, pos.as_vec3(), Vec3::ONE)
            .corners
            .into_iter()
            .enumerate()
        {
            let vertex_data = pos[0] as u32
                | (pos[1] as u32) << 6
                | (pos[2] as u32) << 12
                | (dir as u32) << 18
                | (corner as u32) << 21
                | (block as u32) << 23;

            self.vertices.push(VoxelVertex { vertex_data });
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

    #[inline(always)]
    pub fn get_adjacent_blocks(
        &self,
        pos: IVec3,
        left_chunk: Option<&Chunk>,
        back_chunk: Option<&Chunk>,
        down_chunk: Option<&Chunk>,
    ) -> (Block, Block, Block) {
        let x = pos.x;
        let y = pos.y;
        let z = pos.z;

        let get_block = |dx: i32, dy: i32, dz: i32, fallback: Option<&Chunk>| -> Block {
            let nx = x + dx;
            let ny = y + dy;
            let nz = z + dz;

            if (0..CHUNK_SIZE).contains(&nx)
                && (0..CHUNK_SIZE).contains(&ny)
                && (0..CHUNK_SIZE).contains(&nz)
            {
                return *unsafe {
                    self.blocks
                        .get_unchecked(vec3_to_index(IVec3::new(nx, ny, nz)))
                };
            }

            let mut lx = nx;
            let mut ly = ny;
            let mut lz = nz;

            if nx < 0 {
                lx += CHUNK_SIZE;
            } else if nx >= CHUNK_SIZE {
                lx -= CHUNK_SIZE;
            }

            if ny < 0 {
                ly += CHUNK_SIZE;
            } else if ny >= CHUNK_SIZE {
                ly -= CHUNK_SIZE;
            }

            if nz < 0 {
                lz += CHUNK_SIZE;
            } else if nz >= CHUNK_SIZE {
                lz -= CHUNK_SIZE;
            }

            if let Some(chunk) = fallback {
                return *unsafe {
                    chunk
                        .blocks
                        .get_unchecked(vec3_to_index(IVec3::new(lx, ly, lz)))
                };
            }

            Block::Air
        };

        let back = get_block(0, 0, -1, back_chunk);
        let left = get_block(-1, 0, 0, left_chunk);
        let down = get_block(0, -1, 0, down_chunk);

        (back, left, down)
    }
}
