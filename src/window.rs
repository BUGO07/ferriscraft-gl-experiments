use glfw::{Action, WindowEvent};

use crate::{App, ecs::*};

#[derive(Event)]
pub struct WindowEventECS(pub WindowEvent);

pub fn window_plugin(app: &mut App) {
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
) {
    for event in events.read() {
        match event.0 {
            WindowEvent::FramebufferSize(x, y) => {
                unsafe { gl::Viewport(0, 0, x, y) };
            }
            WindowEvent::Key(key, _scancode, action, _modifiers) => match action {
                Action::Press => {
                    keyboard.just_pressed.insert(key);
                    keyboard.pressed.insert(key);
                }
                Action::Release => {
                    keyboard.just_released.insert(key);
                    keyboard.pressed.remove(&key);
                }
                _ => {}
            },
            WindowEvent::MouseButton(button, action, _modifiers) => match action {
                Action::Press => {
                    mouse.just_pressed.insert(button);
                    mouse.pressed.insert(button);
                }
                Action::Release => {
                    mouse.just_released.insert(button);
                    mouse.pressed.remove(&button);
                }
                _ => {}
            },
            WindowEvent::CursorPos(x, y) => {
                mouse.motion.x += x as f32 - mouse.position.x;
                mouse.motion.y += y as f32 - mouse.position.y;
                mouse.position.x = x as f32;
                mouse.position.y = y as f32;
            }
            WindowEvent::Scroll(x, y) => {
                mouse.scroll.x += x as f32;
                mouse.scroll.y += y as f32;
            }
            _ => {}
        }
    }
}

pub fn handle_input_cleanup(mut keyboard: ResMut<KeyboardInput>, mut mouse: ResMut<MouseInput>) {
    keyboard.just_pressed.clear();
    keyboard.just_released.clear();

    mouse.just_pressed.clear();
    mouse.just_released.clear();
    mouse.motion = Vec2::ZERO;
    mouse.scroll = Vec2::ZERO;
}

pub fn handle_window(
    mut ns_window: NonSendMut<NSWindow>,
    mut window: ResMut<Window>,
    mut not_first_run: Local<bool>,
) {
    if !*not_first_run {
        // don't do anything on the first frame
        // changing cursor visibility doesn't work on the first frame
        *not_first_run = true;
        return;
    }
    // idk
    if window.cursor_grab {
        ns_window.window.set_cursor_mode(glfw::CursorMode::Disabled);
    } else if window.cursor_visible {
        ns_window.window.set_cursor_mode(glfw::CursorMode::Normal);
    } else {
        ns_window.window.set_cursor_mode(glfw::CursorMode::Hidden);
    }

    let (width, height) = ns_window.window.get_size();
    window.width = width;
    window.height = height;
}
