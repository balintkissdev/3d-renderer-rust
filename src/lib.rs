mod app;
pub use app::App;
mod camera;
pub use camera::Camera;
mod draw_properties;
pub use draw_properties::DrawProperties;
mod gui;
pub use gui::Gui;
mod model;
pub use model::Model;
mod renderer;
pub use renderer::Renderer;
mod shader;
mod skybox;
pub use skybox::{Skybox, SkyboxBuilder};

mod gl {
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}
