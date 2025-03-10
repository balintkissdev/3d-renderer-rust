/// Parameter object for user to customize selected model, model transformations
/// and rendering properties from UI.
///
/// Recommended to use RefCell instead of Cell, because coyping this data is costly.
pub struct DrawProperties {
    #[cfg(not(target_arch = "wasm32"))]
    pub vsync_enabled: bool,
    pub background_color: [f32; 3],
    pub model_rotation: [f32; 3],
    pub model_color: [f32; 3],
    pub light_direction: [f32; 3],
    pub field_of_view: f32,
    pub selected_model_index: usize,
    pub skybox_enabled: bool,
    pub wireframe_mode_enabled: bool,
    pub diffuse_enabled: bool,
    pub specular_enabled: bool,
}

impl Default for DrawProperties {
    fn default() -> Self {
        Self {
            #[cfg(not(target_arch = "wasm32"))]
            vsync_enabled: false,
            background_color: [0.5, 0.5, 0.5],
            model_rotation: [0.0, 0.0, 0.0],
            model_color: [0.0, 0.8, 1.0],
            light_direction: [-0.5, -1.0, 0.0],
            field_of_view: 60.0,
            selected_model_index: 2,
            skybox_enabled: true,
            wireframe_mode_enabled: false,
            diffuse_enabled: true,
            specular_enabled: true,
        }
    }
}

/// Information for displaying framerate measurements.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Default)]
pub struct FrameRateInfo {
    /// Average number of rendered frames for 1 second
    pub frames_per_second: f32,
    /// Number of milliseconds spent during rendering a single frame. More
    /// useful metric for performance measurement than simple FPS.
    pub ms_per_frame: f32,
}
