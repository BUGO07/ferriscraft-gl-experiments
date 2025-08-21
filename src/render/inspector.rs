use egui_glium::{EguiGlium, egui_winit::egui};

use crate::{ecs::*, ui::UIRect, window::WindowEventECS};

pub fn handle_egui(
    mut egui: NonSendMut<EguiGlium>,
    mut window_events: EventReader<WindowEventECS>,
    // mut query: Query<(&mut Transform, Option<&Name>)>,
    mut camera: Single<(&mut Transform, &mut Camera3d)>,
    mut ui_query: Query<&mut UIRect>,
    debug_info: Option<Res<DebugInfo>>,
    window: NonSend<NSWindow>,
) {
    for event in window_events.read() {
        let _ = egui.on_event(&window.winit, &event.0);
    }

    egui.run(&window.winit, |ctx| {
        egui::Window::new("change transforms").show(ctx, |ui| {
            ui.add(
                egui::DragValue::new(&mut camera.1.fov)
                    .speed(0.1)
                    .range(0.1..=179.9),
            );
            // let mut iter = query.iter_mut();
            // let (mut transform, name) = iter.next().unwrap();
            egui_display_transform("camera", ui, &mut camera.0);
            for mut ui_node in ui_query.iter_mut() {
                ui.vertical_centered(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("ui node");
                        ui.add(
                            egui::DragValue::new(ui_node.x.as_f32())
                                .speed(0.1)
                                .range(f32::MIN..=f32::MAX)
                                .min_decimals(2)
                                .max_decimals(2),
                        );
                        ui.add(
                            egui::DragValue::new(ui_node.y.as_f32())
                                .speed(0.1)
                                .range(f32::MIN..=f32::MAX)
                                .min_decimals(2)
                                .max_decimals(2),
                        );
                        ui.add(
                            egui::DragValue::new(ui_node.width.as_f32())
                                .speed(0.1)
                                .range(f32::MIN..=f32::MAX)
                                .min_decimals(2)
                                .max_decimals(2),
                        );
                        ui.add(
                            egui::DragValue::new(ui_node.height.as_f32())
                                .speed(0.1)
                                .range(f32::MIN..=f32::MAX)
                                .min_decimals(2)
                                .max_decimals(2),
                        );
                    });
                });
            }
        });

        if let Some(debug_info) = &debug_info {
            egui::Window::new("debuginfo").show(ctx, |ui| {
                ui.label(format!("draw calls: {}", debug_info.draw_calls));
                ui.label(format!(
                    "vertices: {:.2}m",
                    debug_info.vertices as f32 / 1_000_000.0
                ));
                ui.label(format!(
                    "indices: {:.2}m",
                    debug_info.indices as f32 / 1_000_000.0
                ));
            });
        }
    });
}

fn egui_display_transform(label: &str, ui: &mut egui::Ui, transform: &mut Transform) {
    ui.set_max_width(200.0);
    let (yaw, pitch, roll) = transform.rotation.to_euler(EulerRot::YXZ);

    let mut rotation = vec3(yaw, pitch, roll);

    fn drag_vec3(label: &str, ui: &mut egui::Ui, vec: &mut Vec3, speed: f32, min: f32, max: f32) {
        ui.vertical_centered(|ui| {
            ui.horizontal(|ui| {
                ui.label(label);
                ui.add(
                    egui::DragValue::new(&mut vec.x)
                        .speed(speed)
                        .range(min..=max)
                        .min_decimals(2)
                        .max_decimals(2),
                );
                ui.add(
                    egui::DragValue::new(&mut vec.y)
                        .speed(speed)
                        .range(min..=max)
                        .min_decimals(2)
                        .max_decimals(2),
                );
                ui.add(
                    egui::DragValue::new(&mut vec.z)
                        .speed(speed)
                        .range(min..=max)
                        .min_decimals(2)
                        .max_decimals(2),
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
        drag_vec3("rotation     ", ui, &mut rotation, 0.01, f32::MIN, f32::MAX);
        drag_vec3(
            "scale           ",
            ui,
            &mut transform.scale,
            0.1,
            0.0,
            f32::MAX,
        );
    });

    transform.rotation = Quat::from_euler(EulerRot::YXZ, rotation.x, rotation.y, rotation.z);
}
