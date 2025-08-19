use std::time::{Duration, Instant};

use bevy_ecs::system::ScheduleSystem;
use egui_glium::{EguiGlium, egui_winit::egui};
use glam::*;
use glium::{
    backend::glutin::SimpleWindowBuilder,
    winit::{
        application::ApplicationHandler,
        event::{DeviceEvent, MouseScrollDelta, WindowEvent},
        event_loop::{ActiveEventLoop, EventLoop},
        window::{CursorGrabMode, WindowId},
    },
};

use crate::{ecs::*, window::WindowEventECS};

#[macro_use]
extern crate glium;

const CHUNK_SIZE: i32 = 16;
const SEA_LEVEL: i32 = 32;

pub mod ecs;
pub mod world;

mod player;
mod render;
mod utils;
mod window;

pub struct Application {
    world: World,
    last_update: Instant,
    fu_accumulator: Duration,
    fixed_dt: Duration,
}

impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let (winit, facade) = SimpleWindowBuilder::new()
            .with_title("FerrisCraft GL")
            .with_inner_size(1280, 720)
            .build(event_loop);

        assert!(facade.is_glsl_version_supported(&glium::Version(glium::Api::Gl, 3, 3)));

        self.world.insert_resource(Window {
            cursor_grab: CursorGrabMode::None,
            cursor_visible: true,
            width: winit.inner_size().width,
            height: winit.inner_size().height,
        });
        self.world.insert_non_send_resource(EguiGlium::new(
            egui::ViewportId::ROOT,
            &facade,
            &winit,
            &event_loop,
        ));
        self.world
            .insert_non_send_resource(NSWindow { winit, facade });
        self.world.init_non_send_resource::<Meshes>();
        self.world.init_non_send_resource::<Materials>();

        #[cfg(debug_assertions)]
        self.world.init_resource::<DebugInfo>();

        // in this exact order
        self.world.add_schedule(Schedule::new(Startup));
        self.world.add_schedule(Schedule::new(PreUpdate));
        self.world.add_schedule(Schedule::new(Update));
        self.world.add_schedule(Schedule::new(FixedUpdate));
        self.world.add_schedule(Schedule::new(EguiContextPass));
        self.world.add_schedule(Schedule::new(RenderUpdate));
        self.world.add_schedule(Schedule::new(PostUpdate));

        window::window_plugin(self);
        player::player_plugin(self);
        world::world_plugin(self);
        render::render_plugin(self);

        self.world.run_schedule(Startup);
    }
    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.world
            .non_send_resource_mut::<NSWindow>()
            .winit
            .request_redraw()
    }
    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: glium::winit::event::DeviceId,
        event: glium::winit::event::DeviceEvent,
    ) {
        // when the cursor is locked window events dont receive the mouse motion but device events do
        match event {
            DeviceEvent::MouseMotion { delta } => {
                let window = self.world.resource::<Window>();
                if !matches!(window.cursor_grab, CursorGrabMode::None) {
                    let mut mouse = self.world.resource_mut::<MouseInput>();
                    mouse.motion.x += delta.0 as f32;
                    mouse.motion.y += delta.1 as f32;
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
                    self.world.run_schedule(FixedUpdate);
                    self.fu_accumulator -= self.fixed_dt;
                }

                self.world.run_schedule(PreUpdate);
                self.world.run_schedule(Update);
                self.world.run_schedule(EguiContextPass);
                self.world.run_schedule(RenderUpdate);
                self.world.run_schedule(PostUpdate);
            }
            _ => {}
        }
        self.world.send_event(WindowEventECS(event));
    }
    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        self.world.clear_all();
    }
}

impl Application {
    fn add_systems<M>(
        &mut self,
        schedule: impl ScheduleLabel,
        systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) -> &mut Self {
        self.world
            .resource_mut::<Schedules>()
            .add_systems(schedule, systems);
        self
    }
}

fn main() {
    let event_loop = EventLoop::new().expect("couldn't create event loop");

    let mut app = Application {
        world: World::new(),
        last_update: Instant::now(),
        fu_accumulator: Duration::ZERO,
        fixed_dt: Duration::from_secs_f32(1.0 / 64.0),
    };

    event_loop.run_app(&mut app).ok();
}
