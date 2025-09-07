use std::{marker::PhantomData, ptr::null};

use gl::types::*;

pub trait Vertex {
    fn attributes() -> &'static [(GLuint, GLint, GLenum, GLboolean, usize)];
}

pub struct Mesh<V: Vertex> {
    pub vao: GLuint,
    pub vbo: GLuint,
    pub ebo: GLuint,
    pub index_count: GLint,
    _marker: PhantomData<V>,
}

impl<V: Vertex> Mesh<V> {
    pub fn new(vertices: &[V], indices: &[u32]) -> Result<Self, String> {
        let (mut vao, mut vbo, mut ebo) = (0, 0, 0);
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            gl::GenBuffers(1, &mut ebo);

            gl::BindVertexArray(vao);

            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                size_of_val(vertices) as isize,
                vertices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                size_of_val(indices) as isize,
                indices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            for &(location, size, type_, normalized, offset) in V::attributes() {
                match type_ {
                    gl::UNSIGNED_INT | gl::INT | gl::UNSIGNED_BYTE | gl::BYTE => {
                        gl::VertexAttribIPointer(
                            location,
                            size,
                            type_,
                            size_of::<V>() as GLint,
                            offset as *const _,
                        );
                    }
                    _ => {
                        gl::VertexAttribPointer(
                            location,
                            size,
                            type_,
                            normalized,
                            size_of::<V>() as GLint,
                            offset as *const _,
                        );
                    }
                }
                gl::EnableVertexAttribArray(location);
            }

            gl::BindVertexArray(0);
        }

        Ok(Mesh {
            vao,
            vbo,
            ebo,
            index_count: indices.len() as GLint,
            _marker: PhantomData,
        })
    }

    pub fn draw(&self) -> usize {
        unsafe {
            let mut size = 0;
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::GetBufferParameteriv(gl::ARRAY_BUFFER, gl::BUFFER_SIZE, &mut size);
            gl::BindVertexArray(self.vao);
            gl::DrawElements(gl::TRIANGLES, self.index_count, gl::UNSIGNED_INT, null());

            (size as usize / size_of::<V>()) / 3
        }
    }
}

impl<V: Vertex> Drop for Mesh<V> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteBuffers(1, &self.ebo);
        }
    }
}
