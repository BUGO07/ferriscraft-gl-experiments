use glium::winit::{
    event::{ElementState, WindowEvent},
    keyboard::PhysicalKey,
    window::CursorGrabMode,
};

use crate::{Application, ecs::*};

#[derive(Event)]
pub struct WindowEventECS(pub WindowEvent);

pub fn window_plugin(app: &mut Application) {
    app.world.init_resource::<Events<WindowEventECS>>();
    app.world.init_resource::<KeyboardInput>();
    app.world.init_resource::<MouseInput>();
    app.world.init_resource::<Time>();
    app.add_systems(PreUpdate, handle_events)
        .add_systems(PostUpdate, (handle_input_cleanup, handle_window));
}

pub fn handle_events(
    mut events: EventReader<WindowEventECS>,
    mut keyboard: ResMut<KeyboardInput>,
    mut mouse: ResMut<MouseInput>,
    ns_window: NonSend<NSWindow>,
) {
    for event in events.read() {
        match event.0 {
            WindowEvent::Resized(window_size) => {
                ns_window.facade.resize(window_size.into());
            }
            WindowEvent::KeyboardInput {
                device_id: _,
                ref event,
                is_synthetic: _,
            } => {
                if let PhysicalKey::Code(code) = event.physical_key {
                    match event.state {
                        ElementState::Pressed => {
                            keyboard.just_pressesd.insert(code);
                            keyboard.pressed.insert(code);
                        }
                        ElementState::Released => {
                            keyboard.just_released.insert(code);
                            keyboard.pressed.remove(&code);
                        }
                    }
                }
            }
            WindowEvent::MouseInput {
                device_id: _,
                state,
                button,
            } => match state {
                ElementState::Pressed => {
                    mouse.just_pressesd.insert(button);
                    mouse.pressed.insert(button);
                }
                ElementState::Released => {
                    mouse.just_released.insert(button);
                    mouse.pressed.remove(&button);
                }
            },
            _ => {}
        }
    }
}

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
