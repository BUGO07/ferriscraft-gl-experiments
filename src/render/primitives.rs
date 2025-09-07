#![allow(clippy::new_ret_no_self)]

use gl::types::*;
use glam::Vec3;

use crate::{render::mesh::Vertex, world::mesher::Direction};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct PrimitiveVertex {
    pub pos: [f32; 3],
}

impl Vertex for PrimitiveVertex {
    fn attributes() -> &'static [(GLuint, GLint, GLenum, GLboolean, usize)] {
        &[(0, 3, gl::FLOAT, gl::FALSE, 0)]
    }
}

pub struct Cuboid;

impl Cuboid {
    pub fn new(size: Vec3, pos_offset: Vec3) -> Vec<PrimitiveVertex> {
        use super::Direction::*;
        let mut vertices = Vec::new();
        for dir in [Left, Right, Bottom, Top, Back, Front] {
            let quad = Quad::new(
                dir,
                (dir.as_ivec3().as_vec3().max(Vec3::ZERO) + pos_offset) * size,
                size,
            );
            for pos in quad {
                vertices.push(PrimitiveVertex { pos });
            }
        }
        vertices
    }

    pub fn generate_indices(vertices: usize) -> Vec<u32> {
        (0..vertices)
            .step_by(4)
            .flat_map(|i| {
                let idx = i as u32;
                [idx, idx + 1, idx + 2, idx, idx + 2, idx + 3]
            })
            .collect::<Vec<_>>()
    }
}

pub struct Quad;

impl Quad {
    #[inline]
    pub const fn new(direction: Direction, pos: Vec3, size: Vec3) -> [[f32; 3]; 4] {
        match direction {
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
        }
    }
}
