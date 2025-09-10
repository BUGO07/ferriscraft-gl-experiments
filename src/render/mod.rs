use gl::types::*;
use glfw::Context;

use crate::{
    App,
    ecs::*,
    player::Projectile,
    render::{
        material::{Material, MaterialOptions, UniformValue},
        mesh::Mesh,
        primitives::{Cuboid, PrimitiveVertex, Quad},
    },
    ui::{TextVertex, UIRect, UIText},
    utils::{should_cull_aabb, should_cull_sphere},
    world::mesher::Direction,
};

pub mod material;
pub mod mesh;
pub mod primitives;

pub fn render_plugin(app: &mut App) {
    let mut materials = Materials::default();

    // materials[0] // voxel
    materials.add(
        Material::new(
            "voxel",
            MaterialOptions {
                base_texture: Some("assets/atlas.png"),
                ..Default::default()
            },
        )
        .unwrap(),
    );

    // materials[1] // primitive
    materials.add(
        Material::new(
            "primitive",
            MaterialOptions {
                base_color: Some(Vec4::new(0.8, 0.8, 0.8, 1.0)),
                ..Default::default()
            },
        )
        .unwrap(),
    );
    app.init_resource::<Meshes>()
        .insert_non_send_resource(materials)
        .add_systems(Startup, setup)
        .add_systems(
            RenderUpdate,
            calculations
                .pipe(render_world)
                .pipe(render_projectiles)
                .pipe(render_skybox)
                .pipe(render_ui),
        )
        .add_systems(PostRenderUpdate, finish_up);
}

fn setup(mut commands: Commands, mut materials: NonSendMut<Materials>) {
    let mut texture_id: GLuint = 0;
    let mut vao: GLuint = 0;
    let mut vbo: GLuint = 0;

    unsafe {
        gl::GenBuffers(1, &mut vbo);

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            size_of_val(CUBEMAP_VERTICES) as isize,
            CUBEMAP_VERTICES.as_ptr() as *const _,
            gl::STATIC_DRAW,
        );

        gl::GenVertexArrays(1, &mut vao);

        gl::BindVertexArray(vao);
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            3 * size_of::<GLfloat>() as GLint,
            std::ptr::null(),
        );
        gl::EnableVertexAttribArray(0);

        gl::GenTextures(1, &mut texture_id);
        gl::BindTexture(gl::TEXTURE_CUBE_MAP, texture_id);
        for i in 0..6 {
            let img = image::open(format!("assets/skybox/face{}.png", i)).unwrap();

            gl::TexImage2D(
                gl::TEXTURE_CUBE_MAP_POSITIVE_X + i,
                0,
                gl::RGB as GLint,
                img.width() as GLint,
                img.height() as GLint,
                0,
                gl::RGB,
                gl::UNSIGNED_BYTE,
                img.to_rgb8().into_raw().as_ptr() as *const _,
            );
        }

        gl::TexParameteri(
            gl::TEXTURE_CUBE_MAP,
            gl::TEXTURE_MIN_FILTER,
            gl::LINEAR as GLint,
        );
        gl::TexParameteri(
            gl::TEXTURE_CUBE_MAP,
            gl::TEXTURE_MAG_FILTER,
            gl::LINEAR as GLint,
        );
        gl::TexParameteri(
            gl::TEXTURE_CUBE_MAP,
            gl::TEXTURE_WRAP_S,
            gl::CLAMP_TO_EDGE as GLint,
        );
        gl::TexParameteri(
            gl::TEXTURE_CUBE_MAP,
            gl::TEXTURE_WRAP_T,
            gl::CLAMP_TO_EDGE as GLint,
        );
        gl::TexParameteri(
            gl::TEXTURE_CUBE_MAP,
            gl::TEXTURE_WRAP_R,
            gl::CLAMP_TO_EDGE as GLint,
        );
    }

    let material_id = materials
        .add(Material::new("skybox", MaterialOptions::default()).unwrap())
        .0;

    commands.insert_resource(Skybox {
        material_id,
        texture_id,
        vao,
        vbo,
    });
}

