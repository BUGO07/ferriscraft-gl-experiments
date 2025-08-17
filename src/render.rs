use std::collections::HashMap;

use egui_glium::EguiGlium;
use glium::{
    BackfaceCullingMode, Depth, DepthTest, DrawParameters, IndexBuffer, Program, Surface,
    Texture2d, VertexBuffer,
    index::PrimitiveType,
    texture::RawImage2d,
    uniforms::{MagnifySamplerFilter, MinifySamplerFilter},
};
use image::ImageFormat;

use crate::{
    CHUNK_SIZE, FRAGMENT_SHADER, SEA_LEVEL, VERTEX_SHADER,
    ecs::*,
    mesher::{Chunk, ChunkMesh},
    utils::{frustum_planes, generate_block_at, should_cull, vec3_to_index},
};

pub fn render_setup(
    mut commands: Commands,
    window: NonSend<Window>,
    mut meshes: NonSendMut<Meshes>,
    mut materials: NonSendMut<Materials>,
) {
    // Camera
    commands.spawn((
        Camera3d {
            fov: 60.0,
            near: 0.1,
            far: 1024.0,
        },
        Transform::from_xyz(0.0, (4 * CHUNK_SIZE + 10) as f32, 5.5)
            .looking_at(Vec3::Y * (8 * CHUNK_SIZE) as f32, Vec3::Y),
    ));

    // Directional light
    commands.spawn((
        DirectionalLight {
            illuminance: 1000.0,
        },
        Transform::from_xyz(3.0, 5.0, 2.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Shared material / texture
    let image = image::load(
        std::io::Cursor::new(std::fs::read("assets/atlas.png").unwrap()),
        ImageFormat::Png,
    )
    .unwrap()
    .to_rgba8();
    let image_dimensions = image.dimensions();
    let image = RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
    let texture = Texture2d::new(&window.gl_context, image).unwrap();
    let material = materials.add(Material::new(
        Program::from_source(&window.gl_context, VERTEX_SHADER, FRAGMENT_SHADER, None).unwrap(),
        texture,
    ));

    const CHUNK_SIZE_VEC: Vec3 = Vec3::splat(CHUNK_SIZE as f32);

    let mut chunks = HashMap::new();

    // Spawn chunks individually
    for cy in 0..4 {
        for cz in -8..8 {
            for cx in -8..8 {
                let chunk_pos = IVec3::new(cx, cy, cz);
                let mut chunk = Chunk::new(chunk_pos);

                for y in 0..CHUNK_SIZE {
                    for z in 0..CHUNK_SIZE {
                        for x in 0..CHUNK_SIZE {
                            let block_pos = ivec3(x, y, z);
                            chunk.blocks[vec3_to_index(block_pos)] = generate_block_at(
                                block_pos + chunk_pos * CHUNK_SIZE,
                                rand::random_range(SEA_LEVEL..(CHUNK_SIZE * 4 - 1)),
                            );
                        }
                    }
                }

                chunks.insert(chunk_pos, chunk);
            }
        }
    }

    for (chunk_pos, chunk) in &chunks {
        if let Some(mesh_data) = ChunkMesh::default().build(chunk, &chunks) {
            let vertex_buffer = VertexBuffer::new(&window.gl_context, &mesh_data.vertices).unwrap();
            let index_buffer = IndexBuffer::new(
                &window.gl_context,
                PrimitiveType::TrianglesList,
                &mesh_data.indices,
            )
            .unwrap();

            let mesh_id = meshes.add(Mesh::new(vertex_buffer, index_buffer));

            commands.spawn((
                mesh_id,
                material,
                Transform::from_translation((chunk_pos * CHUNK_SIZE).as_vec3()),
                Aabb::new(Vec3::ZERO, CHUNK_SIZE_VEC),
            ));
        }
    }
}
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn render_update(
    window: NonSend<Window>,
    meshes: NonSend<Meshes>,
    materials: NonSend<Materials>,
    mesh_entities: Query<
        (&Transform, &Mesh3d, &MeshMaterial, &Aabb),
        (Without<Camera3d>, Without<DirectionalLight>),
    >,
    camera_query: Single<(&mut Transform, &Camera3d), (Without<Mesh3d>, Without<DirectionalLight>)>,
    light_query: Single<(&Transform, &DirectionalLight), (Without<Mesh3d>, Without<Camera3d>)>,
    debug_info: Option<ResMut<DebugInfo>>,
    mut egui: NonSendMut<EguiGlium>,
) {
    let mut target = window.gl_context.draw();
    target.clear_color_and_depth((0.0, 0.0, 1.0, 1.0), 1.0);

    let (camera_transform, camera) = camera_query.into_inner();
    let (light_transform, _light) = light_query.into_inner();

    let (width, height) = target.get_dimensions();
    let perspective = Mat4::perspective_rh_gl(
        camera.fov.to_radians(),
        width as f32 / height as f32,
        camera.near,
        camera.far,
    );

    let view = camera_transform.as_mat4().inverse();
    let vp: Mat4 = perspective * view;
    let frustum = frustum_planes(&vp);

    let mut draw_calls = 0;
    let mut vertices = 0;
    let mut indices = 0;

    for (chunk_transform, mesh_id, material_id, aabb) in mesh_entities.iter() {
        if should_cull(&frustum, chunk_transform.translation, aabb) {
            continue;
        }
        let Mesh {
            vertex_buffer,
            index_buffer,
        } = &meshes.0[mesh_id.0];
        let Material { program, texture } = &materials.0[material_id.0];

        let sampler = texture
            .sampled()
            .magnify_filter(MagnifySamplerFilter::Nearest)
            .minify_filter(MinifySamplerFilter::NearestMipmapNearest);

        let uniforms = uniform! {
            model: chunk_transform.as_mat4().to_cols_array_2d(),
            view: view.to_cols_array_2d(),
            perspective: perspective.to_cols_array_2d(),
            tex: sampler,
            u_light: (light_transform.rotation * Vec3::NEG_Z).normalize().to_array(),
        };

        let params = DrawParameters {
            depth: Depth {
                test: DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            backface_culling: BackfaceCullingMode::CullClockwise,
            ..Default::default()
        };

        vertices += vertex_buffer.len();
        indices += index_buffer.len();
        draw_calls += 1;

        target
            .draw(vertex_buffer, index_buffer, program, &uniforms, &params)
            .unwrap();
    }

    if let Some(mut debug_info) = debug_info {
        debug_info.vertices = vertices;
        debug_info.indices = indices;
        debug_info.draw_calls = draw_calls;
    }

    egui.paint(&window.gl_context, &mut target);
    target.finish().unwrap();
}
