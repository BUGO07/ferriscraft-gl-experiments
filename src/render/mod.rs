use gl::types::*;
use glfw::{Context, Key};

use crate::{
    App,
    ecs::*,
    particles::ParticleEmitter,
    player::Projectile,
    render::{
        material::UniformValue,
        mesh::{Mesh, Vertex},
    },
    ui::UIText,
    utils::{Quad, should_cull},
    world::mesher::Direction,
};

pub mod material;
pub mod mesh;

pub fn render_plugin(app: &mut App) {
    app.add_systems(RenderUpdate, render_update);
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn render_update(
    mut ns_window: NonSendMut<NSWindow>,
    meshes: NonSend<Meshes>,
    materials: NonSend<Materials>,
    mesh_entities: Query<
        (&Transform, &Mesh3d, &MeshMaterial, &Aabb),
        (
            Without<Camera3d>,
            Without<DirectionalLight>,
            Without<Projectile>,
        ),
    >,
    projectile_query: Query<(&Transform, &Projectile)>,
    camera_query: Single<
        (&mut Transform, &Camera3d),
        (
            Without<Mesh3d>,
            Without<DirectionalLight>,
            Without<Projectile>,
        ),
    >,
    light_query: Single<(&Transform, &DirectionalLight), (Without<Mesh3d>, Without<Camera3d>)>,
    particle_emmiters: Query<&ParticleEmitter>,
    ui_query: Query<&UIText>,
    debug_info: Option<ResMut<DebugInfo>>,
    keyboard: Res<KeyboardInput>,
    // mut egui: NonSendMut<EguiGlium>,
    skybox: ResMut<Skybox>,
    mut disable_ao: Local<bool>,
) {
    unsafe {
        gl::Enable(gl::MULTISAMPLE);
        gl::Enable(gl::DEPTH_TEST);
        gl::ClearColor(0.44, 0.73, 0.88, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        gl::Enable(gl::CULL_FACE);
        gl::CullFace(gl::BACK);
    }
    let (width, height) = ns_window.window.get_framebuffer_size();

    let mut draw_calls = 0;
    let mut indices = 0;

    // temporary
    if keyboard.just_pressed(Key::F1) {
        *disable_ao = !*disable_ao;
    }

    // main pass
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

        // skybox
        unsafe {
            gl::DepthMask(gl::FALSE);
            gl::DepthFunc(gl::LEQUAL);

            let material = &materials.0[3];
            material.bind();
            material.set_uniform(
                c"view",
                UniformValue::Mat4(Mat4::from_quat(camera_transform.rotation).inverse()),
            );
            material.set_uniform(c"perspective", UniformValue::Mat4(perspective));

            gl::BindVertexArray(skybox.vao);
            gl::BindTexture(gl::TEXTURE_CUBE_MAP, skybox.texture_id);
            gl::DrawArrays(gl::TRIANGLES, 0, 36);
            gl::BindVertexArray(0);

            gl::DepthMask(gl::TRUE);
            gl::DepthFunc(gl::LESS);
        }

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

        for (proj_transform, projectile) in projectile_query.iter() {
            use Direction::*;

            let mut vertices = Vec::new();
            for offset in -1..=1 {
                // idk can be better
                for dir in [Left, Right, Bottom, Top, Back, Front] {
                    let size = 0.25;
                    let quad = Quad::from_direction(
                        dir,
                        (dir.as_ivec3().as_vec3().max(Vec3::ZERO)
                            + offset as f32 * projectile.direction)
                            * size,
                        Vec3::ONE * size,
                    );
                    for pos in quad.corners {
                        vertices.push(ProjectileVertex { pos });
                    }
                }
            }
            let indices = (0..vertices.len())
                .step_by(4)
                .flat_map(|i| {
                    let idx = i as u32;
                    [idx, idx + 1, idx + 2, idx, idx + 2, idx + 3]
                })
                .collect::<Vec<_>>();

            let Ok(mesh) = Mesh::new(&vertices, &indices) else {
                continue;
            };
            let material = &materials.0[1]; // projectile
            material.bind();
            material.set_uniform(c"perspective", UniformValue::Mat4(perspective));
            material.set_uniform(c"view", UniformValue::Mat4(view));
            material.set_uniform(c"model", UniformValue::Mat4(proj_transform.as_mat4()));
            mesh.draw();
        }

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
            material.set_uniform(c"perspective", UniformValue::Mat4(perspective));
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
            material.set_uniform(c"disable_ao", UniformValue::Bool(*disable_ao));
            mesh.draw();

            indices += mesh.index_count;
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

    // ui
    {
        const CHARACTERS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+-=()[]{}<>/*:#%!?.,'\"@&$";

        let window_size = vec2(width as f32, height as f32);

        unsafe {
            gl::Disable(gl::DEPTH_TEST);
            gl::Disable(gl::CULL_FACE);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }

        for ui_text in ui_query.iter() {
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

                mesh.draw();
            }
        }
    }

    if let Some(mut debug_info) = debug_info {
        debug_info.indices = indices as usize;
        debug_info.draw_calls = draw_calls;
    }
    ns_window.window.swap_buffers();
}

#[repr(C)]
#[derive(Copy, Clone)]
struct ProjectileVertex {
    pos: [f32; 3],
}

impl Vertex for ProjectileVertex {
    fn attributes() -> &'static [(GLuint, GLint, GLenum, GLboolean, usize)] {
        &[(0, 3, gl::FLOAT, gl::FALSE, 0)]
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
struct TextVertex {
    position: [f32; 2],
    char_id: u32,
    // size: [f32; 2],
}

impl Vertex for TextVertex {
    fn attributes() -> &'static [(GLuint, GLint, GLenum, GLboolean, usize)] {
        &[
            (0, 2, gl::FLOAT, gl::FALSE, 0),
            (1, 1, gl::UNSIGNED_INT, gl::FALSE, size_of::<[f32; 2]>()),
            // (2, 2, gl::FLOAT, gl::FALSE, size_of::<[f32; 2]>()),
        ]
    }
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
