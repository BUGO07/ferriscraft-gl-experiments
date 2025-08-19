use std::time::{Duration, Instant};

use bevy_ecs::system::ScheduleSystem;
use egui_glium::{EguiGlium, egui_winit::egui};
use glam::*;
use glium::{
    Display,
    glutin::{
        config::{ConfigTemplateBuilder, GlConfig},
        context::{ContextApi, ContextAttributesBuilder},
        display::GetGlDisplay,
        prelude::{GlDisplay, NotCurrentGlContext},
        surface::{GlSurface, SwapInterval, WindowSurface},
    },
    winit::{
        application::ApplicationHandler,
        dpi::PhysicalSize,
        event::{DeviceEvent, MouseScrollDelta, WindowEvent},
        event_loop::{ActiveEventLoop, EventLoop},
        raw_window_handle::HasWindowHandle,
        window::{CursorGrabMode, WindowId},
    },
};
use glutin_winit::{DisplayBuilder, GlWindow};

use crate::{
    ecs::*,
    window::WindowEventECS,
    world::mesher::{UIVertex, VoxelVertex},
};

#[macro_use]
extern crate glium;

const CHUNK_SIZE: i32 = 16;

pub mod ecs;
pub mod world;

mod player;
mod render;
mod utils;
mod window;

pub struct App {
    world: World,
    last_update: Instant,
    fu_accumulator: Duration,
    fixed_dt: Duration,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let (winit, facade) = create_context(event_loop);

        // * here because i might revert to this if something goes wrong with the above
        // SimpleWindowBuilder::new()
        //     .with_title("FerrisCraft GL")
        //     .with_inner_size(1280, 720)
        //     .build(event_loop);

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
        self.world.init_non_send_resource::<Meshes<VoxelVertex>>();
        self.world.init_non_send_resource::<Meshes<UIVertex>>();
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

impl App {
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

    let mut app = App {
        world: World::new(),
        last_update: Instant::now(),
        fu_accumulator: Duration::ZERO,
        fixed_dt: Duration::from_secs_f32(1.0 / 64.0),
    };

    event_loop.run_app(&mut app).ok();
}

fn create_context(
    event_loop: &ActiveEventLoop,
) -> (glium::winit::window::Window, Display<WindowSurface>) {
    let (winit, gl_config) = DisplayBuilder::new()
        .with_window_attributes(Some(
            glium::winit::window::Window::default_attributes()
                .with_transparent(true)
                .with_title("FerrisCraft GL")
                .with_inner_size(PhysicalSize::new(1280, 720)),
        ))
        .build(event_loop, ConfigTemplateBuilder::new(), |configs| {
            configs
                .reduce(|accum, config| {
                    let transparency_check = config.supports_transparency().unwrap_or(false)
                        & !accum.supports_transparency().unwrap_or(false);

                    if transparency_check || config.num_samples() > accum.num_samples() {
                        config
                    } else {
                        accum
                    }
                })
                .unwrap()
        })
        .map(|(w, c)| (w.unwrap(), c))
        .unwrap();

    let attrs = winit
        .build_surface_attributes(Default::default())
        .expect("Failed to build surface attributes");
    let gl_surface = unsafe {
        gl_config
            .display()
            .create_window_surface(&gl_config, &attrs)
            .unwrap()
    };

    let gl_context = {
        let raw_window_handle = winit.window_handle().ok().map(|wh| wh.as_raw());

        let context_attributes = ContextAttributesBuilder::new().build(raw_window_handle);

        let fallback_context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::Gles(None))
            .build(raw_window_handle);

        let legacy_context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::OpenGl(Some(
                glium::glutin::context::Version::new(2, 1),
            )))
            .build(raw_window_handle);

        let gl_display = gl_config.display();

        unsafe {
            gl_display
                .create_context(&gl_config, &context_attributes)
                .unwrap_or_else(|_| {
                    gl_display
                        .create_context(&gl_config, &fallback_context_attributes)
                        .unwrap_or_else(|_| {
                            gl_display
                                .create_context(&gl_config, &legacy_context_attributes)
                                .expect("failed to create context")
                        })
                })
        }
    }
    .make_current(&gl_surface)
    .unwrap();

    gl_surface
        .set_swap_interval(&gl_context, SwapInterval::DontWait)
        .unwrap();

    let facade = Display::from_context_surface(gl_context, gl_surface).unwrap();

    (winit, facade)
}
