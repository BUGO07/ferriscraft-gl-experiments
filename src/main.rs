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

use crate::{
    ecs::*,
    render::{Meshes, Window},
};

#[macro_use]
extern crate glium;

const VERTEX_SHADER: &str = include_str!("../assets/shaders/vertex.glsl");
const FRAGMENT_SHADER: &str = include_str!("../assets/shaders/fragment.glsl");

const CHUNK_SIZE: i32 = 16;
const CHUNK_HEIGHT: i32 = 16;

mod ecs;
mod mesher;
mod render;
mod utils;

pub struct MyApp {
    world: World,
    last_update: Instant,
    fu_accumulator: Duration,
    fixed_dt: Duration,
    startup_schedule: Schedule,
    update_schedule: Schedule,
    fixed_update_schedule: Schedule,
    render_schedule: Schedule,
}

#[derive(Event)]
pub struct WindowEventECS(WindowEvent);

impl ApplicationHandler for MyApp {
    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        self.world.clear_all();
    }
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let (window, display) = SimpleWindowBuilder::new()
            .with_title("FerrisCraft GL")
            .with_inner_size(1280, 720)
            .build(event_loop);

        let egui = EguiGlium::new(egui::ViewportId::ROOT, &display, &window, &event_loop);
        self.world.init_resource::<Events<WindowEventECS>>();
        self.world.insert_non_send_resource(Window {
            winit_window: window,
            display,
        });
        self.world.insert_non_send_resource(egui);
        self.world.insert_non_send_resource(Meshes(Vec::new()));

        self.startup_schedule.add_systems(render::render_setup);
        self.startup_schedule.run(&mut self.world);

        self.render_schedule.add_systems(render::render_update);
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
                    .display
                    .resize(window_size.into());
            }
            _ => (),
        }
        self.world.send_event(WindowEventECS(event));
        // window.request_redraw();
    }
}

#[allow(deprecated)]
fn main() {
    let event_loop = EventLoop::new().expect("couldn't create event loop");

    let mut app = MyApp {
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

fn egui_display_transform(label: &str, ui: &mut egui::Ui, transform: &mut Transform) {
    ui.set_max_width(200.0);
    let (yaw, pitch, roll) = transform.rotation.to_euler(EulerRot::YXZ);

    let mut rotation = vec3(yaw.to_degrees(), pitch.to_degrees(), roll.to_degrees());

    fn drag_vec3(label: &str, ui: &mut egui::Ui, vec: &mut Vec3, speed: f32, min: f32, max: f32) {
        ui.vertical_centered(|ui| {
            ui.horizontal(|ui| {
                ui.label(label);
                ui.add(
                    egui::DragValue::new(&mut vec.x)
                        .speed(speed)
                        .range(min..=max),
                );
                ui.add(
                    egui::DragValue::new(&mut vec.y)
                        .speed(speed)
                        .range(min..=max),
                );
                ui.add(
                    egui::DragValue::new(&mut vec.z)
                        .speed(speed)
                        .range(min..=max),
                );
            });
        });
    }

    ui.vertical_centered(|ui| {
        ui.label(label);
        drag_vec3(
            "translation",
            ui,
            &mut transform.translation,
            0.1,
            f32::MIN,
            f32::MAX,
        );
        drag_vec3("rotation     ", ui, &mut rotation, 1.0, -180.0, 179.9);
        drag_vec3(
            "scale           ",
            ui,
            &mut transform.scale,
            0.1,
            0.0,
            f32::MAX,
        );
    });

    transform.rotation = Quat::from_euler(
        EulerRot::YXZ,
        rotation.x.to_radians(),
        rotation.y.to_radians(),
        rotation.z.to_radians(),
    );
}
