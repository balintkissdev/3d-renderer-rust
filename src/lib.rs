use cfg_if::cfg_if;

mod app;
mod assets;
pub use app::App;
mod camera;
pub use camera::Camera;
mod draw_properties;
pub use draw_properties::DrawProperties;
mod model;
pub use model::Model;
mod renderer;
pub use renderer::Renderer;
mod shader;
mod skybox;
pub use skybox::Skybox;

cfg_if! { if #[cfg(target_arch = "wasm32")] {
    use wasm_bindgen::prelude::*;

    mod html_ui;
    pub use html_ui::HtmlUI;
    pub use skybox::SkyboxBufferBuilder;

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
    pub fn start() -> Result<(), JsValue> {
        let mut app = App::new().map_err(|e| JsValue::from_str(&format!("failed to initialize app: {}", e)))?;
        app.run().map_err(|e| JsValue::from_str(&e))?;

        Ok(())
    }
} else {
    mod gui;
    pub use gui::Gui;
    pub use draw_properties::FrameRateInfo;
    pub use skybox::SkyboxFileBuilder;
}}
