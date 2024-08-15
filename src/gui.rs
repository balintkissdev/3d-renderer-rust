use glfw::{Window, WindowEvent};
use imgui_glfw_rs::{
    imgui::{self, CollapsingHeader, Condition, ConfigFlags, Context, StyleColor},
    ImguiGLFW,
};

use crate::{draw_properties::DrawProperties, Camera};

// UI overlay on top of rendered scene to manipulate rendering properties.
//
// Immediate mode UI does not contain internal state, as it is the
// application's responsibility to provide that in the form of DrawProperties.
// Widgets are redrawn for each frame to integrate well into the loop of
// real-time graphics and game applications.
pub struct Gui {
    imgui: Context,
    imgui_glfw: ImguiGLFW,
}

impl Gui {
    pub fn new(window: &mut Window) -> Self {
        let mut imgui = imgui::Context::create();
        let imgui_glfw = ImguiGLFW::new(&mut imgui, window);

        // Disable ImGUI overriding GLFW cursor appearance for right-click mouselook
        imgui
            .io_mut()
            .config_flags
            .insert(ConfigFlags::NO_MOUSE_CURSOR_CHANGE);

        let transparent_background_color = [0.1, 0.1, 0.1, 0.5];
        let colors = &mut imgui.style_mut().colors;
        colors[StyleColor::WindowBg as usize] = transparent_background_color;
        colors[StyleColor::ChildBg as usize] = transparent_background_color;
        colors[StyleColor::TitleBg as usize] = transparent_background_color;

        Self { imgui, imgui_glfw }
    }

    pub fn handle_event(&mut self, event: &WindowEvent) {
        self.imgui_glfw.handle_event(&mut self.imgui, &event);
    }

    pub fn draw(&mut self, window: &mut Window, camera: &Camera, draw_props: &mut DrawProperties) {
        let ui = self.imgui_glfw.new_frame(window, &mut self.imgui);

        ui.window("Properties")
            .position([20.0, 20.0], Condition::Appearing)
            .size([280.0, 600.0], Condition::Appearing)
            .build(|| {
                if CollapsingHeader::new("Help").default_open(true).build(ui) {
                    ui.bullet_text("Movement: W, A, S, D");
                    ui.bullet_text("Mouse look: Right-click and drag");
                    ui.bullet_text("Ascend: Spacebar");
                    ui.bullet_text("Descend: C");
                }

                if CollapsingHeader::new("Camera").default_open(true).build(ui) {
                    let camera_position = camera.position();
                    ui.text(format!(
                        "X:{:.3} Y:{:.3} Z:{:.3}",
                        camera_position.x, camera_position.y, camera_position.z
                    ));
                    let camera_rotation = camera.rotation();
                    ui.text(format!(
                        "Yaw:{:.1}° Pitch:{:.1}°",
                        camera_rotation.x, camera_rotation.y
                    ));
                    ui.slider_config("##FOV", 45.0, 120.0)
                        .display_format("FOV = %.1f°")
                        .build(&mut draw_props.field_of_view);
                    ui.checkbox("Skybox", &mut draw_props.skybox_enabled);
                    if !draw_props.skybox_enabled {
                        ui.color_edit3("Background", &mut draw_props.background_color);
                    }
                }

                if CollapsingHeader::new("Model").default_open(true).build(ui) {
                    let model_items = ["Blender Cube", "Utah Teapot", "Stanford Bunny"];
                    ui.combo_simple_string(
                        "##Selected Model",
                        &mut draw_props.selected_model_index,
                        &model_items,
                    );
                    ui.checkbox("Wireframe mode", &mut draw_props.wireframe_mode_enabled);
                }

                if CollapsingHeader::new("Transform")
                    .default_open(true)
                    .build(ui)
                {
                    let min_model_rotation = 0.0;
                    let max_model_rotation = 360.0;
                    ui.slider_config("##Rotate X", min_model_rotation, max_model_rotation)
                        .display_format("X rotation = %.0f°")
                        .build(&mut draw_props.model_rotation[0]);
                    ui.slider_config("##Rotate Y", min_model_rotation, max_model_rotation)
                        .display_format("Y rotation = %.0f°")
                        .build(&mut draw_props.model_rotation[1]);
                    ui.slider_config("##Rotate Z", min_model_rotation, max_model_rotation)
                        .display_format("Z rotation = %.0f°")
                        .build(&mut draw_props.model_rotation[2]);
                }

                if CollapsingHeader::new("Material")
                    .default_open(true)
                    .build(ui)
                {
                    ui.color_edit3("##Solid Color", &mut draw_props.model_color);
                }

                if CollapsingHeader::new("Lighting")
                    .default_open(true)
                    .build(ui)
                {
                    // No Rust binding is available for ImGui::SliderFloat3()
                    ui.group(|| {
                        let slider_width = ui.content_region_avail()[0] / 3.0;
                        let min_light_direction = -1.0;
                        let max_light_direction = 1.0;
                        ui.set_next_item_width(slider_width);
                        ui.slider(
                            "##Light Direction X",
                            min_light_direction,
                            max_light_direction,
                            &mut draw_props.light_direction[0],
                        );
                        ui.same_line();
                        ui.set_next_item_width(slider_width);
                        ui.slider(
                            "##Light Direction Y",
                            min_light_direction,
                            max_light_direction,
                            &mut draw_props.light_direction[1],
                        );
                        ui.same_line();
                        ui.set_next_item_width(slider_width);
                        ui.slider(
                            "##Light Direction Z",
                            min_light_direction,
                            max_light_direction,
                            &mut draw_props.light_direction[2],
                        );
                    });
                    ui.checkbox("Diffuse", &mut draw_props.diffuse_enabled);
                    ui.checkbox("Specular", &mut draw_props.specular_enabled);
                }
            });

        self.imgui_glfw.prepare_frame(ui, window);
        self.imgui_glfw.render(&mut self.imgui);
    }
}
