use glium::winit::event::WindowEvent;

use crate::ecs::*;

#[derive(Event)]
pub struct WindowEventECS(pub WindowEvent);
