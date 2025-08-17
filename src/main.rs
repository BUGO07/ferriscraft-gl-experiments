use std::time::{Duration, Instant};

use egui_glium::{EguiGlium, egui_winit::egui};
use glam::*;
use glium::{
    backend::glutin::SimpleWindowBuilder,
    winit::{
        application::ApplicationHandler,
        event::WindowEvent,
        event_loop::{ActiveEventLoop, EventLoop},
        window::WindowId,
    },
};

use crate::{ecs::*, events::WindowEventECS};

#[macro_use]
extern crate glium;

const VERTEX_SHADER: &str = include_str!("../assets/shaders/vertex.glsl");
const FRAGMENT_SHADER: &str = include_str!("../assets/shaders/fragment.glsl");

const CHUNK_SIZE: i32 = 16;
const SEA_LEVEL: i32 = 32;

mod ecs;
mod events;
mod inspector;
mod mesher;
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
        self.world.insert_non_send_resource(EguiGlium::new(
            egui::ViewportId::ROOT,
            &display,
            &window,
            &event_loop,
        ));
        self.world.insert_non_send_resource(Window {
            winit_window: window,
            gl_context: display,
        });
        self.world.init_non_send_resource::<Meshes>();
        self.world.init_non_send_resource::<Materials>();

        #[cfg(debug_assertions)]
        self.world.init_resource::<DebugInfo>();

        self.startup_schedule.add_systems(render::render_setup);
        self.startup_schedule.run(&mut self.world);

        self.render_schedule
            .add_systems((inspector::handle_egui, render::render_update).chain());
    }
    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.world
            .non_send_resource_mut::<Window>()
            .winit_window
            .request_redraw()
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
                self.fu_accumulator += now.duration_since(self.last_update);
                self.last_update = now;

                while self.fu_accumulator >= self.fixed_dt {
                    self.fixed_update_schedule.run(&mut self.world);
                    self.fu_accumulator -= self.fixed_dt;
                }

                self.update_schedule.run(&mut self.world);
                self.render_schedule.run(&mut self.world);
            }
            WindowEvent::Resized(window_size) => {
                self.world
                    .non_send_resource_mut::<Window>()
                    .gl_context
                    .resize(window_size.into());
            }
            _ => (),
        }
        self.world.send_event(WindowEventECS(event));
        // window.request_redraw();
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
        render_schedule: Schedule::new(Render),
    };

    event_loop.run_app(&mut app).ok();
}
