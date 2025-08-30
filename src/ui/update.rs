use hlua::Lua;

use crate::{
    CHUNK_SIZE,
    ecs::*,
    ui::{DebugText, UIText},
};

pub fn update_ui(
    mut debug_text: Single<&mut UIText, With<DebugText>>,
    mut last_frames: Local<(u32, f32, u32, f32)>, // frame amount accumulated, last_time, last_fps, frame delta accumulated
    time: Res<Time>,
    player: Single<&Transform, With<Camera3d>>,
    mut lua: NonSendMut<Lua>,
) {
    let pt = player.translation;
    let chunk_pos = (pt / CHUNK_SIZE as f32).as_ivec3();

    let local_block_pos = vec3(
        pt.x.rem_euclid(CHUNK_SIZE as f32),
        pt.y.rem_euclid(CHUNK_SIZE as f32),
        pt.z.rem_euclid(CHUNK_SIZE as f32),
    )
    .as_ivec3();

    let rot = player.rotation.to_euler(EulerRot::YXZ);

    let facing = match (360.0 - rot.0.to_degrees()) % 360.0 {
        x if (22.5..67.5).contains(&x) => "NE",
        x if (67.5..112.5).contains(&x) => "E",
        x if (112.5..157.5).contains(&x) => "SE",
        x if (157.5..202.5).contains(&x) => "S",
        x if (202.5..247.5).contains(&x) => "SW",
        x if (247.5..292.5).contains(&x) => "W",
        x if (292.5..337.5).contains(&x) => "NW",
        _ => "N",
    };

    // hell nawww
    last_frames.0 += 1;
    last_frames.3 += time.delta_secs();
    if last_frames.1 + 0.25 < time.elapsed_secs() {
        last_frames.2 = (last_frames.0 as f32 / last_frames.3) as u32;
        last_frames.1 = time.elapsed_secs();
        last_frames.0 = 0;
        last_frames.3 = 0.0;
    }

    let x: i32 = lua.get("x").unwrap_or(0);
    debug_text.text = format!(
        "FPS:    {}\nXYZ:    {:.2}\nChunk:  {:.2}\nBlock:  {:.2}\nFacing: {} / {}'/ {}'\nLUA - {}",
        last_frames.2,
        pt,
        chunk_pos,
        local_block_pos,
        facing,
        -rot.0.to_degrees() as i32,
        -rot.1.to_degrees() as i32,
        x
    )
}
