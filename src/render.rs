use std::collections::HashMap;

use crate::{
    CHUNK_HEIGHT, CHUNK_SIZE, FRAGMENT_SHADER, VERTEX_SHADER, WindowEventECS,
    ecs::*,
    egui_display_transform,
    mesher::{Chunk, ChunkMesh, Vertex},
    utils::{generate_block_at, vec3_to_index},
};
use egui_glium::{EguiGlium, egui_winit::egui};
use glam::*;
use glium::{
    BackfaceCullingMode, Depth, DepthTest, Display, DrawParameters, IndexBuffer, Program, Surface,
    Texture2d, VertexBuffer,
    glutin::surface::WindowSurface,
    index::PrimitiveType,
    texture::RawImage2d,
    uniforms::{MagnifySamplerFilter, MinifySamplerFilter},
};
use image::ImageFormat;

pub struct Window {
    pub winit_window: glium::winit::window::Window,
    pub display: Display<WindowSurface>,
}

#[derive(Debug)]
pub struct Meshes(pub Vec<Mesh>);

#[derive(Debug)]
pub struct Mesh {
    pub program: Program,
    pub texture: Texture2d,
    pub vertex_buffer: VertexBuffer<Vertex>,
    pub index_buffer: IndexBuffer<u32>,
}

pub fn render_setup(
    mut commands: Commands,
    window: NonSend<Window>,
    mut meshes: NonSendMut<Meshes>,
) {
    commands.spawn((
        ChunkEntity,
        Transform::from_translation(vec3(-0.2, -0.2, 0.8)),
    ));
    commands.spawn((
        Camera3d {
            fov: 60.0,
            near: 0.1,
            far: 1024.0,
        },
        Transform::from_translation(vec3(0.0, 0.0, 2.0)),
    ));
    commands.spawn((
        DirectionalLight {
            // TODO: implement
            illuminance: 1000.0,
        },
        Transform::DEFAULT,
    ));

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
        IndexBuffer::new(&window.display, PrimitiveType::TrianglesList, &mesh.indices).unwrap();

    let image = image::load(
        std::io::Cursor::new(std::fs::read("assets/atlas.png").unwrap()),
        ImageFormat::Png,
    )
    .unwrap()
    .to_rgba8();
    let image_dimensions = image.dimensions();
    let image = RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);

    let texture = Texture2d::new(&window.display, image).unwrap();

    let vertex_buffer = VertexBuffer::new(&window.display, &mesh.vertices).unwrap();

    meshes.0.push(Mesh {
        texture,
        index_buffer,
        vertex_buffer,
        program: Program::from_source(&window.display, VERTEX_SHADER, FRAGMENT_SHADER, None)
            .unwrap(),
    });
}

#[allow(clippy::type_complexity)]
pub fn render_update(
    window: NonSend<Window>,
    meshes: NonSend<Meshes>,
    mut chunk_transform: Single<
        &mut Transform,
        (
            With<ChunkEntity>,
            Without<Camera3d>,
            Without<DirectionalLight>,
        ),
    >,
    camera_query: Single<
        (&mut Transform, &mut Camera3d),
        (Without<ChunkEntity>, Without<DirectionalLight>),
    >,
    light_query: Single<
        (&mut Transform, &mut DirectionalLight),
        (Without<ChunkEntity>, Without<Camera3d>),
    >,
    mut egui: NonSendMut<EguiGlium>,
    mut window_events: EventReader<WindowEventECS>,
) {
    for event in window_events.read() {
        let _ = egui.on_event(&window.winit_window, &event.0);
    }
    let mut target = window.display.draw();
    target.clear_color_and_depth((0.0, 0.0, 1.0, 1.0), 1.0);

    let (mut camera_transform, mut camera) = camera_query.into_inner();
    let (mut light_transform, mut _light) = light_query.into_inner();

    let (width, height) = target.get_dimensions();
    let perspective = Mat4::perspective_rh_gl(
        camera.fov.to_radians(),
        width as f32 / height as f32,
        camera.near,
        camera.far,
    )
    .to_cols_array_2d();

    let Mesh {
        index_buffer,
        vertex_buffer,
        program,
        texture,
    } = &meshes.0[0];
    let sampler = texture
        .sampled()
        .magnify_filter(MagnifySamplerFilter::Nearest)
        .minify_filter(MinifySamplerFilter::NearestMipmapNearest);

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
        .draw(vertex_buffer, index_buffer, program, &uniforms, &params)
        .unwrap();

    egui.run(&window.winit_window, |ctx| {
        egui::Window::new("change transforms").show(ctx, |ui| {
            ui.add(
                egui::DragValue::new(&mut camera.fov)
                    .speed(0.1)
                    .range(0.1..=179.9),
            );
            egui_display_transform("chunk", ui, &mut chunk_transform);
            egui_display_transform("camera", ui, &mut camera_transform);
            egui_display_transform("light", ui, &mut light_transform);
        });
    });
    egui.paint(&window.display, &mut target);
    target.finish().unwrap();
}
