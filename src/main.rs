use std::time::{Duration, Instant};

use bevy_ecs::system::ScheduleSystem;
use bevy_tasks::{AsyncComputeTaskPool, TaskPool};
use glam::*;
use glfw::Context;

use crate::{ecs::*, window::WindowEventECS};

const CHUNK_SIZE: i32 = 32;
const SEA_LEVEL: i32 = 64;

pub mod ecs;
pub mod world;

mod particles;
mod player;
mod render;
// mod scripting;
mod ui;
mod utils;
mod window;

pub struct App {
    world: World,
    last_update: Instant,
}

#[allow(dead_code)]
impl App {
    fn init_resource<R: Resource + Default>(&mut self) -> &mut Self {
        self.world.init_resource::<R>();
        self
    }
    fn insert_resource<R: Resource>(&mut self, resource: R) -> &mut Self {
        self.world.insert_resource(resource);
        self
    }
    fn init_non_send_resource<R: Default + 'static>(&mut self) -> &mut Self {
        self.world.init_non_send_resource::<R>();
        self
    }
    fn insert_non_send_resource<R: 'static>(&mut self, resource: R) -> &mut Self {
        self.world.insert_non_send_resource(resource);
        self
    }
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
    let mut app = App {
        world: World::new(),
        last_update: Instant::now(),
    };
    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersionMajor(3));
    glfw.window_hint(glfw::WindowHint::ContextVersionMinor(3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));
    glfw.window_hint(glfw::WindowHint::Samples(Some(4)));

    #[cfg(target_os = "macos")]
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

    let (mut window, events) = glfw
        .create_window(1280, 720, "FerrisCraft GL", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window");

    window.make_current();
    window.set_framebuffer_size_polling(true);
    window.set_key_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_mouse_button_polling(true);
    window.set_scroll_polling(true);

    gl::load_with(|s| window.get_proc_address(s).unwrap() as *const _);

    glfw.set_swap_interval(glfw::SwapInterval::None);
    let (width, height) = window.get_size();
    app.world.insert_resource(Window {
        cursor_grab: false,
        cursor_visible: true,
        width,
        height,
    });

    // app.world.insert_non_send_resource(EguiGlium::new(
    //     egui::ViewportId::ROOT,
    //     &facade,
    //     &winit,
    //     &event_loop,
    // ));
    app.world.insert_non_send_resource(NSWindow {
        window,
        context: glfw,
    });

    #[cfg(debug_assertions)]
    app.world.init_resource::<DebugInfo>();

    app.world.init_resource::<Time<UpdateTime>>();
    app.world.insert_resource(Time::<FixedTime> {
        delta: Duration::from_secs_f32(1.0 / 64.0),
        ..Default::default()
    });

    // in this exact order
    app.world.add_schedule(Schedule::new(Startup));
    app.world.add_schedule(Schedule::new(PreUpdate));
    app.world.add_schedule(Schedule::new(Update));
    app.world.add_schedule(Schedule::new(FixedUpdate));
    app.world.add_schedule(Schedule::new(EguiContextPass));
    app.world.add_schedule(Schedule::new(RenderUpdate));
    app.world.add_schedule(Schedule::new(PostRenderUpdate));
    app.world.add_schedule(Schedule::new(PostUpdate));
    app.world.add_schedule(Schedule::new(Exiting));

    window::window_plugin(&mut app);
    player::player_plugin(&mut app);
    world::world_plugin(&mut app);
    ui::ui_plugin(&mut app);
    render::render_plugin(&mut app);
    particles::particle_plugin(&mut app);
    // scripting::scripting_plugin(&mut app);

    AsyncComputeTaskPool::get_or_init(TaskPool::new);

    app.world.run_schedule(Startup);

    while !app
        .world
        .non_send_resource::<NSWindow>()
        .window
        .should_close()
    {
        let now = Instant::now();
        let delta = now.duration_since(app.last_update);
        app.world
            .resource_mut::<Time<FixedTime>>()
            .extra
            .accumulator += delta;
        let mut time = app.world.resource_mut::<Time>();
        time.delta = delta;
        time.elapsed += delta.as_secs_f64();
        app.last_update = now;

        while {
            let fixed_time = app.world.resource::<Time<FixedTime>>();
            fixed_time.extra.accumulator >= fixed_time.delta
        } {
            {
                let mut fixed_time = app.world.resource_mut::<Time<FixedTime>>();
                fixed_time.elapsed += fixed_time.delta_secs_f64();
                let delta = fixed_time.delta;
                fixed_time.extra.accumulator -= delta;
            }
            app.world.run_schedule(FixedUpdate);
        }

        app.world.run_schedule(PreUpdate);
        app.world.run_schedule(Update);
        app.world.run_schedule(EguiContextPass);
        app.world.run_schedule(RenderUpdate);
        app.world.run_schedule(PostRenderUpdate);
        app.world.run_schedule(PostUpdate);

        unsafe { glfw::ffi::glfwPollEvents() };
        for (_, event) in glfw::flush_messages(&events) {
            if let glfw::WindowEvent::FramebufferSize(x, y) = event {
                unsafe { gl::Viewport(0, 0, x, y) };
            }
            app.world.send_event(WindowEventECS(event));
        }
    }

    app.world.run_schedule(Exiting);
    app.world.clear_all();
}
