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

mod player;
mod render;
mod ui;
mod utils;
mod window;

pub struct App {
    world: World,
    last_update: Instant,
    fu_accumulator: Duration,
    fixed_dt: Duration,
}

impl App {
    fn init_resource<R: Resource + Default>(&mut self) -> &mut Self {
        self.world.init_resource::<R>();
        self
    }
    #[allow(dead_code)]
    fn insert_resource<R: Resource>(&mut self, resource: R) -> &mut Self {
        self.world.insert_resource(resource);
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
        fu_accumulator: Duration::ZERO,
        fixed_dt: Duration::from_secs_f32(1.0 / 64.0),
    };
    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersionMajor(3));
    glfw.window_hint(glfw::WindowHint::ContextVersionMinor(3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));

    #[cfg(target_os = "macos")]
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

    let (mut window, events) = glfw
        .create_window(800, 600, "FerrisCraft GL", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window");

    window.make_current();
    window.set_framebuffer_size_polling(true);
    window.set_key_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_mouse_button_polling(true);
    window.set_scroll_polling(true);

    gl::load_with(|s| window.get_proc_address(s) as *const _);
    gl::Viewport::load_with(|s| window.get_proc_address(s) as *const _);

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
    app.world.init_non_send_resource::<Meshes>();
    app.world.init_non_send_resource::<Materials>();

    #[cfg(debug_assertions)]
    app.world.init_resource::<DebugInfo>();

    // in this exact order
    app.world.add_schedule(Schedule::new(Startup));
    app.world.add_schedule(Schedule::new(PreUpdate));
    app.world.add_schedule(Schedule::new(Update));
    app.world.add_schedule(Schedule::new(FixedUpdate));
    app.world.add_schedule(Schedule::new(EguiContextPass));
    app.world.add_schedule(Schedule::new(RenderUpdate));
    app.world.add_schedule(Schedule::new(PostUpdate));

    window::window_plugin(&mut app);
    player::player_plugin(&mut app);
    world::world_plugin(&mut app);
    ui::ui_plugin(&mut app);
    render::render_plugin(&mut app);

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
        app.fu_accumulator += delta;
        let mut time = app.world.resource_mut::<Time>();
        time.delta = delta;
        time.elapsed += delta.as_secs_f32();
        app.last_update = now;

        while app.fu_accumulator >= app.fixed_dt {
            app.world.run_schedule(FixedUpdate);
            app.fu_accumulator -= app.fixed_dt;
        }

        app.world.run_schedule(PreUpdate);
        app.world.run_schedule(Update);
        app.world.run_schedule(EguiContextPass);
        app.world.run_schedule(RenderUpdate);
        app.world.run_schedule(PostUpdate);

        let mut ns_window = app.world.non_send_resource_mut::<NSWindow>();
        ns_window.context.poll_events();
        while let Some((_, event)) = events.receive() {
            app.world.send_event(WindowEventECS(event));
        }
    }
}
