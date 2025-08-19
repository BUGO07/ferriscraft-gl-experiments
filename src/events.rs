use glium::winit::{event::WindowEvent, window::CursorGrabMode};

use crate::ecs::*;

#[derive(Event)]
pub struct WindowEventECS(pub WindowEvent);

pub fn handle_input_cleanup(mut keyboard: ResMut<KeyboardInput>, mut mouse: ResMut<MouseInput>) {
    keyboard.just_pressesd.clear();
    keyboard.just_released.clear();

    mouse.just_pressesd.clear();
    mouse.just_released.clear();
    mouse.motion = Vec2::ZERO;
    mouse.scroll = Vec2::ZERO;
}

pub fn handle_window(
    ns_window: NonSendMut<NSWindow>,
    mut window: ResMut<Window>,
    mut not_first_run: Local<bool>,
) {
    if !*not_first_run {
        // don't do anything on the first frame
        // changing cursor visibility doesn't work on the first frame
        *not_first_run = true;
        return;
    }
    match window.cursor_grab {
        CursorGrabMode::None => {
            ns_window
                .winit
                .set_cursor_grab(CursorGrabMode::None)
                .unwrap();
        }
        _ => ns_window
            .winit
            .set_cursor_grab(CursorGrabMode::Locked)
            .or_else(|_| ns_window.winit.set_cursor_grab(CursorGrabMode::Confined))
            .unwrap(),
    }

    ns_window.winit.set_cursor_visible(window.cursor_visible);

    window.height = ns_window.winit.inner_size().height;
    window.width = ns_window.winit.inner_size().width;
}
