use glium::winit::keyboard::KeyCode;

use crate::{ecs::*, utils::set_cursor_grab};

pub fn handle_movement(
    mut camera: Single<&mut Transform, With<Camera3d>>,
    keyboard: Res<KeyboardInput>,
    mouse: Res<MouseInput>,
    time: Res<Time>,
    mut window: ResMut<Window>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        let grab = window.cursor_visible;
        set_cursor_grab(&mut window, grab);
    }

    let mut move_dir = Vec3::ZERO;

    let local_z = camera.rotation * Vec3::Z;
    let forward = -Vec3::new(local_z.x, 0.0, local_z.z).normalize_or_zero();
    let right = Vec3::new(local_z.z, 0.0, -local_z.x).normalize_or_zero();

    if keyboard.pressed(KeyCode::KeyW) {
        move_dir += forward;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        move_dir -= forward;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        move_dir += right;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        move_dir -= right;
    }
    if keyboard.pressed(KeyCode::Space) {
        move_dir += Vec3::Y;
    }
    if keyboard.pressed(KeyCode::ShiftLeft) {
        move_dir -= Vec3::Y;
    }

    let (mut yaw, mut pitch, _) = camera.rotation.to_euler(EulerRot::YXZ);
    let window_scale = window.height.max(window.width) as f32;
    // idk
    pitch -= (1.2 * mouse.motion.y * window_scale / 10_000.0).to_radians();
    yaw -= (1.2 * mouse.motion.x * window_scale / 10_000.0).to_radians();

    pitch = pitch.clamp(-1.54, 1.54);

    camera.rotation = Quat::from_rotation_y(yaw) * Quat::from_rotation_x(pitch);

    camera.translation += move_dir.normalize_or_zero() * 50.0 * time.delta_secs();
}
