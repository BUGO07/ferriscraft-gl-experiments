use glfw::{Context, Key};

use crate::{
    App,
    ecs::*,
    render::{material::UniformValue, mesh::Mesh},
    ui::UIRect,
    utils::should_cull,
};

// mod inspector;
pub mod material;
pub mod mesh;

pub fn render_plugin(app: &mut App) {
    app
        // .add_systems(EguiContextPass, inspector::handle_egui)
        .add_systems(RenderUpdate, render_update);
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn render_update(
    mut ns_window: NonSendMut<NSWindow>,
    meshes: NonSend<Meshes>,
    materials: NonSend<Materials>,
    mesh_entities: Query<
        (&Transform, &Mesh3d, &MeshMaterial, &Aabb),
        (Without<Camera3d>, Without<DirectionalLight>),
    >,
    camera_query: Single<(&mut Transform, &Camera3d), (Without<Mesh3d>, Without<DirectionalLight>)>,
    light_query: Single<(&Transform, &DirectionalLight), (Without<Mesh3d>, Without<Camera3d>)>,
    ui_query: Query<&UIRect>,
    debug_info: Option<ResMut<DebugInfo>>,
    keyboard: Res<KeyboardInput>,
    time: Res<Time>,
    // mut egui: NonSendMut<EguiGlium>,
    mut last_frames: Local<(u32, f32)>, // frame amount accumulated, last_time
    mut disable_ao: Local<bool>,
) {
    last_frames.0 += 1;
    if last_frames.1 + 1.0 < time.elapsed_secs() {
        ns_window
            .window
            .set_title(format!("FerrisCraft GL - FPS: {}", last_frames.0).as_str()); // maybe update ui instead of title
        last_frames.1 = time.elapsed_secs();
        last_frames.0 = 0;
    }
    unsafe {
        gl::Enable(gl::DEPTH_TEST);
        gl::ClearColor(0.44, 0.73, 0.88, 1.0);
        gl::ClearDepthf(1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        gl::Enable(gl::CULL_FACE);
        gl::CullFace(gl::BACK);
    }
    let (width, height) = ns_window.window.get_size();

    let mut draw_calls = 0;
    let mut indices = 0;

    // temporary
    if keyboard.just_pressed(Key::F1) {
        *disable_ao = !*disable_ao;
    }

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
        let frustum = {
            let row1 = vp.row(0);
            let row2 = vp.row(1);
            let row3 = vp.row(2);
            let row4 = vp.row(3);

            // left right bottom top near far
            [
                row4 + row1,
                row4 - row1,
                row4 + row2,
                row4 - row2,
                row4 + row3,
                row4 - row3,
            ]
        };

        for (chunk_transform, mesh_id, material_id, aabb) in mesh_entities.iter() {
            if should_cull(&frustum, chunk_transform.translation, aabb) {
                continue;
            }
            let Some(mesh) = &meshes.0.get(&mesh_id.0) else {
                continue;
            };
            let Ok(mesh) = Mesh::new(&mesh.vertices, &mesh.indices) else {
                continue;
            };
            let material = &materials.0[material_id.0];

            material.bind();
            material.set_uniform("perspective", UniformValue::Mat4(perspective));
            material.set_uniform("view", UniformValue::Mat4(view));
            material.set_uniform("model", UniformValue::Mat4(chunk_transform.as_mat4()));
            material.set_uniform(
                "u_light",
                UniformValue::Vec4(
                    (view * Mat4::from_quat(light_transform.rotation) * Vec4::NEG_Z)
                        .truncate()
                        .normalize()
                        .extend(light.illuminance),
                ),
            );
            material.set_uniform("disable_ao", UniformValue::Bool(*disable_ao));
            mesh.draw();

            indices += mesh.index_count;
            draw_calls += 1;
        }
    }

    // TODO shadow mapping
    {}

    // ui
    {
        let window_size = vec2(width as f32, height as f32);

        for ui_item in ui_query.iter() {
            let Ok(mesh) = Mesh::new(&[0, 0, 0, 0], &[0, 1, 2, 0, 2, 3]) else {
                continue;
            };
            // 1x1 quad
            let material = &materials.0[ui_item.material.0];

            let pos = Vec2::new(
                ui_item.x.calculate(window_size.x) - 1.0,
                1.0 - ui_item.y.calculate(window_size.y),
            );
            let size = Vec2::new(
                ui_item.width.calculate(window_size.x),
                -ui_item.height.calculate(window_size.y),
            );

            material.bind();
            material.set_uniform("pos", UniformValue::Vec2(pos));
            material.set_uniform("size", UniformValue::Vec2(size));
            mesh.draw();

            indices += mesh.index_count;
            draw_calls += 1;
        }
    }

    if let Some(mut debug_info) = debug_info {
        debug_info.indices = indices as usize;
        debug_info.draw_calls = draw_calls;
    }
    ns_window.window.swap_buffers();
    // egui.paint(&ns_window.context, &mut target);
    // target.finish().unwrap();
}
