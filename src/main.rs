use std::collections::HashMap;

use egui_glium::egui_winit::egui::{Slider, ViewportId, Window};
use glam::{EulerRot, IVec3, Mat4, Quat, ivec3};
use glium::{IndexBuffer, Surface};

use crate::{
    mesher::{Chunk, ChunkMesh},
    utils::{generate_block_at, vec3_to_index},
};

#[macro_use]
extern crate glium;

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 3],
    uvs: [f32; 2],
}

implement_vertex!(Vertex, position, uvs);

const VERTEX_SHADER: &str = include_str!("../assets/shaders/vertex.glsl");
const FRAGMENT_SHADER: &str = include_str!("../assets/shaders/fragment.glsl");

const CHUNK_SIZE: i32 = 16;
const CHUNK_HEIGHT: i32 = 16;

mod mesher;
mod utils;

#[allow(deprecated)]
fn main() {
    let event_loop = glium::winit::event_loop::EventLoop::builder()
        .build()
        .expect("event loop building");
    let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new()
        .with_title("FerrisCraft GL")
        .with_inner_size(1280, 720)
        .build(&event_loop);

    let mut shape = vec![];
    let mut chunk = Chunk::new(IVec3::ZERO);

    let mut egui = egui_glium::EguiGlium::new(ViewportId::ROOT, &display, &window, &event_loop);

    for y in 0..CHUNK_HEIGHT {
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                chunk.blocks[vec3_to_index(IVec3::new(x, y, z))] =
                    generate_block_at(ivec3(x, y, z), rand::random_range(7..16));
            }
        }
    }

    let chunks = HashMap::from([(IVec3::ZERO, chunk.clone())]);

    let mesh = ChunkMesh::default().build(&chunk, &chunks).unwrap();

    for vertex in mesh.vertices.iter() {
        shape.push(Vertex {
            position: vertex.pos.into(),
            uvs: vertex.uv.into(),
        });
    }
    let binding = IndexBuffer::new(
        &display,
        glium::index::PrimitiveType::TrianglesList,
        &mesh.indices,
    )
    .unwrap();
    let indices = glium::index::IndicesSource::IndexBuffer {
        buffer: binding.as_slice_any(),
        data_type: glium::index::IndexType::U32,
        primitives: glium::index::PrimitiveType::TrianglesList,
    };

    let image = image::load(
        std::io::Cursor::new(&include_bytes!("../assets/atlas.png")),
        image::ImageFormat::Png,
    )
    .unwrap()
    .to_rgba8();
    let image_dimensions = image.dimensions();
    let image =
        glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);

    let texture = glium::texture::Texture2d::new(&display, image).unwrap();
    let sampled = texture
        .sampled()
        .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest);

    let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();

    let program =
        glium::Program::from_source(&display, VERTEX_SHADER, FRAGMENT_SHADER, None).unwrap();

    // (glium::index::PrimitiveType::TrianglesList);
    let mut yaw = 0.0_f32;
    let mut pitch = 0.0_f32;
    let mut roll = 0.0_f32;

    event_loop
        .run(move |event, window_target| {
            match event {
                glium::winit::event::Event::WindowEvent { event, .. } => {
                    let _ = egui.on_event(&window, &event);
                    match event {
                        glium::winit::event::WindowEvent::CloseRequested => window_target.exit(),
                        glium::winit::event::WindowEvent::RedrawRequested => {
                            let mut target = display.draw();
                            target.clear_color(0.0, 0.0, 1.0, 1.0);

                            let rotation = Quat::from_euler(
                                EulerRot::YXZ,
                                yaw.to_radians(),
                                pitch.to_radians(),
                                roll.to_radians(),
                            );
                            let mat = Mat4::from_quat(rotation);
                            let matrix: [[f32; 4]; 4] = [
                                mat.x_axis.into(),
                                mat.y_axis.into(),
                                mat.z_axis.into(),
                                mat.w_axis.into(),
                            ];

                            let uniforms = uniform! {
                                matrix: matrix,
                                tex: sampled,
                            };

                            target
                                .draw(
                                    &vertex_buffer,
                                    indices.clone(),
                                    &program,
                                    &uniforms,
                                    &Default::default(),
                                )
                                .unwrap();

                            egui.run(&window, |ctx| {
                                Window::new("rotate chunk").show(ctx, |ui| {
                                    ui.add(Slider::new(&mut yaw, 0.0..=360.0));
                                    ui.add(Slider::new(&mut pitch, 0.0..=360.0));
                                    ui.add(Slider::new(&mut roll, 0.0..=360.0));
                                });
                            });
                            egui.paint(&display, &mut target);
                            target.finish().unwrap();
                        }
                        glium::winit::event::WindowEvent::Resized(window_size) => {
                            display.resize(window_size.into());
                        }
                        _ => (),
                    }
                }
                glium::winit::event::Event::AboutToWait => {
                    window.request_redraw();
                }
                _ => (),
            };
        })
        .ok();
}