fn calculations(
    query: Single<(&mut Transform, &Camera3d)>,
    window: Res<Window>,
) -> (Mat4, Mat4, [Vec4; 6]) {
    let (camera_transform, camera) = query.into_inner();

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
    (projection, view, frustum)
}

fn render_world(
    vp: In<(Mat4, Mat4, [Vec4; 6])>,
    meshes: Res<Meshes>,
    materials: NonSend<Materials>,
    mesh_entities: Query<(&Transform, &Mesh3d, &MeshMaterial, &Aabb), Without<DirectionalLight>>,
    light_query: Single<(&Transform, &DirectionalLight)>,
    #[cfg(debug_assertions)] mut debug_info: ResMut<DebugInfo>,
) -> (Mat4, Mat4, [Vec4; 6]) {
    let (light_transform, light) = light_query.into_inner();
    let (projection, view, frustum) = *vp;

    unsafe {
        gl::Enable(gl::MULTISAMPLE);
        gl::Enable(gl::DEPTH_TEST);
        gl::Enable(gl::CULL_FACE);
        gl::CullFace(gl::BACK);
        gl::ClearColor(0.44, 0.73, 0.88, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }

    // main pass
    {
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

            let _triangles = mesh.draw();

            #[cfg(debug_assertions)]
            {
                debug_info.triangles += _triangles;
                debug_info.draw_calls += 1;
            }
        }
    }

    // TODO shadow mapping
    {}

    (projection, view, frustum)
}

fn render_projectiles(
    vp: In<(Mat4, Mat4, [Vec4; 6])>,
    materials: NonSend<Materials>,
    query: Query<(&Transform, &Projectile), Without<Camera3d>>,
    #[cfg(debug_assertions)] mut debug_info: ResMut<DebugInfo>,
) -> (Mat4, Mat4) {
    unsafe {
        gl::Enable(gl::DEPTH_TEST);
        gl::Enable(gl::CULL_FACE);
        gl::CullFace(gl::BACK);
    }

    let (projection, view, frustum) = *vp;

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

        let _triangles = mesh.draw();

        #[cfg(debug_assertions)]
        {
            debug_info.triangles += _triangles;
            debug_info.draw_calls += 1;
        }
    }

    (projection, view)
}

fn render_skybox(vp: In<(Mat4, Mat4)>, materials: NonSend<Materials>, skybox: Res<Skybox>) {
    let (projection, view) = *vp;
    unsafe {
        gl::DepthMask(gl::FALSE);
        gl::DepthFunc(gl::LEQUAL);

        let material = &materials.0[skybox.material_id];
        material.bind();
        material.set_uniform(c"projection", UniformValue::Mat4(projection));
        material.set_uniform(
            c"view",
            UniformValue::Mat4(Mat4::from_quat(view.to_scale_rotation_translation().1)),
        );

        gl::BindVertexArray(skybox.vao);
        gl::BindTexture(gl::TEXTURE_CUBE_MAP, skybox.texture_id);
        gl::DrawArrays(gl::TRIANGLES, 0, 36);
        gl::BindVertexArray(0);

        gl::DepthMask(gl::TRUE);
        gl::DepthFunc(gl::LESS);
    }
}

