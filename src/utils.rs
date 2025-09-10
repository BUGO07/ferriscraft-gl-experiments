use std::{alloc::Layout, path::Path, ptr::null_mut};

use glam::*;
use glfw::{Context, PWindow};

use crate::{
    CHUNK_SIZE, SEA_LEVEL,
    ecs::{Aabb, Window},
    world::mesher::Block,
};

#[inline]
pub fn set_cursor_grab(window: &mut Window, val: bool) {
    window.cursor_grab = val;
    window.cursor_visible = !val;
}

#[inline]
pub const fn generate_block_at(pos: IVec3, max_y: i32) -> Block {
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
    } else if y < SEA_LEVEL {
        Block::Water
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

#[inline]
pub const fn vec3_to_index(pos: IVec3) -> usize {
    (pos.x + pos.y * CHUNK_SIZE + pos.z * CHUNK_SIZE * CHUNK_SIZE) as usize
}

#[inline]
pub const fn index_to_vec3(index: usize) -> IVec3 {
    ivec3(
        index as i32 % CHUNK_SIZE,
        (index as i32 / CHUNK_SIZE) % CHUNK_SIZE,
        index as i32 / (CHUNK_SIZE * CHUNK_SIZE),
    )
}

#[inline]
pub fn should_cull_aabb(frustum: &[Vec4; 6], pos: Vec3, aabb: &Aabb) -> bool {
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

#[inline]
pub fn should_cull_sphere(frustum: &[Vec4; 6], pos: Vec3, radius: f32) -> bool {
    for plane in frustum {
        let distance = plane.xyz().dot(pos) + plane.w;
        if distance < -radius {
            return true;
        }
    }
    false
}

#[inline]
pub fn take_screenshot(window: &PWindow) {
    unsafe {
        let (width, height) = window.get_framebuffer_size();
        let size = (3 * width * height) as usize;
        let layout = Layout::from_size_align(size, 0x8).unwrap();
        let pixels = std::alloc::alloc(layout);

        gl::ReadPixels(
            0,
            0,
            width,
            height,
            gl::RGB,
            gl::UNSIGNED_BYTE,
            pixels as *mut _,
        );

        let img = image::imageops::flip_vertical(
            &image::RgbImage::from_raw(
                width as u32,
                height as u32,
                Vec::from_raw_parts(pixels, size, size),
            )
            .expect("couldn't read pixels"),
        );

        let path = Path::new("screenshots");
        if !path.exists() {
            std::fs::create_dir(path).expect("couldn't create dir");
        }
        image::save_buffer(
            path.join(format!(
                "screenshot-{}.png",
                chrono::Local::now().format("%Y-%m-%d-%H-%M-%S%.3fZ")
            )),
            &img,
            width as u32,
            height as u32,
            image::ColorType::Rgb8,
        )
        .expect("couldn't save to disk");
    }
}

#[inline]
pub fn toggle_fullscreen(window: &PWindow) {
    unsafe {
        let window_ptr = window.window_ptr();
        let monitor_ptr = glfw::ffi::glfwGetWindowMonitor(window_ptr);

        if monitor_ptr.is_null() {
            let primary = glfw::ffi::glfwGetPrimaryMonitor();
            let mode = *glfw::ffi::glfwGetVideoMode(primary);
            glfw::ffi::glfwSetWindowMonitor(
                window_ptr,
                primary,
                0,
                0,
                mode.width,
                mode.height,
                mode.refreshRate,
            );
        } else {
            glfw::ffi::glfwSetWindowMonitor(window_ptr, null_mut(), 0, 0, 1280, 720, 0);
        }
    }
}
