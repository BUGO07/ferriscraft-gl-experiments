use std::collections::HashMap;

use egui_glium::EguiGlium;
use fastnoise_lite::FastNoiseLite;
use glium::{
    BackfaceCullingMode, Depth, DepthTest, DrawParameters, IndexBuffer, Surface, VertexBuffer,
    index::PrimitiveType,
    uniforms::{MagnifySamplerFilter, MinifySamplerFilter},
};

use crate::{
    App, CHUNK_SIZE,
    ecs::*,
    utils::{Quad, frustum_planes, generate_block_at, should_cull, vec3_to_index},
    world::mesher::{Chunk, ChunkMesh, UIVertex, VoxelVertex},
};

pub mod inspector;

pub fn render_plugin(app: &mut App) {
    app.add_systems(Startup, setup)
        .add_systems(EguiContextPass, inspector::handle_egui)
        .add_systems(RenderUpdate, render_update);
}

pub fn setup(
    mut commands: Commands,
    mut ui_meshes: NonSendMut<Meshes<UIVertex>>,
    mut voxel_meshes: NonSendMut<Meshes<VoxelVertex>>,
    mut materials: NonSendMut<Materials>,
    window: NonSend<NSWindow>,
) {
    commands.spawn((
        Camera3d {
            fov: 60.0,
            near: 0.1,
            far: 1024.0,
        },
        Transform::from_xyz(0.0, (4 * CHUNK_SIZE + 10) as f32, 5.5)
            .looking_at(Vec3::Y * (8 * CHUNK_SIZE) as f32, Vec3::Y),
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 1000.0,
        },
        Transform::DEFAULT.with_rotation(
            Quat::from_rotation_x(45_f32.to_radians())
                * Quat::from_rotation_y(-30_f32.to_radians()),
        ),
    ));

    let ui_material = materials.add(Material::new(&window.facade, "ui", None));

    commands.spawn(UIRect::new(
        Val::Percent(0.0),
        Val::Percent(0.0),
        Val::Px(80.0),
        Val::Px(80.0),
        ui_material,
    ));

    commands.spawn(UIRect::new(
        Val::Percent(50.0),
        Val::Percent(50.0),
        Val::Percent(1.0),
        Val::Percent(1.0),
        ui_material,
    ));

    let verts = Quad::DEFAULT
        .corners
        .iter()
        .enumerate()
        .map(|c| UIVertex { corner: c.0 as u32 })
        .collect::<Vec<_>>();

    let inds = (0..verts.len())
        .step_by(4)
        .flat_map(|i| {
            let idx = i as u32;
            [idx, idx + 1, idx + 2, idx, idx + 2, idx + 3]
        })
        .collect::<Vec<_>>();

    let vertex_buffer = VertexBuffer::new(&window.facade, &verts).unwrap();
    let index_buffer =
        IndexBuffer::new(&window.facade, PrimitiveType::TrianglesList, &inds).unwrap();
    ui_meshes.add(Mesh::new(vertex_buffer, index_buffer));

    let voxel_material = materials.add(Material::new(&window.facade, "voxel", Some("atlas.png")));

    const CHUNK_SIZE_VEC: Vec3 = Vec3::splat(CHUNK_SIZE as f32);

    let mut chunks = HashMap::new();

    let mut noise = FastNoiseLite::new();
    noise.set_noise_type(Some(fastnoise_lite::NoiseType::Perlin));
    noise.set_frequency(Some(0.01));

    for cy in 0..4 {
        for cz in -8..8 {
            for cx in -8..8 {
                let chunk_pos = IVec3::new(cx, cy, cz);
                let mut chunk = Chunk::new(chunk_pos);

                for z in 0..CHUNK_SIZE {
                    for x in 0..CHUNK_SIZE {
                        let global_x = x + chunk_pos.x * CHUNK_SIZE;
                        let global_z = z + chunk_pos.z * CHUNK_SIZE;

                        let max_y = (noise.get_noise_2d(global_x as f32, global_z as f32) + 1.0)
                            * 0.5
                            * 64.0;
                        let max_y = max_y as i32;

                        for y in 0..CHUNK_SIZE {
                            let global_y = y + chunk_pos.y * CHUNK_SIZE;
                            let block_pos = ivec3(x, y, z);

                            chunk.blocks[vec3_to_index(block_pos)] =
                                generate_block_at(ivec3(global_x, global_y, global_z), max_y);
                        }
                    }
                }

                chunks.insert(chunk_pos, chunk);
            }
        }
    }

    for (chunk_pos, chunk) in &chunks {
        if let Some(mesh_data) = ChunkMesh::default().build(chunk, &chunks) {
            let vertex_buffer = VertexBuffer::new(&window.facade, &mesh_data.vertices).unwrap();
            let index_buffer = IndexBuffer::new(
                &window.facade,
                PrimitiveType::TrianglesList,
                &mesh_data.indices,
            )
            .unwrap();

            let mesh_id = voxel_meshes.add(Mesh::new(vertex_buffer, index_buffer));

            commands.spawn((
                mesh_id,
                voxel_material,
                Transform::from_translation((chunk_pos * CHUNK_SIZE).as_vec3()),
                Aabb::new(Vec3::ZERO, CHUNK_SIZE_VEC),
            ));
        }
    }
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn render_update(
    window: NonSend<NSWindow>,
    ui_meshes: NonSend<Meshes<UIVertex>>,
    voxel_meshes: NonSend<Meshes<VoxelVertex>>,
    materials: NonSend<Materials>,
    mesh_entities: Query<
        (&Transform, &Mesh3d, &MeshMaterial, &Aabb),
        (Without<Camera3d>, Without<DirectionalLight>),
    >,
    camera_query: Single<(&mut Transform, &Camera3d), (Without<Mesh3d>, Without<DirectionalLight>)>,
    light_query: Single<(&Transform, &DirectionalLight), (Without<Mesh3d>, Without<Camera3d>)>,
    ui_query: Query<&UIRect>,
    debug_info: Option<ResMut<DebugInfo>>,
    mut egui: NonSendMut<EguiGlium>,
) {
    let mut target = window.facade.draw();
    target.clear_color_and_depth((0.0, 0.0, 1.0, 1.0), 1.0);
    let (width, height) = target.get_dimensions();

    let mut draw_calls = 0;
    let mut vertices = 0;
    let mut indices = 0;

    // chunks
    {
        let (camera_transform, camera) = camera_query.into_inner();
        let (light_transform, light) = light_query.into_inner();

        let perspective = Mat4::perspective_rh_gl(
            camera.fov.to_radians(),
            width as f32 / height as f32,
            camera.near,
            camera.far,
        );

        let view = camera_transform.as_mat4().inverse();
        let vp: Mat4 = perspective * view;
        let frustum = frustum_planes(&vp);

        for (chunk_transform, mesh_id, material_id, aabb) in mesh_entities.iter() {
            if should_cull(&frustum, chunk_transform.translation, aabb) {
                continue;
            }
            let Mesh {
                vertex_buffer,
                index_buffer,
            } = &voxel_meshes.0[mesh_id.0];
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
                u_light: (view * Mat4::from_quat(light_transform.rotation) * Vec4::NEG_Z).truncate().normalize().extend(light.illuminance).to_array(),
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
    }

    // TODO shadow mapping
    {}

    // ui
    {
        let window_size = vec2(width as f32, height as f32);

        for ui_item in ui_query.iter() {
            let Mesh {
                vertex_buffer,
                index_buffer,
            } = &ui_meshes.0[0]; // 1x1 quad
            let Material { program, texture } = &materials.0[ui_item.material.0];

            let sampler = texture
                .sampled()
                .magnify_filter(MagnifySamplerFilter::Nearest)
                .minify_filter(MinifySamplerFilter::NearestMipmapNearest);

            let pos = [
                ui_item.x.calculate(window_size.x) - 1.0,
                1.0 - ui_item.y.calculate(window_size.y),
            ];
            let size = [
                ui_item.width.calculate(window_size.x),
                -ui_item.height.calculate(window_size.y),
            ];

            let uniforms = uniform! {
                pos: pos,
                size: size,
                tex: sampler,
            };

            vertices += vertex_buffer.len();
            indices += index_buffer.len();
            draw_calls += 1;

            target
                .draw(
                    vertex_buffer,
                    index_buffer,
                    program,
                    &uniforms,
                    &DrawParameters::default(),
                )
                .unwrap();
        }
    }

    if let Some(mut debug_info) = debug_info {
        debug_info.vertices = vertices;
        debug_info.indices = indices;
        debug_info.draw_calls = draw_calls;
    }
    egui.paint(&window.facade, &mut target);
    target.finish().unwrap();
}
