use egui_glium::EguiGlium;
use glium::{
    BackfaceCullingMode, Blend, Depth, DepthTest, DrawParameters, Surface,
    uniforms::{MagnifySamplerFilter, MinifySamplerFilter},
};

use crate::{
    App,
    ecs::*,
    ui::{UIRect, UIVertex},
    utils::{frustum_planes, should_cull},
    world::mesher::VoxelVertex,
};

mod inspector;

pub fn render_plugin(app: &mut App) {
    app.add_systems(EguiContextPass, inspector::handle_egui)
        .add_systems(RenderUpdate, render_update);
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn render_update(
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
        let vp = perspective * view;
        let frustum = frustum_planes(&vp);

        for (chunk_transform, mesh_id, material_id, aabb) in mesh_entities.iter() {
            if should_cull(&frustum, chunk_transform.translation, aabb) {
                continue;
            }
            let (vertex_buffer, index_buffer) = &voxel_meshes.0[mesh_id.0];
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
                blend: Blend::alpha_blending(),
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
            let (vertex_buffer, index_buffer) = &ui_meshes.0[0]; // 1x1 quad
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
