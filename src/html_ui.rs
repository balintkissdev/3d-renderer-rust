use cfg_if::cfg_if;
cfg_if! { if #[cfg(target_arch = "wasm32")] {

use std::{cell::RefCell, sync::Arc};

use egui::{Color32, Rgba};
use wasm_bindgen::prelude::*;
use web_sys::{
    Document, HtmlInputElement, HtmlSelectElement,
};

use crate::DrawProperties;

/// HTML equivalent of widgets available in overlay immediate GUI.
pub struct HtmlUI {
    skybox_checkbox: HtmlInputElement,
    background_color_picker: HtmlInputElement,
    fov_slider: HtmlInputElement,
    model_select: HtmlSelectElement,
    transform_rotation_x_slider: HtmlInputElement,
    transform_rotation_y_slider: HtmlInputElement,
    transform_rotation_z_slider: HtmlInputElement,
    material_color_picker: HtmlInputElement,
    light_direction_x_slider: HtmlInputElement,
    light_direction_y_slider: HtmlInputElement,
    light_direction_z_slider: HtmlInputElement,
    diffuse_checkbox: HtmlInputElement,
    specular_checkbox: HtmlInputElement,
}

impl HtmlUI {
    pub fn new(draw_props: Arc<RefCell<DrawProperties>>) -> Self {
        let document = web_sys::window().unwrap().document().unwrap();

        // Skybox
        let draw_props_clone = draw_props.clone();
        let skybox_checkbox = setup_checkbox(
            &document,
            "skybox-checkbox",
            draw_props.borrow().skybox_enabled,
            move |v| {
                draw_props_clone.borrow_mut().skybox_enabled = v;
            },
        );

        // Background
        let draw_props_clone = draw_props.clone();
        let background_color_picker = setup_color_picker(
            &document,
            "background-color-picker",
            draw_props.borrow().background_color,
            move |v| {
                draw_props_clone.borrow_mut().background_color = v;
            },
        );

        // Camera
        let draw_props_clone = draw_props.clone();
        let fov_slider = setup_slider(
            &document,
            "fov-slider",
            draw_props.borrow().field_of_view,
            move |v| {
                draw_props_clone.borrow_mut().field_of_view = v;
            },
        );

        // Model
        let draw_props_clone = draw_props.clone();
        let model_select = setup_select(
            &document,
            "model-select",
            draw_props.borrow().selected_model_index,
            move |v| {
                draw_props_clone.borrow_mut().selected_model_index = v;
            },
        );

        // Transform
        let draw_props_clone = draw_props.clone();
        let transform_rotation_x_slider = setup_slider(
            &document,
            "transform-rotation-x-slider",
            draw_props.borrow().model_rotation[0],
            move |v| {
                draw_props_clone.borrow_mut().model_rotation[0] = v;
            },
        );
        let draw_props_clone = draw_props.clone();
        let transform_rotation_y_slider = setup_slider(
            &document,
            "transform-rotation-y-slider",
            draw_props.borrow().model_rotation[1],
            move |v| {
                draw_props_clone.borrow_mut().model_rotation[1] = v;
            },
        );
        let draw_props_clone = draw_props.clone();
        let transform_rotation_z_slider = setup_slider(
            &document,
            "transform-rotation-z-slider",
            draw_props.borrow().model_rotation[2],
            move |v| {
                draw_props_clone.borrow_mut().model_rotation[2] = v;
            },
        );

        // Material
        let draw_props_clone = draw_props.clone();
        let material_color_picker = setup_color_picker(
            &document,
            "material-color-picker",
            draw_props.borrow().model_color,
            move |v| {
                draw_props_clone.borrow_mut().model_color = v;
            },
        );

        // Lighting
        let draw_props_clone = draw_props.clone();
        let light_direction_x_slider = setup_slider(
            &document,
            "light-direction-x-slider",
            draw_props.borrow().light_direction[0],
            move |v| {
                draw_props_clone.borrow_mut().light_direction[0] = v;
            },
        );
        let draw_props_clone = draw_props.clone();
        let light_direction_y_slider = setup_slider(
            &document,
            "light-direction-y-slider",
            draw_props.borrow().light_direction[1],
            move |v| {
                draw_props_clone.borrow_mut().light_direction[1] = v;
            },
        );
        let draw_props_clone = draw_props.clone();
        let light_direction_z_slider = setup_slider(
            &document,
            "light-direction-z-slider",
            draw_props.borrow().light_direction[2],
            move |v| {
                draw_props_clone.borrow_mut().light_direction[2] = v;
            },
        );

        let draw_props_clone = draw_props.clone();
        let diffuse_checkbox = setup_checkbox(
            &document,
            "diffuse-checkbox",
            draw_props.borrow().diffuse_enabled,
            move |v| {
                draw_props_clone.borrow_mut().diffuse_enabled = v;
            },
        );
        let draw_props_clone = draw_props.clone();
        let specular_checkbox = setup_checkbox(
            &document,
            "specular-checkbox",
            draw_props.borrow().specular_enabled,
            move |v| {
                draw_props_clone.borrow_mut().specular_enabled = v;
            },
        );

        Self {
            skybox_checkbox,
            background_color_picker,
            fov_slider,
            model_select,
            transform_rotation_x_slider,
            transform_rotation_y_slider,
            transform_rotation_z_slider,
            material_color_picker,
            light_direction_x_slider,
            light_direction_y_slider,
            light_direction_z_slider,
            diffuse_checkbox,
            specular_checkbox,
        }
    }

    pub fn sync_widgets(&mut self, draw_props: &DrawProperties) {
        self.skybox_checkbox
            .set_checked(draw_props.skybox_enabled);
        let background_color_hex =
            normalized_rgb_to_hex_color(&draw_props.background_color);
        self.background_color_picker
            .set_value(&background_color_hex.as_str());
        self.fov_slider
            .set_value(&draw_props.field_of_view.to_string().to_string());
        self.model_select
            .set_selected_index(draw_props.selected_model_index as i32);
        self.transform_rotation_x_slider.set_value(
            &draw_props.model_rotation[0]
                .to_string()
                .to_string(),
        );
        self.transform_rotation_y_slider.set_value(
            &draw_props.model_rotation[1]
                .to_string()
                .to_string(),
        );
        self.transform_rotation_z_slider.set_value(
            &draw_props.model_rotation[2]
                .to_string()
                .to_string(),
        );
        let material_color_hex = normalized_rgb_to_hex_color(&draw_props.model_color);
        self.material_color_picker
            .set_value(&material_color_hex.as_str());
        self.light_direction_x_slider.set_value(
            &draw_props.light_direction[0]
                .to_string()
                .to_string(),
        );
        self.light_direction_y_slider.set_value(
            &draw_props.light_direction[1]
                .to_string()
                .to_string(),
        );
        self.light_direction_z_slider.set_value(
            &draw_props.light_direction[2]
                .to_string()
                .to_string(),
        );
        self.diffuse_checkbox
            .set_checked(draw_props.diffuse_enabled);
        self.specular_checkbox
            .set_checked(draw_props.specular_enabled);
    }
}

fn setup_checkbox<F>(
    document: &Document,
    id: &str,
    initial_value: bool,
    oninput_fn: F,
) -> HtmlInputElement
where
    F: 'static + Fn(bool),
{
    let checkbox: HtmlInputElement = document.get_element_by_id(&id).unwrap().dyn_into().unwrap();
    checkbox.set_checked(initial_value);
    let f = Closure::<dyn FnMut(_)>::new(move |e: web_sys::Event| {
        let checkbox: HtmlInputElement = e.target().unwrap().dyn_into().unwrap();
        let v = checkbox.checked();
        oninput_fn(v);
    });
    checkbox.set_oninput(Some(f.as_ref().unchecked_ref()));
    f.forget();

    checkbox
}

fn setup_slider<F>(
    document: &Document,
    id: &str,
    initial_value: f32,
    oninput_fn: F,
) -> HtmlInputElement
where
    F: 'static + Fn(f32),
{
    let slider: HtmlInputElement = document.get_element_by_id(&id).unwrap().dyn_into().unwrap();
    slider.set_value(&initial_value.to_string());
    let f = Closure::<dyn FnMut(_)>::new(move |e: web_sys::Event| {
        let slider: HtmlInputElement = e.target().unwrap().dyn_into().unwrap();
        let v: f32 = slider.value().parse().unwrap();
        oninput_fn(v);
    });
    slider.set_oninput(Some(f.as_ref().unchecked_ref()));
    f.forget();

    slider
}

fn setup_select<F>(
    document: &Document,
    id: &str,
    initial_value: usize,
    oninput_fn: F,
) -> HtmlSelectElement
where
    F: 'static + Fn(usize),
{
    let select: HtmlSelectElement = document.get_element_by_id(&id).unwrap().dyn_into().unwrap();
    select.set_selected_index(initial_value as i32);
    let f = Closure::<dyn FnMut(_)>::new(move |e: web_sys::Event| {
        let select: HtmlSelectElement = e.target().unwrap().dyn_into().unwrap();
        let selected_index = select.selected_index() as usize;
        oninput_fn(selected_index);
    });
    select.set_oninput(Some(f.as_ref().unchecked_ref()));
    f.forget();

    select
}

fn setup_color_picker<F>(
    document: &Document,
    id: &str,
    initial_value: [f32; 3],
    oninput_fn: F,
) -> HtmlInputElement
where
    F: 'static + Fn([f32; 3]),
{
    let color_picker: HtmlInputElement =
        document.get_element_by_id(&id).unwrap().dyn_into().unwrap();
    let color_hex = normalized_rgb_to_hex_color(&initial_value);
    color_picker.set_value(&color_hex);
    let f = Closure::<dyn FnMut(_)>::new(move |e: web_sys::Event| {
        let color_picker: HtmlInputElement = e.target().unwrap().dyn_into().unwrap();
        let hex_color = color_picker.value();
        let rgb_color: [f32; 3] = hex_color_to_normalized_rgb(&hex_color);
        oninput_fn(rgb_color);
    });
    color_picker.set_oninput(Some(f.as_ref().unchecked_ref()));
    f.forget();

    color_picker
}

// Rely on egui crate's color transformation because egui does gamma correction behind the scenes.
// This fixes the bug of egui color picker and HTML color picker displaying different colors.
fn hex_color_to_normalized_rgb(hex: &String) -> [f32; 3] {
    debug_assert!(hex.starts_with('#'));
    let egui_srgb = Color32::from_hex(hex).unwrap();
    let normalized_egui_rgb =
        Rgba::from_srgba_unmultiplied(egui_srgb.r(), egui_srgb.g(), egui_srgb.b(), 255);
    [
        normalized_egui_rgb.r(),
        normalized_egui_rgb.g(),
        normalized_egui_rgb.b(),
    ]
}

fn normalized_rgb_to_hex_color(rgb: &[f32; 3]) -> String {
    let normalized_egui_rgb = Rgba::from_rgba_premultiplied(rgb[0], rgb[1], rgb[2], 1.0);
    let srgb = normalized_egui_rgb.to_srgba_unmultiplied();
    let hex = format!("#{:02x}{:02x}{:02x}", srgb[0], srgb[1], srgb[2]);
    hex
}

}} // cfg_if!
