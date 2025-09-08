use glfw::MouseButton;

use crate::{
    CHUNK_SIZE,
    ecs::*,
    ui::{Button, DebugText, UIRect, UIText},
};

pub fn handle_picking(
    query: Query<&UIRect, With<Button>>,
    mouse: Res<MouseInput>,
    window: Res<Window>,
) {
    let mouse_pos = mouse.position / vec2(window.width as f32, window.height as f32);
    let mouse_pos = vec2(mouse_pos.x * 2.0 - 1.0, 1.0 - mouse_pos.y * 2.0);
    for button in query.iter() {
        let pos = vec2(
            button.x.calculate(window.width as f32) - 1.0,
            1.0 - button.y.calculate(window.height as f32),
        );
        let size = vec2(
            button.width.calculate(window.width as f32),
            -button.height.calculate(window.height as f32),
        );
        if (pos.x..pos.x + size.x).contains(&mouse_pos.x)
            && (pos.y + size.y..pos.y).contains(&mouse_pos.y)
            && mouse.just_pressed(MouseButton::Left)
        {
            println!("pressed button")
        }
    }
}

pub fn update_ui(
    mut debug_text: Single<&mut UIText, With<DebugText>>,
    mut last_frames: Local<(u32, f64, u32, f64)>, // frame count, time, last fps, last update time
    time: Res<Time>,
    player: Single<&Transform, With<Camera3d>>,
) {
    let pt = player.translation;
    let chunk_pos = (pt / CHUNK_SIZE as f32).as_ivec3();

    let local_block_pos = vec3(
        pt.x.rem_euclid(CHUNK_SIZE as f32),
        pt.y.rem_euclid(CHUNK_SIZE as f32),
        pt.z.rem_euclid(CHUNK_SIZE as f32),
    )
    .as_ivec3();

    let (yaw, pitch, _) = player.rotation.to_euler(EulerRot::YXZ);

    let facing = match (360.0 - yaw.to_degrees()) % 360.0 {
        x if (22.5..67.5).contains(&x) => "NE",
        x if (67.5..112.5).contains(&x) => "E",
        x if (112.5..157.5).contains(&x) => "SE",
        x if (157.5..202.5).contains(&x) => "S",
        x if (202.5..247.5).contains(&x) => "SW",
        x if (247.5..292.5).contains(&x) => "W",
        x if (292.5..337.5).contains(&x) => "NW",
        _ => "N",
    };

    let (f, t, lf, lt) = &mut *last_frames;

    // hell nawww
    *f += 1;
    *t += time.delta_secs_f64();
    if *lt + 0.25 < time.elapsed_secs_f64() {
        *lf = (*f as f64 / *t) as u32;
        *lt = time.elapsed_secs_f64();
        *f = 0;
        *t = 0.0;
    }

    debug_text.text = format!(
        "FPS:    {}\nXYZ:    {:.2}\nChunk:  {:.2}\nBlock:  {:.2}\nFacing: {} / {}'/ {}'",
        *lf,
        pt,
        chunk_pos,
        local_block_pos,
        facing,
        -yaw.to_degrees() as i32,
        -pitch.to_degrees() as i32,
    )
}
