use glfw::Context;

use crate::{
    App,
    ecs::*,
    particles::ParticleEmitter,
    player::Projectile,
    render::{material::UniformValue, mesh::Mesh, primitives::Cuboid},
    ui::{TextVertex, UIText},
    utils::{should_cull_aabb, should_cull_sphere},
    world::mesher::Direction,
};

pub mod material;
pub mod mesh;
pub mod primitives;

pub fn render_plugin(app: &mut App) {
    app.init_resource::<Meshes>()
        .init_non_send_resource::<Materials>()
        .add_systems(Startup, setup)
        .add_systems(RenderUpdate, (render_ui, render_projectiles, render_update))
        .add_systems(PostRenderUpdate, finish_up);
}

fn setup() {}

fn render_projectiles(
    window: Res<Window>,
    materials: NonSend<Materials>,
    query: Query<(&Transform, &Projectile), Without<Camera3d>>,
    camera: Single<(&mut Transform, &Camera3d)>,
    debug_info: Option<ResMut<DebugInfo>>,
) {
    unsafe {
        gl::Enable(gl::MULTISAMPLE);
        gl::Enable(gl::DEPTH_TEST);
        gl::Enable(gl::CULL_FACE);
    }
    let (camera_transform, camera) = camera.into_inner();
    let projection = Mat4::perspective_rh_gl(
        camera.fov.to_radians(),
        window.width as f32 / window.height as f32,
        camera.near,
        camera.far,
    );

    let view = camera_transform.as_mat4().inverse();

    let frustum = {
        let vp = projection * view;
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

    let mut draw_calls = 0;
    let mut triangle_count = 0;

    for (proj_transform, projectile) in query.iter() {
        if should_cull_sphere(&frustum, proj_transform.translation, 0.5) {
            continue;
        }
        let mut vertices = Vec::new();
        for offset in -1..=1 {
            vertices.extend(Cuboid::new(
                Vec3::ONE * 0.25,
                offset as f32 * projectile.direction,
            ));
        }
        let indices = Cuboid::generate_indices(vertices.len());

        let Ok(mesh) = Mesh::new(&vertices, &indices) else {
            continue;
        };
        let material = &materials.0[1]; // primitive
        material.bind();
        material.set_uniform(c"projection", UniformValue::Mat4(projection));
        material.set_uniform(c"view", UniformValue::Mat4(view));
        material.set_uniform(c"model", UniformValue::Mat4(proj_transform.as_mat4()));
        triangle_count += mesh.draw();
        draw_calls += 1;
    }

    if let Some(mut debug_info) = debug_info {
        debug_info.triangles += triangle_count;
        debug_info.draw_calls += draw_calls;
    }
}

fn render_ui(
    materials: NonSend<Materials>,
    query: Query<&UIText>,
    window: Res<Window>,
    debug_info: Option<ResMut<DebugInfo>>,
) {
    const CHARACTERS: &str =
        "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+-=()[]{}<>/*:#%!?.,'\"@&$";

    let window_size = vec2(window.width as f32, window.height as f32);

    let mut triangle_count = 0;
    let mut draw_calls = 0;

    unsafe {
        gl::Disable(gl::DEPTH_TEST);
        gl::Disable(gl::CULL_FACE);
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
    }

    for ui_text in query.iter() {
        let material = &materials.0[ui_text.material.0];
        let char_width = ui_text.font_size.calculate(window_size.x);
        let char_height = ui_text.font_height.calculate(window_size.y);
        let base_x = ui_text.x.calculate(window_size.x);
        let mut base_y = ui_text.y.calculate(window_size.y);

        let mut vertices = Vec::new();

        for line in ui_text.text.split('\n') {
            for (char_index, character) in line.chars().enumerate() {
                if let Some(i) = CHARACTERS.find(character) {
                    let vert = TextVertex {
                        position: [base_x + char_index as f32 * char_width - 1.0, 1.0 - base_y],
                        char_id: i as u32,
                    };
                    vertices.extend_from_slice(&[vert, vert, vert, vert]);
                }
            }
            base_y += char_height;
        }

        if let Ok(mesh) = Mesh::new(
            &vertices,
            &(0..vertices.len())
                .step_by(4)
                .flat_map(|i| {
                    let idx = i as u32;
                    [idx, idx + 1, idx + 2, idx, idx + 2, idx + 3]
                })
                .collect::<Vec<_>>(),
        ) {
            material.bind();
            material.set_uniform(
                c"u_size",
                UniformValue::Vec2(Vec2::new(char_width, -char_height)),
            );

            triangle_count += mesh.draw();
            draw_calls += 1;
        }
    }

    if let Some(mut debug_info) = debug_info {
        debug_info.triangles += triangle_count;
        debug_info.draw_calls += draw_calls;
    }
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn render_update(
    window: Res<Window>,
    meshes: Res<Meshes>,
    materials: NonSend<Materials>,
    mesh_entities: Query<
        (&Transform, &Mesh3d, &MeshMaterial, &Aabb),
        (Without<Camera3d>, Without<DirectionalLight>),
    >,
    camera_query: Single<(&mut Transform, &Camera3d), (Without<Mesh3d>, Without<DirectionalLight>)>,
    light_query: Single<(&Transform, &DirectionalLight)>,
    particle_emmiters: Query<&ParticleEmitter>,
    debug_info: Option<ResMut<DebugInfo>>,
    // mut egui: NonSendMut<EguiGlium>,
    skybox: ResMut<Skybox>,
) {
    unsafe {
        gl::Enable(gl::MULTISAMPLE);
        gl::Enable(gl::DEPTH_TEST);
        gl::Enable(gl::CULL_FACE);
        gl::ClearColor(0.44, 0.73, 0.88, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        gl::CullFace(gl::BACK);
    }

    let (width, height) = (window.width, window.height);

    let mut draw_calls = 0;
    let mut triangle_count = 0;

    // main pass
    {
        let (camera_transform, camera) = camera_query.into_inner();
        let (light_transform, light) = light_query.into_inner();

        let projection = Mat4::perspective_rh_gl(
            camera.fov.to_radians(),
            width as f32 / height as f32,
            camera.near,
            camera.far,
        );

        let view = camera_transform.as_mat4().inverse();

        // skybox
        unsafe {
            gl::DepthMask(gl::FALSE);
            gl::DepthFunc(gl::LEQUAL);

            let material = &materials.0[skybox.material_id];
            material.bind();
            material.set_uniform(c"projection", UniformValue::Mat4(projection));
            material.set_uniform(
                c"view",
                UniformValue::Mat4(Mat4::from_quat(camera_transform.rotation).inverse()),
            );

            gl::BindVertexArray(skybox.vao);
            gl::BindTexture(gl::TEXTURE_CUBE_MAP, skybox.texture_id);
            gl::DrawArrays(gl::TRIANGLES, 0, 36);
            gl::BindVertexArray(0);

            gl::DepthMask(gl::TRUE);
            gl::DepthFunc(gl::LESS);
        }

        let frustum = {
            let vp = projection * view;
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
            if should_cull_aabb(&frustum, chunk_transform.translation, aabb) {
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
            material.set_uniform(c"projection", UniformValue::Mat4(projection));
            material.set_uniform(c"view", UniformValue::Mat4(view));
            material.set_uniform(c"model", UniformValue::Mat4(chunk_transform.as_mat4()));
            material.set_uniform(
                c"u_light",
                UniformValue::Vec4(
                    (view * Mat4::from_quat(light_transform.rotation) * Vec4::NEG_Z)
                        .truncate()
                        .normalize()
                        .extend(light.illuminance),
                ),
            );
            triangle_count += mesh.draw();
            draw_calls += 1;
        }
    }

    // particles
    {
        for _emmiter in particle_emmiters.iter() {
            // TODO
        }
    }

    // TODO shadow mapping
    {}

    if let Some(mut debug_info) = debug_info {
        debug_info.triangles += triangle_count;
        debug_info.draw_calls += draw_calls;
    }
}

pub fn finish_up(mut ns_window: NonSendMut<NSWindow>, debug_info: Option<ResMut<DebugInfo>>) {
    if let Some(mut debug_info) = debug_info {
        println!("{debug_info:?}");
        debug_info.draw_calls = 0;
        debug_info.triangles = 0
    }
    ns_window.window.swap_buffers();
}

#[rustfmt::skip]
pub const CUBEMAP_VERTICES: &[f32] = &[
    -1.0,  1.0, -1.0,
    -1.0, -1.0, -1.0,
     1.0, -1.0, -1.0,
     1.0, -1.0, -1.0,
     1.0,  1.0, -1.0,
    -1.0,  1.0, -1.0,

    -1.0, -1.0,  1.0,
    -1.0, -1.0, -1.0,
    -1.0,  1.0, -1.0,
    -1.0,  1.0, -1.0,
    -1.0,  1.0,  1.0,
    -1.0, -1.0,  1.0,

     1.0, -1.0, -1.0,
     1.0, -1.0,  1.0,
     1.0,  1.0,  1.0,
     1.0,  1.0,  1.0,
     1.0,  1.0, -1.0,
     1.0, -1.0, -1.0,

    -1.0, -1.0,  1.0,
    -1.0,  1.0,  1.0,
     1.0,  1.0,  1.0,
     1.0,  1.0,  1.0,
     1.0, -1.0,  1.0,
    -1.0, -1.0,  1.0,

    -1.0,  1.0, -1.0,
     1.0,  1.0, -1.0,
     1.0,  1.0,  1.0,
     1.0,  1.0,  1.0,
    -1.0,  1.0,  1.0,
    -1.0,  1.0, -1.0,

    -1.0, -1.0, -1.0,
    -1.0, -1.0,  1.0,
     1.0, -1.0, -1.0,
     1.0, -1.0, -1.0,
    -1.0, -1.0,  1.0,
     1.0, -1.0,  1.0
];