fn render_ui(
    materials: NonSend<Materials>,
    text_query: Query<&UIText>,
    rect_query: Query<&UIRect>,
    window: Res<Window>,
    #[cfg(debug_assertions)] mut debug_info: ResMut<DebugInfo>,
) {
    const CHARACTERS: &str =
        "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+-=()[]{}<>/*:#%!?.,'\"@&$";

    let window_size = vec2(window.width as f32, window.height as f32);

    unsafe {
        gl::Disable(gl::DEPTH_TEST);
        gl::Disable(gl::CULL_FACE);
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
    }

    for ui_rect in rect_query.iter() {
        let material = &materials.0[ui_rect.material.0];

        let quad = Quad::new(
            Direction::Front,
            vec3(
                ui_rect.x.calculate(window_size.x) - 1.0,
                1.0 - ui_rect.y.calculate(window_size.y),
                0.0,
            ),
            vec3(
                ui_rect.width.calculate(window_size.x),
                -ui_rect.height.calculate(window_size.y),
                0.0,
            ),
        );

        let vertices = quad
            .iter()
            .map(|pos| PrimitiveVertex { pos: *pos })
            .collect::<Vec<_>>();

        if let Ok(mesh) = Mesh::new(&vertices, &Cuboid::generate_indices(vertices.len())) {
            material.bind();

            let _triangles = mesh.draw();

            #[cfg(debug_assertions)]
            {
                debug_info.triangles += _triangles;
                debug_info.draw_calls += 1;
            }
        }
    }

    for ui_text in text_query.iter() {
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
                        pos: [base_x + char_index as f32 * char_width - 1.0, 1.0 - base_y],
                        char_id: i as u32,
                    };
                    vertices.extend_from_slice(&[vert, vert, vert, vert]);
                }
            }
            base_y += char_height;
        }

        if let Ok(mesh) = Mesh::new(&vertices, &Cuboid::generate_indices(vertices.len())) {
            material.bind();
            material.set_uniform(
                c"u_size",
                UniformValue::Vec2(Vec2::new(char_width, -char_height)),
            );

            let _triangles = mesh.draw();

            #[cfg(debug_assertions)]
            {
                debug_info.triangles += _triangles;
                debug_info.draw_calls += 1;
            }
        }
    }
}

pub fn finish_up(
    mut ns_window: NonSendMut<NSWindow>,
    #[cfg(debug_assertions)] mut debug_info: ResMut<DebugInfo>,
) {
    #[cfg(debug_assertions)]
    {
        println!(
            "draw calls: {}, triangles: {}, vertices: {}k",
            debug_info.draw_calls,
            debug_info.triangles,
            debug_info.triangles * 3 / 1000
        );
        debug_info.draw_calls = 0;
        debug_info.triangles = 0
    }
    ns_window.window.swap_buffers();
}

#[rustfmt::skip]
pub const CUBEMAP_VERTICES: &[[f32; 3]] = &[
    [-1.0,  1.0, -1.0],
    [-1.0, -1.0, -1.0],
    [ 1.0, -1.0, -1.0],
    [ 1.0, -1.0, -1.0],
    [ 1.0,  1.0, -1.0],
    [-1.0,  1.0, -1.0],
    [-1.0, -1.0,  1.0],
    [-1.0, -1.0, -1.0],
    [-1.0,  1.0, -1.0],
    [-1.0,  1.0, -1.0],
    [-1.0,  1.0,  1.0],
    [-1.0, -1.0,  1.0],
    [ 1.0, -1.0, -1.0],
    [ 1.0, -1.0,  1.0],
    [ 1.0,  1.0,  1.0],
    [ 1.0,  1.0,  1.0],
    [ 1.0,  1.0, -1.0],
    [ 1.0, -1.0, -1.0],
    [-1.0, -1.0,  1.0],
    [-1.0,  1.0,  1.0],
    [ 1.0,  1.0,  1.0],
    [ 1.0,  1.0,  1.0],
    [ 1.0, -1.0,  1.0],
    [-1.0, -1.0,  1.0],
    [-1.0,  1.0, -1.0],
    [ 1.0,  1.0, -1.0],
    [ 1.0,  1.0,  1.0],
    [ 1.0,  1.0,  1.0],
    [-1.0,  1.0,  1.0],
    [-1.0,  1.0, -1.0],
    [-1.0, -1.0, -1.0],
    [-1.0, -1.0,  1.0],
    [ 1.0, -1.0, -1.0],
    [ 1.0, -1.0, -1.0],
    [-1.0, -1.0,  1.0],
    [ 1.0, -1.0,  1.0],
];
