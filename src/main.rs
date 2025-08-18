use std::time::{Duration, Instant};

use egui_glium::{EguiGlium, egui_winit::egui};
use glam::*;
use glium::{
    backend::glutin::SimpleWindowBuilder,
    winit::{
        application::ApplicationHandler,
        event::{DeviceEvent, ElementState, MouseScrollDelta, WindowEvent},
        event_loop::{ActiveEventLoop, EventLoop},
        keyboard::PhysicalKey,
        window::{CursorGrabMode, WindowId},
    },
};

use crate::{ecs::*, events::WindowEventECS};

#[macro_use]
extern crate glium;

const VOXEL_VERTEX_SHADER: &str = include_str!("../assets/shaders/voxel.vert");
const VOXEL_FRAGMENT_SHADER: &str = include_str!("../assets/shaders/voxel.frag");
const UI_VERTEX_SHADER: &str = include_str!("../assets/shaders/ui.vert");
const UI_FRAGMENT_SHADER: &str = include_str!("../assets/shaders/ui.frag");

const CHUNK_SIZE: i32 = 16;
const SEA_LEVEL: i32 = 32;

mod ecs;
mod events;
mod inspector;
mod mesher;
mod movement;
mod render;
mod utils;

pub struct Application {
    world: World,
    last_update: Instant,
    fu_accumulator: Duration,
    fixed_dt: Duration,
    startup_schedule: Schedule,
    update_schedule: Schedule,
    fixed_update_schedule: Schedule,
    post_update_schedule: Schedule,
    render_schedule: Schedule,
}

impl ApplicationHandler for Application {
    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        self.world.clear_all();
    }
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let (window, display) = SimpleWindowBuilder::new()
            .with_title("FerrisCraft GL")
            .with_inner_size(1280, 720)
            .build(event_loop);

        self.world.init_resource::<Events<WindowEventECS>>();
        self.world.init_resource::<KeyboardInput>();
        self.world.init_resource::<MouseInput>();
        self.world.init_resource::<Time>();
        self.world.insert_resource(Window {
            cursor_grab: CursorGrabMode::None,
            cursor_visible: true,
        });
        self.world.insert_non_send_resource(EguiGlium::new(
            egui::ViewportId::ROOT,
            &display,
            &window,
            &event_loop,
        ));
        self.world.insert_non_send_resource(NSWindow {
            winit_window: window,
            gl_context: display,
        });
        self.world.init_non_send_resource::<Meshes>();
        self.world.init_non_send_resource::<Materials>();

        #[cfg(debug_assertions)]
        self.world.init_resource::<DebugInfo>();

        self.startup_schedule
            .add_systems((render::render_setup, movement::setup));
        self.startup_schedule.run(&mut self.world);

        self.update_schedule.add_systems(movement::handle_movement);
        self.render_schedule
            .add_systems((inspector::handle_egui, render::render_update).chain());
        self.post_update_schedule
            .add_systems((events::handle_input, events::handle_window));
    }
    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.world
            .non_send_resource_mut::<NSWindow>()
            .winit_window
            .request_redraw()
    }
    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: glium::winit::event::DeviceId,
        event: glium::winit::event::DeviceEvent,
    ) {
        match event {
            DeviceEvent::MouseMotion { delta } => {
                let window = self.world.resource::<Window>();
                if !matches!(window.cursor_grab, CursorGrabMode::None) {
                    let mut mouse = self.world.resource_mut::<MouseInput>();
                    mouse.motion.x = delta.0 as f32;
                    mouse.motion.y = delta.1 as f32;
                }
            }
            DeviceEvent::MouseWheel { delta } => {
                let mut mouse = self.world.resource_mut::<MouseInput>();
                mouse.scroll = match delta {
                    MouseScrollDelta::LineDelta(x, y) => vec2(x, y),
                    MouseScrollDelta::PixelDelta(pos) => {
                        vec2(pos.x.signum() as f32, pos.y.signum() as f32)
                    }
                };
            }
            _ => {}
        }
    }
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let delta = now.duration_since(self.last_update);
                self.fu_accumulator += delta;
                self.world.resource_mut::<Time>().delta = delta;
                self.last_update = now;

                while self.fu_accumulator >= self.fixed_dt {
                    self.fixed_update_schedule.run(&mut self.world);
                    self.fu_accumulator -= self.fixed_dt;
                }

                self.update_schedule.run(&mut self.world);
                self.render_schedule.run(&mut self.world);
                self.post_update_schedule.run(&mut self.world);
            }
            WindowEvent::Resized(window_size) => {
                self.world
                    .non_send_resource_mut::<NSWindow>()
                    .gl_context
                    .resize(window_size.into());
            }
            WindowEvent::KeyboardInput {
                device_id: _,
                ref event,
                is_synthetic: _,
            } => {
                if let PhysicalKey::Code(code) = event.physical_key {
                    let mut keyboard = self.world.resource_mut::<KeyboardInput>();
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
            } => {
                let mut keyboard = self.world.resource_mut::<MouseInput>();
                match state {
                    ElementState::Pressed => {
                        keyboard.just_pressesd.insert(button);
                        keyboard.pressed.insert(button);
                    }
                    ElementState::Released => {
                        keyboard.just_released.insert(button);
                        keyboard.pressed.remove(&button);
                    }
                }
            }
            _ => {}
        }
        self.world.send_event(WindowEventECS(event));
    }
}

fn main() {
    let event_loop = EventLoop::new().expect("couldn't create event loop");

    let mut app = Application {
        world: World::new(),
        last_update: Instant::now(),
        fu_accumulator: Duration::ZERO,
        fixed_dt: Duration::from_secs_f32(1.0 / 64.0),
        startup_schedule: Schedule::new(Startup),
        update_schedule: Schedule::new(Update),
        fixed_update_schedule: Schedule::new(FixedUpdate),
        post_update_schedule: Schedule::new(PostUpdate),
        render_schedule: Schedule::new(Render),
    };

    event_loop.run_app(&mut app).ok();
}
