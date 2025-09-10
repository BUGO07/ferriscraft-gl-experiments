use glfw::{Action, Key, WindowEvent};

use crate::{
    App, GameSettings,
    ecs::*,
    utils::{take_screenshot, toggle_fullscreen},
};

#[derive(Event)]
pub struct WindowEventECS(pub WindowEvent);

pub fn window_plugin(app: &mut App) {
    app.world.init_resource::<Events<WindowEventECS>>();
    app.world.init_resource::<KeyboardInput>();
    app.world.init_resource::<MouseInput>();
    app.add_systems(PreUpdate, handle_events)
        .add_systems(Update, handle_keybinds)
        .add_systems(PostUpdate, (handle_input_cleanup, handle_window));
}

fn handle_events(
    mut events: EventReader<WindowEventECS>,
    mut keyboard: ResMut<KeyboardInput>,
    mut mouse: ResMut<MouseInput>,
) {
    for event in events.read() {
        match event.0 {
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

fn handle_keybinds(
    ns_window: NonSend<NSWindow>,
    keyboard: Res<KeyboardInput>,
    mut game_settings: ResMut<GameSettings>,
) {
    for key in keyboard.just_pressed.iter() {
        match key {
            Key::F1 => game_settings.wireframe = !game_settings.wireframe,
            Key::F2 => take_screenshot(&ns_window.window),
            Key::F11 => toggle_fullscreen(&ns_window.window),
            _ => {}
        }
    }
}

fn handle_input_cleanup(mut keyboard: ResMut<KeyboardInput>, mut mouse: ResMut<MouseInput>) {
    keyboard.just_pressed.clear();
    keyboard.just_released.clear();

    mouse.just_pressed.clear();
    mouse.just_released.clear();
    mouse.motion = Vec2::ZERO;
    mouse.scroll = Vec2::ZERO;
}

fn handle_window(mut ns_window: NonSendMut<NSWindow>, mut window: ResMut<Window>) {
    if window.cursor_grab {
        ns_window.window.set_cursor_mode(glfw::CursorMode::Disabled);
    } else if window.cursor_visible {
        ns_window.window.set_cursor_mode(glfw::CursorMode::Normal);
    } else {
        ns_window.window.set_cursor_mode(glfw::CursorMode::Hidden);
    }

    let (width, height) = ns_window.window.get_framebuffer_size();
    window.width = width;
    window.height = height;
}
