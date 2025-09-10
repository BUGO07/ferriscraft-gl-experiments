use glfw::Key;

use crate::{ecs::*, utils::set_cursor_grab};

pub fn handle_movement(
    camera: Single<(&mut Transform, &mut Camera3d)>,
    keyboard: Res<KeyboardInput>,
    mouse: Res<MouseInput>,
    time: Res<Time>,
    mut window: ResMut<Window>,
) {
    let (mut transform, mut camera) = camera.into_inner();
    if keyboard.just_pressed(Key::Escape) {
        let grab = window.cursor_visible;
        set_cursor_grab(&mut window, grab);
    }

    if !window.cursor_grab {
        return;
    }

    let mut move_dir = Vec3::ZERO;
    let mut speed = 25.0;

    let local_z = transform.rotation * Vec3::Z;
    let forward = -Vec3::new(local_z.x, 0.0, local_z.z).normalize_or_zero();
    let right = Vec3::new(local_z.z, 0.0, -local_z.x).normalize_or_zero();

    for key in keyboard.pressed.iter() {
        match key {
            Key::W => move_dir += forward,
            Key::S => move_dir -= forward,
            Key::D => move_dir += right,
            Key::A => move_dir -= right,
            Key::Space => move_dir += Vec3::Y,
            Key::LeftShift => move_dir -= Vec3::Y,
            Key::LeftControl => speed *= 10.0,
            _ => {}
        }
    }

    if keyboard.just_pressed(Key::C) {
        camera.fov = 15.0;
    }

    if keyboard.pressed(Key::C) {
        camera.fov = (camera.fov - mouse.scroll.y).clamp(1.0, 90.0);
    }

    if keyboard.just_released(Key::C) {
        camera.fov = 60.0;
    }

    let (mut yaw, mut pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
    let window_scale = window.height.max(window.width) as f32;
    // idk
    pitch -= (1.2 * mouse.motion.y * window_scale / 10_000.0).to_radians();
    yaw -= (1.2 * mouse.motion.x * window_scale / 10_000.0).to_radians();

    pitch = pitch.clamp(-1.54, 1.54);

    transform.rotation = Quat::from_rotation_y(yaw) * Quat::from_rotation_x(pitch);

    transform.translation += move_dir.normalize_or_zero() * speed * time.delta_secs();
}
