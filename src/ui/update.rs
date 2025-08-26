use crate::{
    CHUNK_SIZE,
    ecs::*,
    ui::{CoordsText, UIText},
};

pub fn update_ui(
    mut cooords_text: Single<&mut UIText, With<CoordsText>>,
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

    let deg = player.rotation.to_euler(EulerRot::YXZ).0.to_degrees();
    let deg = 360.0 - if deg < 0.0 { deg + 360.0 } else { deg };

    let facing = match deg {
        x if !(22.5..337.5).contains(&x) => "N",
        x if (22.5..67.5).contains(&x) => "NE",
        x if (67.5..112.5).contains(&x) => "E",
        x if (112.5..157.5).contains(&x) => "SE",
        x if (157.5..202.5).contains(&x) => "S",
        x if (202.5..247.5).contains(&x) => "SW",
        x if (247.5..292.5).contains(&x) => "W",
        x if (292.5..337.5).contains(&x) => "NW",
        _ => "N",
    };

    cooords_text.text = format!(
        "XYZ:    {:.2}\nChunk:  {:.2}\nBlock:  {:.2}\nFacing: {} / {:.2}'",
        pt, chunk_pos, local_block_pos, facing, deg
    )
}
