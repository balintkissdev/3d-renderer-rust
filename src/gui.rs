use std::sync::Arc;

use egui::Shadow;
use egui_glow::EguiGlow;
use winit::{event::WindowEvent, event_loop::ActiveEventLoop};

use crate::{Camera, DrawProperties};
#[cfg(not(target_arch = "wasm32"))]
use crate::FrameRateInfo;

/// Immediate GUI displayed as an overlay on top of rendered 3D scene. Available for both native and
/// web builds.
pub struct Gui {
    egui_glow: EguiGlow,
}

impl Gui {
    pub fn new(event_loop: &ActiveEventLoop, gl: Arc<glow::Context>) -> Self {
        let egui_glow = EguiGlow::new(&event_loop, gl.clone(), None, None, true);

        egui_glow.egui_ctx.style_mut(|style| {
            style.visuals.window_shadow = Shadow::NONE;
        });

        Self { egui_glow }
    }

    pub fn handle_events(&mut self, window: &winit::window::Window, event: &WindowEvent) {
        let _ = self.egui_glow.on_window_event(&window, &event);
    }

    pub fn prepare_frame(
        &mut self,
        window: &winit::window::Window,
        #[cfg(not(target_arch = "wasm32"))] frame_rate_info: &FrameRateInfo,
        camera: &Camera,
        draw_props: &mut DrawProperties,
    ) {
        self.egui_glow.run(&window, |egui_ctx| {
            egui::Window::new("Properties")
                .default_pos([20.0, 20.0])
                .default_size([280.0, 600.])
                .default_open(true)
                .show(egui_ctx, |ui| {
                    // Help
                    egui::CollapsingHeader::new("Help")
                        .default_open(true)
                        .show(ui, |ui| {
                            ui.label("• Movement: W, A, S, D");
                            ui.label("• Mouse look: Right-click and drag");
                            ui.label("• Ascend: Spacebar");
                            ui.label("• Descend: C");
                            #[cfg(not(target_arch = "wasm32"))]
                            {
                                ui.label("• Quit: Esc");
                            }
                        });

                    #[cfg(not(target_arch = "wasm32"))]
                    egui::CollapsingHeader::new("Renderer")
                        .default_open(true)
                        .show(ui, |ui| {
                            ui.label(format!(
                                "{:.2} FPS, {:.6} ms/frame",
                                frame_rate_info.frames_per_second, frame_rate_info.ms_per_frame
                            ));
                            ui.checkbox(&mut draw_props.vsync_enabled, "Vertical sync");
                        });

                    // Camera
                    egui::CollapsingHeader::new("Camera")
                        .default_open(true)
                        .show(ui, |ui| {
                            let camera_position = camera.position();
                            ui.label(format!(
                                "X: {:.3} Y: {:.3} Z: {:.3}",
                                camera_position.x, camera_position.y, camera_position.z
                            ));

                            let camera_rotation = camera.rotation();
                            ui.label(format!(
                                "Yaw: {:.1}° Pitch: {:.1}°",
                                camera_rotation.x, camera_rotation.y
                            ));

                            ui.add(
                                egui::Slider::new(&mut draw_props.field_of_view, 45.0..=120.0)
                                    .text("Field of view (FOV)")
                                    .suffix("°"),
                            );

                            ui.checkbox(&mut draw_props.skybox_enabled, "Skybox");
                            if !draw_props.skybox_enabled {
                                ui.horizontal(|ui| {
                                    ui.color_edit_button_rgb(&mut draw_props.background_color);
                                    ui.label("Background color");
                                });
                            }
                        });

                    // Model
                    egui::CollapsingHeader::new("Model")
                        .default_open(true)
                        .show(ui, |ui| {
                            let model_items = ["Blender Cube", "Utah Teapot", "Stanford Bunny"];
                            let selected_model_index = draw_props.selected_model_index;
                            egui::ComboBox::from_label("Select Model")
                                .selected_text(model_items[selected_model_index])
                                .show_ui(ui, |ui| {
                                    for (index, model) in model_items.iter().enumerate() {
                                        ui.selectable_value(
                                            &mut draw_props.selected_model_index,
                                            index,
                                            *model,
                                        );
                                    }
                                });

                            #[cfg(not(target_arch = "wasm32"))]
                            {
                                ui.checkbox(
                                    &mut draw_props.wireframe_mode_enabled,
                                    "Wireframe mode",
                                );
                            }
                        });

                    // Transform
                    egui::CollapsingHeader::new("Transform")
                        .default_open(true)
                        .show(ui, |ui| {
                            let model_rotation_range = 0.0..=360.0;
                            ui.add(
                                egui::Slider::new(
                                    &mut draw_props.model_rotation[0],
                                    model_rotation_range.clone(),
                                )
                                .text("X rotation")
                                .suffix("°"),
                            );
                            ui.add(
                                egui::Slider::new(
                                    &mut draw_props.model_rotation[1],
                                    model_rotation_range.clone(),
                                )
                                .text("Y rotation")
                                .suffix("°"),
                            );
                            ui.add(
                                egui::Slider::new(
                                    &mut draw_props.model_rotation[2],
                                    model_rotation_range.clone(),
                                )
                                .text("Z rotation")
                                .suffix("°"),
                            );
                        });

                    // Material
                    egui::CollapsingHeader::new("Material")
                        .default_open(true)
                        .show(ui, |ui| {
                            ui.color_edit_button_rgb(&mut draw_props.model_color);
                        });

                    // Lighting
                    egui::CollapsingHeader::new("Lighting")
                        .default_open(true)
                        .show(ui, |ui| {
                            let light_direction_range = -1.0..=1.0;
                            ui.add(
                                egui::Slider::new(
                                    &mut draw_props.light_direction[0],
                                    light_direction_range.clone(),
                                )
                                .text("Light direction X"),
                            );
                            ui.add(
                                egui::Slider::new(
                                    &mut draw_props.light_direction[1],
                                    light_direction_range.clone(),
                                )
                                .text("Light direction Y"),
                            );
                            ui.add(
                                egui::Slider::new(
                                    &mut draw_props.light_direction[2],
                                    light_direction_range.clone(),
                                )
                                .text("Light direction Z"),
                            );

                            ui.checkbox(&mut draw_props.diffuse_enabled, "Diffuse");
                            ui.checkbox(&mut draw_props.specular_enabled, "Specular");
                        });
                });
        });
    }

    pub fn draw(&mut self, window: &winit::window::Window) {
        self.egui_glow.paint(&window);
    }
}
