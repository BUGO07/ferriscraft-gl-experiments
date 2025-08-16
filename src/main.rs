use std::collections::HashMap;

use egui_glium::{EguiGlium, egui_winit::egui};
use glam::*;
use glium::{
    BackfaceCullingMode, Depth, DepthTest, DrawParameters, IndexBuffer, Program, Surface,
    Texture2d, VertexBuffer,
    backend::glutin::SimpleWindowBuilder,
    index::PrimitiveType,
    texture::RawImage2d,
    uniforms::{MagnifySamplerFilter, MinifySamplerFilter},
    winit::{
        event::{Event, WindowEvent},
        event_loop::EventLoop,
    },
};
use image::ImageFormat;

use crate::{
    mesher::{Chunk, ChunkMesh},
    utils::{generate_block_at, vec3_to_index},
};

#[macro_use]
extern crate glium;

const VERTEX_SHADER: &str = include_str!("../assets/shaders/vertex.glsl");
const FRAGMENT_SHADER: &str = include_str!("../assets/shaders/fragment.glsl");

const CHUNK_SIZE: i32 = 16;
const CHUNK_HEIGHT: i32 = 16;

mod mesher;
mod utils;

#[derive(Clone, Copy)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Transform {
    pub const DEFAULT: Self = Self::from_translation(Vec3::ZERO);

    #[inline]
    pub const fn from_translation(translation: Vec3) -> Self {
        Self {
            translation,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
    #[inline]
    pub fn from_xyz(x: f32, y: f32, z: f32) -> Self {
        Self::from_translation(vec3(x, y, z))
    }
    #[inline]
    pub fn with_rotation(mut self, rotation: Quat) -> Self {
        self.rotation = rotation;
        self
    }
    #[inline]
    pub fn with_scale(mut self, scale: Vec3) -> Self {
        self.scale = scale;
        self
    }
    #[inline]
    pub fn from_mat4(mat4: Mat4) -> Self {
        let (scale, rotation, translation) = mat4.to_scale_rotation_translation();
        Self {
            translation,
            rotation,
            scale,
        }
    }
    #[inline]
    pub fn as_mat4(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }
}

#[allow(deprecated)]
fn main() {
    let event_loop = EventLoop::new().expect("couldn't create event loop");

    let (window, display) = SimpleWindowBuilder::new()
        .with_title("FerrisCraft GL")
        .with_inner_size(1280, 720)
        .build(&event_loop);

    let mut egui = EguiGlium::new(egui::ViewportId::ROOT, &display, &window, &event_loop);

    let mut chunk = Chunk::new(IVec3::ZERO);
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

    let index_buffer =
        IndexBuffer::new(&display, PrimitiveType::TrianglesList, &mesh.indices).unwrap();

    let image = image::load(
        std::io::Cursor::new(std::fs::read("assets/atlas.png").unwrap()),
        ImageFormat::Png,
    )
    .unwrap()
    .to_rgba8();
    let image_dimensions = image.dimensions();
    let image = RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);

    let texture = Texture2d::new(&display, image).unwrap();
    let sampler = texture
        .sampled()
        .magnify_filter(MagnifySamplerFilter::Nearest)
        .minify_filter(MinifySamplerFilter::NearestMipmapNearest);

    let vertex_buffer = VertexBuffer::new(&display, &mesh.vertices).unwrap();

    let program = Program::from_source(&display, VERTEX_SHADER, FRAGMENT_SHADER, None).unwrap();

    let mut chunk_transform = Transform::from_translation(vec3(-0.2, -0.2, 0.8));
    let mut camera_transform = Transform::from_translation(vec3(0.0, 0.0, 2.0));
    let mut light_transform = Transform::DEFAULT;
    let mut fov = 60.0_f32;

    event_loop
        .run(move |event, window_target| {
            match event {
                Event::WindowEvent { event, .. } => {
                    let _ = egui.on_event(&window, &event);
                    match event {
                        WindowEvent::CloseRequested => window_target.exit(),
                        WindowEvent::RedrawRequested => {
                            let mut target = display.draw();
                            target.clear_color_and_depth((0.0, 0.0, 1.0, 1.0), 1.0);

                            let (width, height) = target.get_dimensions();
                            let perspective = Mat4::perspective_rh_gl(
                                fov.to_radians(),
                                width as f32 / height as f32,
                                0.1,
                                1024.0,
                            )
                            .to_cols_array_2d();

                            let uniforms = uniform! {
                                model: chunk_transform.as_mat4().to_cols_array_2d(),
                                view: camera_transform.as_mat4().inverse().to_cols_array_2d(),
                                perspective: perspective,
                                tex: sampler,
                                u_light: (light_transform.rotation * Vec3::NEG_Z).normalize().to_array(),
                            };

                            let params = DrawParameters {
                                depth: Depth {
                                    test: DepthTest::IfLess,
                                    write: true,
                                    ..Default::default()
                                },
                                backface_culling: BackfaceCullingMode::CullCounterClockwise,
                                ..Default::default()
                            };

                            target
                                .draw(&vertex_buffer, &index_buffer, &program, &uniforms, &params)
                                .unwrap();

                            egui.run(&window, |ctx| {
                                egui::Window::new("change transforms").show(ctx, |ui| {
                                    ui.add(
                    egui::DragValue::new(&mut fov)
                        .speed(0.1)
                        .range(0.1..=179.9),
                );
                                    egui_display_transform("chunk", ui, &mut chunk_transform);
                                    egui_display_transform("camera", ui, &mut camera_transform);
                                    egui_display_transform("light", ui, &mut light_transform);
                                });
                            });
                            egui.paint(&display, &mut target);
                            target.finish().unwrap();
                        }
                        WindowEvent::Resized(window_size) => {
                            display.resize(window_size.into());
                        }
                        _ => (),
                    }
                }
                Event::AboutToWait => {
                    window.request_redraw();
                }
                _ => (),
            };
        })
        .ok();
}

fn egui_display_transform(label: &str, ui: &mut egui::Ui, transform: &mut Transform) {
    ui.set_max_width(200.0);
    let (yaw, pitch, roll) = transform.rotation.to_euler(EulerRot::YXZ);

    let mut rotation = vec3(yaw.to_degrees(), pitch.to_degrees(), roll.to_degrees());

    fn drag_vec3(label: &str, ui: &mut egui::Ui, vec: &mut Vec3, speed: f32, min: f32, max: f32) {
        ui.vertical_centered(|ui| {
            ui.horizontal(|ui| {
                ui.label(label);
                ui.add(
                    egui::DragValue::new(&mut vec.x)
                        .speed(speed)
                        .range(min..=max),
                );
                ui.add(
                    egui::DragValue::new(&mut vec.y)
                        .speed(speed)
                        .range(min..=max),
                );
                ui.add(
                    egui::DragValue::new(&mut vec.z)
                        .speed(speed)
                        .range(min..=max),
                );
            });
        });
    }

    ui.vertical_centered(|ui| {
        ui.label(label);
        drag_vec3(
            "translation",
            ui,
            &mut transform.translation,
            0.1,
            f32::MIN,
            f32::MAX,
        );
        drag_vec3("rotation     ", ui, &mut rotation, 1.0, -180.0, 179.9);
        drag_vec3(
            "scale           ",
            ui,
            &mut transform.scale,
            0.1,
            0.0,
            f32::MAX,
        );
    });

    transform.rotation = Quat::from_euler(
        EulerRot::YXZ,
        rotation.x.to_radians(),
        rotation.y.to_radians(),
        rotation.z.to_radians(),
    );
}
