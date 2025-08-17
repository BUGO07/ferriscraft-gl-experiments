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

impl Direction {
    pub const NORMALS: &[[f32; 3]; 6] = &[
        [-1.0, 0.0, 0.0], // Left
        [1.0, 0.0, 0.0],  // Right
        [0.0, -1.0, 0.0], // Bottom
        [0.0, 1.0, 0.0],  // Top
        [0.0, 0.0, -1.0], // Back
        [0.0, 0.0, 1.0],  // Front
    ];

    #[inline]
    pub fn as_vec3(self) -> [f32; 3] {
        Self::NORMALS[self as usize]
    }

    #[inline]
    pub fn get_opposite(self) -> Self {
        match self {
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
            Direction::Bottom => Direction::Top,
            Direction::Top => Direction::Bottom,
            Direction::Back => Direction::Front,
            Direction::Front => Direction::Back,
        }
    }

    #[inline]
    pub fn get_uvs(self, block: Block) -> [[f32; 2]; 4] {
        const ATLAS_SIZE_X: f32 = 1.0;
        const ATLAS_SIZE_Y: f32 = 10.0;

        let block_direction = Direction::Top;

        let face_idx = 0.0;

        let pos = vec2(
            face_idx / ATLAS_SIZE_X,
            1.0 - ((block as u32 - 1) as f32 / ATLAS_SIZE_Y),
        );

        let base = [
            [pos.x, pos.y + 1.0 / ATLAS_SIZE_Y],
            [pos.x, pos.y],
            [pos.x + 1.0 / ATLAS_SIZE_X, pos.y],
            [pos.x + 1.0 / ATLAS_SIZE_X, pos.y + 1.0 / ATLAS_SIZE_Y],
        ];
        let rotate_90 = [base[3], base[0], base[1], base[2]];
        let rotate_180 = [base[2], base[3], base[0], base[1]];
        let rotate_270 = [base[1], base[2], base[3], base[0]];

        // HOLY BAD CODE
        use Direction::*;
        match (block_direction, self) {
            (Right, Top | Bottom) => base,
            (Right, Back) => rotate_90,
            (Right, _) => rotate_270,
            (Top, Front | Back) => base,
            (Top, Left) => rotate_90,
            (Top, _) => rotate_270,
            (Front, Right | Left) => base,
            (Front, Bottom) => rotate_90,
            (Front, _) => rotate_270,
            (Left, Top | Bottom) => rotate_180,
            (Left, Back) => rotate_270,
            (Left, _) => rotate_90,
            (Bottom, Front | Back) => rotate_180,
            (Bottom, Left) => rotate_270,
            (Bottom, _) => rotate_90,
            (Back, Right | Left) => rotate_180,
            (Back, Bottom) => rotate_270,
            (Back, _) => rotate_90,
        }
    }
}

#[derive(Default)]
pub struct ChunkMesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}
implement_vertex!(Vertex, pos, normal, uv);

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
                let local = pos.as_vec3();

                let current = *unsafe { chunk.blocks.get_unchecked(i as usize) };

                let (back, left, down) =
                    chunk.get_adjacent_blocks(pos, left_chunk, back_chunk, down_chunk);

                if !current.is_air() {
                    if left.is_air() {
                        local_mesh.push_face(Direction::Left, local, current);
                    }
                    if back.is_air() {
                        local_mesh.push_face(Direction::Back, local, current);
                    }
                    if down.is_air() {
                        local_mesh.push_face(Direction::Bottom, local, current);
                    }
                } else {
                    if !left.is_air() {
                        local_mesh.push_face(Direction::Right, local, left);
                    }
                    if !back.is_air() {
                        local_mesh.push_face(Direction::Front, local, back);
                    }
                    if !down.is_air() {
                        local_mesh.push_face(Direction::Top, local, down);
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
            for v in part.vertices {
                self.vertices.push(v);
            }
            for i in part.indices {
                self.indices.push(i + self.vertices.len() as u32);
            }
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
    pub fn push_face(&mut self, dir: Direction, pos: Vec3, block: Block) {
        let uvs = dir.get_uvs(block);
        for (i, corner) in Quad::from_direction(dir, pos, Vec3::ONE)
            .corners
            .into_iter()
            .enumerate()
        {
            self.vertices.push(Vertex {
                pos: corner,
                normal: dir.as_vec3(),
                uv: uvs[i],
            });
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
