use std::{cell::RefCell, sync::Arc};

use cfg_if::cfg_if;
use cgmath::{Point3, Vector2};
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, ElementState, KeyEvent, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{Key, KeyCode, NamedKey, PhysicalKey},
    window::{CursorGrabMode, Window, WindowAttributes},
};

use crate::{assets, Camera, DrawProperties, Gui, Model, Renderer, Skybox};

cfg_if! { if #[cfg(not(target_arch = "wasm32"))] {
    use std::{
        num::NonZeroU32,
        time::Duration,
    };

    use glutin::{
        config::{Config, ConfigTemplateBuilder},
        context::{ContextApi, ContextAttributesBuilder, PossiblyCurrentContext, Version},
        display::GetGlDisplay,
        prelude::*,
        surface::{Surface, SurfaceAttributesBuilder, SwapInterval, WindowSurface},
    };
    use glutin_winit::{DisplayBuilder, GlWindow};
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use winit::{
        dpi::{LogicalSize, PhysicalPosition},
        platform::pump_events::{EventLoopExtPumpEvents, PumpStatus}
    };

    use crate::FrameRateInfo;
    use crate::SkyboxFileBuilder;
} else {
    use wasm_bindgen::prelude::*;
    use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};
    use winit::platform::web::WindowAttributesExtWebSys;

    use crate::HtmlUI;
    use crate::SkyboxBufferBuilder;
}}

cfg_if! { if #[cfg(not(target_arch = "wasm32"))] {
    const WINDOW_WIDTH: u32 = 1024;
    const WINDOW_HEIGHT: u32 = 768;
}}
const WINDOW_TITLE: &str = "3D Renderer in Rust by Bálint Kiss";

/// This is the granularity of how often to update logic and not to be confused
/// with framerate limiting or 60 frames per second, because the main loop
/// implementation uses a fixed update, variable framerate timestep algorithm.
///
/// 60 logic updates per second is a common value used in games.
/// - Higher update rate (120) can lead to smoother gameplay, more precise
/// control, at the cost of CPU load. Keep mobile devices in mind.
/// - Lower update rate (30) reduces CPU load, runs game logic less frequently,
/// but can make game less responsive.
const MAX_LOGIC_UPDATE_PER_SECOND: f32 = 60.0;
const FIXED_UPDATE_TIMESTEP: f32 = 1.0 / MAX_LOGIC_UPDATE_PER_SECOND;

enum InputEvent {
    MoveForward,
    MoveBackward,
    StrafeLeft,
    StrafeRight,
    Ascend,
    Descend,
}

// Using array instead of HashSet results in a single jump table which is more friendlier to cache,
// avoids heap allocation and hash function calls for HashSet, has better branch prediction and has
// fewer CPU instructions.
//
// (Even though gains are negligable, because bottleneck is usually not the input handling)
type InputState = [bool; 6];

impl std::ops::Index<InputEvent> for InputState {
    type Output = bool;

    fn index(&self, e: InputEvent) -> &Self::Output {
        match e {
            InputEvent::MoveForward => &self[0],
            InputEvent::MoveBackward => &self[1],
            InputEvent::StrafeLeft => &self[2],
            InputEvent::StrafeRight => &self[3],
            InputEvent::Ascend => &self[4],
            InputEvent::Descend => &self[5],
        }
    }
}

impl std::ops::IndexMut<InputEvent> for InputState {
    fn index_mut(&mut self, e: InputEvent) -> &mut Self::Output {
        match e {
            InputEvent::MoveForward => &mut self[0],
            InputEvent::MoveBackward => &mut self[1],
            InputEvent::StrafeLeft => &mut self[2],
            InputEvent::StrafeRight => &mut self[3],
            InputEvent::Ascend => &mut self[4],
            InputEvent::Descend => &mut self[5],
        }
    }
}

/// Encapsulation of renderer application lifecycle and logic update to avoid
/// polluting main().
pub struct App {
    window: Option<Window>,
    #[cfg(not(target_arch = "wasm32"))]
    glutin_window_context: Option<GlutinWindowContext>,
    #[cfg(not(target_arch = "wasm32"))]
    vsync_enabled: bool,
    #[cfg(not(target_arch = "wasm32"))]
    frame_rate_info: FrameRateInfo,
    renderer: Option<Renderer>,
    // Pushing pressed keys from event loop into this collection and processing in update() makes
    // movement continous. Naively checking for key press during event consumption leads to choppy
    // movement.
    input_state: InputState,
    right_mouse_pressed: bool,
    draw_props: Arc<RefCell<DrawProperties>>,
    camera: Camera,
    skybox: Option<Skybox>,
    models: Vec<Model>,
    gui: Option<Gui>,
    #[cfg(target_arch = "wasm32")]
    html_ui: Option<HtmlUI>,
}

impl ApplicationHandler for App {
    // It is recommended for winit applications to create window and initialize their graphics context
    // after the first WindowEvent::Resumed even is received. There are systems that won't allow
    // applications to create a renderer until that.
    //
    // Web: WindowEvent::Resumed is emitted in response to `pageshow` event.
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        cfg_if! { if #[cfg(not(target_arch = "wasm32"))] {
            let (window, glutin_window_context, gl) = match initialize_native_window(&event_loop) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("unable to initialize native window: {:?}", e);
                    return;
                }
            };
            self.vsync_enabled = self.draw_props.borrow().vsync_enabled;
            glutin_window_context.set_vsync_enabled(self.vsync_enabled);
            let gl = Arc::new(gl);

            let skybox = match SkyboxFileBuilder::new()
                .with_right(assets::skybox::RIGHT_FACE_PATH)
                .with_left(assets::skybox::LEFT_FACE_PATH)
                .with_top(assets::skybox::TOP_FACE_PATH)
                .with_bottom(assets::skybox::BOTTOM_FACE_PATH)
                .with_front(assets::skybox::FRONT_FACE_PATH)
                .with_back(assets::skybox::BACK_FACE_PATH)
                .build(gl.clone()) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("unable to create skybox for application: {e}");
                        return;
                    }
                };

            let model_paths = [
                assets::model::CUBE_PATH,
                assets::model::TEAPOT_PATH,
                assets::model::BUNNY_PATH,
            ];
            let mut models: Vec<Model> = Vec::with_capacity(model_paths.len());
            for model_path in &model_paths {
                match Model::create_from_file(gl.clone(), model_path) {
                    Ok(m) => models.push(m),
                    Err(e) => {
                        eprintln!("unable to create model from path {model_path}: {e}");
                        return;
                    }
                }
            }
        } else {
            let (window, gl) = match initialize_web_window(&event_loop) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("unable to initialize web window: {:?}", e);
                    return;
                }
            };
            let gl = Arc::new(gl);

            let skybox = match SkyboxBufferBuilder::new()
                .with_right(assets::skybox::RIGHT_FACE_BYTES)
                .with_left(assets::skybox::LEFT_FACE_BYTES)
                .with_top(assets::skybox::TOP_FACE_BYTES)
                .with_bottom(assets::skybox::BOTTOM_FACE_BYTES)
                .with_front(assets::skybox::FRONT_FACE_BYTES)
                .with_back(assets::skybox::BACK_FACE_BYTES)
                .build(gl.clone()) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("unable to create skybox for application: {e}");
                        return;
                    }
                };

            let model_binaries: &[&'static [u8]] = &[
                assets::model::CUBE_BYTES,
                assets::model::TEAPOT_BYTES,
                assets::model::BUNNY_BYTES,
            ];
            let mut models: Vec<Model> = Vec::with_capacity(model_binaries.len());
            for model_data in model_binaries {
                match Model::create_from_buffer(gl.clone(), model_data) {
                    Ok(m) => models.push(m),
                    Err(e) => {
                        eprintln!("unable to create model: {e}");
                        return;
                    }
                }
            }
        }}

        let renderer = match Renderer::new(gl.clone()) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("unable to create renderer: {e}");
                return;
            }
        };
        let gui = Gui::new(&event_loop, gl.clone());

        self.window = Some(window);
        self.renderer = Some(renderer);
        self.skybox = Some(skybox);
        self.models = models;
        self.gui = Some(gui);

        cfg_if! { if #[cfg(not(target_arch = "wasm32"))] {
            self.glutin_window_context = Some(glutin_window_context);
        } else {
            let html_ui = HtmlUI::new(self.draw_props.clone());
            self.html_ui = Some(html_ui);
        }}
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Named(NamedKey::Escape),
                        ..
                    },
                ..
            } => event_loop.exit(),
            WindowEvent::Resized(physical_size)
                if physical_size.width != 0 && physical_size.height != 0 =>
            {
                // Even though window sizing by user is prevented, the initial window size is set
                // on application startup. OpenGL viewport setup is also setup here for the first
                // time.
                //
                // Not all platforms require the resize of glutin surface, but it's best to be safe
                // for portability.
                #[cfg(not(target_arch = "wasm32"))]
                self.glutin_window_context
                    .as_ref()
                    .unwrap()
                    .resize(physical_size.width, physical_size.height);

                let field_of_view = self.draw_props.borrow().field_of_view;
                self.renderer.as_mut().unwrap().resize(
                    physical_size.width,
                    physical_size.height,
                    field_of_view,
                );
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key),
                        repeat: false,
                        state,
                        ..
                    },
                is_synthetic: false,
                ..
            } => {
                let input_event = match key {
                    KeyCode::KeyW => InputEvent::MoveForward,
                    KeyCode::KeyS => InputEvent::MoveBackward,
                    KeyCode::KeyA => InputEvent::StrafeLeft,
                    KeyCode::KeyD => InputEvent::StrafeRight,
                    KeyCode::Space => InputEvent::Ascend,
                    KeyCode::KeyC => InputEvent::Descend,
                    _ => return,
                };
                self.input_state[input_event] = state == ElementState::Pressed;
            }
            WindowEvent::MouseInput {
                button: MouseButton::Right,
                state,
                ..
            } => {
                let window = &mut self.window.as_mut().unwrap();
                self.right_mouse_pressed = state == ElementState::Pressed;
                match state {
                    // X11 and Win32: Doesn't support CursorGrabMode::Locked
                    // Web: Doesn't support CursorGrabMode::Confined
                    ElementState::Pressed => {
                        window.set_cursor_visible(false);
                        window
                            .set_cursor_grab(CursorGrabMode::Locked)
                            .or_else(|_| window.set_cursor_grab(CursorGrabMode::Confined))
                            .unwrap();
                    }
                    ElementState::Released => {
                        // Wayland: Centering back cursor is not relevant to Wayland, because
                        // CursorGrabMode::Locked always keeps cursor at center.
                        // Web: Doesn't support changing cursor position
                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            let window_center_pos =
                                PhysicalPosition::new(WINDOW_WIDTH / 2, WINDOW_HEIGHT / 2);
                            let _ = window.set_cursor_position(window_center_pos);
                        }
                        window.set_cursor_grab(CursorGrabMode::None).unwrap();
                        window.set_cursor_visible(true);
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                // Web: corresponds to HTML canvas requestAnimationFrame() call, hence calling
                // update() here and using the custom loop on native.
                #[cfg(target_arch = "wasm32")]
                self.update();

                let draw_props = &mut self.draw_props.borrow_mut();
                if draw_props.overlay_gui_enabled {
                    self.gui.as_mut().unwrap().prepare_frame(
                        &self.window.as_mut().unwrap(),
                        #[cfg(not(target_arch = "wasm32"))]
                        &self.frame_rate_info,
                        &self.camera,
                        draw_props,
                    );
                }
                // TODO: Calling this every frame is slow.
                #[cfg(target_arch = "wasm32")]
                self.html_ui.as_mut().unwrap().sync_widgets(&draw_props);

                let skybox = &self.skybox.as_ref().unwrap();
                self.renderer.as_mut().unwrap().draw(
                    &self.window.as_ref().unwrap(),
                    &self.camera,
                    &draw_props,
                    &self.models,
                    &skybox,
                );
                if draw_props.overlay_gui_enabled {
                    self.gui
                        .as_mut()
                        .unwrap()
                        .draw(&self.window.as_mut().unwrap());
                }

                #[cfg(not(target_arch = "wasm32"))]
                self.glutin_window_context.as_ref().unwrap().swap_buffers();
            }
            _ => (),
        }

        self.gui
            .as_mut()
            .unwrap()
            .handle_events(&self.window.as_mut().unwrap(), &event);
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        match event {
            // Use the raw mouse motion for mouse look to avoid mouse position being limited
            // within the window.
            DeviceEvent::MouseMotion {
                delta: (offset_x, offset_y),
            } => {
                if self.right_mouse_pressed {
                    self.camera.look(offset_x as f32, offset_y as f32);
                }
            }
            _ => (),
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(w) = self.window.as_ref() {
            w.request_redraw();
        }
    }
}

impl App {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            window: None,
            #[cfg(not(target_arch = "wasm32"))]
            glutin_window_context: None,
            #[cfg(not(target_arch = "wasm32"))]
            vsync_enabled: false,
            #[cfg(not(target_arch = "wasm32"))]
            frame_rate_info: FrameRateInfo::default(),
            renderer: None,
            input_state: InputState::default(),
            right_mouse_pressed: false,
            // Positioning and rotation accidentally imitates a right-handed 3D
            // coordinate system with positive Z going farther from model, but this
            // setting is done because of initial orientation of the loaded Stanford
            // Bunny mesh.
            camera: Camera::new(Point3::new(1.7, 1.3, 4.0), Vector2::new(240.0, -15.0)),
            draw_props: Arc::new(RefCell::new(DrawProperties::default())),
            skybox: None,
            models: Vec::new(),
            gui: None,
            #[cfg(target_arch = "wasm32")]
            html_ui: None,
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn run(&mut self) {
        let mut event_loop = EventLoop::new().unwrap();

        let mut elapsed_frame_time: f32 = 0.0;
        let mut frame_count: i32 = 0;

        // Frame-rate independent loop with fixed update, variable framerate.
        //
        // A naive calculation and passing of a deltaTime introduces floating point
        // precision errors, leading to choppy camera movement and unstable logic
        // even on high framerate. Here, think of it as renderer dictating time, and
        // logic update adapting to it.
        let mut previous_time = std::time::Instant::now();
        // How much application "clock" is behind real time. Also known as
        // "accumulator"
        let mut lag: f32 = 0.0;
        loop {
            let current_time = std::time::Instant::now();
            let elapsed_time = (current_time - previous_time).as_secs_f32();
            previous_time = current_time;
            lag += elapsed_time;

            // Increase framerate counter
            elapsed_frame_time += elapsed_time;
            frame_count += 1;

            let timeout = Some(Duration::ZERO);
            let status = event_loop.pump_app_events(timeout, self);
            if let PumpStatus::Exit(_exit_code) = status {
                break;
            }

            while lag >= FIXED_UPDATE_TIMESTEP {
                self.update();
                lag -= FIXED_UPDATE_TIMESTEP;
            }

            let window = &self.window.as_ref().unwrap();
            window.request_redraw();

            // Measure framerate when 1 second is exceeded
            if 1.0 <= elapsed_frame_time {
                self.frame_rate_info.frames_per_second = frame_count as f32 / elapsed_frame_time;
                self.frame_rate_info.ms_per_frame = 1000.0 / frame_count as f32;

                // Reset framerate counter
                elapsed_frame_time -= 1.0;
                frame_count = 0;
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn run(&mut self) -> Result<(), String> {
        let event_loop = EventLoop::new().unwrap();
        let _ = event_loop
            .run_app(self)
            .map_err(|e| format!("error during app runtime: {:?}", e))?;
        Ok(())
    }

    fn update(&mut self) {
        // Keyboard input
        if self.input_state[InputEvent::MoveForward] {
            self.camera.move_forward(FIXED_UPDATE_TIMESTEP);
        }
        if self.input_state[InputEvent::MoveBackward] {
            self.camera.move_backward(FIXED_UPDATE_TIMESTEP);
        }
        if self.input_state[InputEvent::StrafeLeft] {
            self.camera.strafe_left(FIXED_UPDATE_TIMESTEP);
        }
        if self.input_state[InputEvent::StrafeRight] {
            self.camera.strafe_right(FIXED_UPDATE_TIMESTEP);
        }
        if self.input_state[InputEvent::Ascend] {
            self.camera.ascend(FIXED_UPDATE_TIMESTEP);
        }
        if self.input_state[InputEvent::Descend] {
            self.camera.descend(FIXED_UPDATE_TIMESTEP);
        }

        #[cfg(not(target_arch = "wasm32"))]
        if self.vsync_enabled != self.draw_props.borrow().vsync_enabled {
            self.vsync_enabled = self.draw_props.borrow().vsync_enabled;
            self.glutin_window_context
                .as_mut()
                .unwrap()
                .set_vsync_enabled(self.vsync_enabled);
        }
    }
}

/// Context Object pattern
/// (https://accu.org/journals/overload/12/63/kelly_246/) to avoid blowing up App with large number
/// of Option<> fields.
#[cfg(not(target_arch = "wasm32"))]
struct GlutinWindowContext {
    glutin_context: PossiblyCurrentContext,
    glutin_surface: Surface<WindowSurface>,
}

#[cfg(not(target_arch = "wasm32"))]
impl GlutinWindowContext {
    fn new(glutin_context: PossiblyCurrentContext, glutin_surface: Surface<WindowSurface>) -> Self {
        Self {
            glutin_context,
            glutin_surface,
        }
    }

    fn set_vsync_enabled(&self, vsync_enabled: bool) {
        let swap_interval = match vsync_enabled {
            true => SwapInterval::Wait(NonZeroU32::new(1).unwrap()),
            false => SwapInterval::DontWait,
        };

        self.glutin_surface
            .set_swap_interval(&self.glutin_context, swap_interval)
            .unwrap();
    }

    fn resize(&self, width: u32, height: u32) {
        self.glutin_surface.resize(
            &self.glutin_context,
            NonZeroU32::new(width).unwrap(),
            NonZeroU32::new(height).unwrap(),
        );
    }

    fn swap_buffers(&self) {
        let _ = self.glutin_surface.swap_buffers(&self.glutin_context);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn initialize_native_window(
    event_loop: &ActiveEventLoop,
) -> Result<(Window, GlutinWindowContext, glow::Context), String> {
    let window_attributes = WindowAttributes::default()
        .with_title(WINDOW_TITLE)
        .with_resizable(false)
        .with_inner_size(LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT));
    let display_builder =
        DisplayBuilder::new().with_window_attributes(Some(window_attributes.clone()));
    let (mut window, gl_config) = display_builder
        .build(
            event_loop,
            ConfigTemplateBuilder::default(),
            gl_config_picker,
        )
        .map_err(|e| format!("failed to create gl_config: {:?}", e))?;
    let raw_window_handle = window
        .as_ref()
        .and_then(|w| w.window_handle().ok())
        .map(|handle| handle.as_raw());
    match raw_window_handle {
        Some(RawWindowHandle::Win32(_)) => println!("Display backend is Win32"),
        Some(RawWindowHandle::Xlib(_)) => println!("Display backend is X11"),
        Some(RawWindowHandle::Wayland(_)) => println!("Display backend is Wayland"),
        _ => (),
    }

    let gl_display = gl_config.display();
    let gl_version = Version::new(4, 3);
    let context_attributes = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::OpenGl(Some(gl_version)))
        .build(raw_window_handle);

    let not_current_gl_context = unsafe {
        gl_display
            .create_context(&gl_config, &context_attributes)
            .map_err(|e| format!("failed to create a temporary context: {:?}", e))?
    };

    // Apply glutin gl_config options to winit window (removing incompatible options in the
    // process)
    let window = match window.take() {
        Some(w) => w,
        None => glutin_winit::finalize_window(event_loop, window_attributes, &gl_config)
            .map_err(|e| format!("failed to apply GL options to window: {:?}", e))?,
    };

    let surface_attributes = window
        .build_surface_attributes(SurfaceAttributesBuilder::default())
        .map_err(|e| format!("failed to build window surface attributes: {:?}", e))?;
    let glutin_surface = unsafe {
        gl_config
            .display()
            .create_window_surface(&gl_config, &surface_attributes)
            .map_err(|e| format!("failed to create window surface: {:?}", e))?
    };
    let glutin_context = not_current_gl_context
        .make_current(&glutin_surface)
        .map_err(|e| format!("failed to context make current: {:?}", e))?;

    let gl = unsafe {
        glow::Context::from_loader_function_cstr(|symbol| gl_display.get_proc_address(symbol))
    };

    Ok((
        window,
        GlutinWindowContext::new(glutin_context, glutin_surface),
        gl,
    ))
}

#[cfg(not(target_arch = "wasm32"))]
fn gl_config_picker(configs: Box<dyn Iterator<Item = Config> + '_>) -> Config {
    configs
        .reduce(|accum, config| {
            let transparency_check = config.supports_transparency().unwrap_or(false)
                & !accum.supports_transparency().unwrap_or(false);

            if transparency_check || config.num_samples() > accum.num_samples() {
                config
            } else {
                accum
            }
        })
        .unwrap()
}

#[cfg(target_arch = "wasm32")]
fn initialize_web_window(event_loop: &ActiveEventLoop) -> Result<(Window, glow::Context), String> {
    let window = web_sys::window().ok_or_else(|| "could not get browser window".to_string())?;
    let document = window
        .document()
        .ok_or_else(|| "could not get document from window".to_string())?;
    let canvas_id = "renderer-canvas";
    let canvas = document
        .get_element_by_id(&canvas_id)
        .ok_or_else(|| format!("could not find canvas element with id '{canvas_id}'"))?;
    let canvas: HtmlCanvasElement = canvas
        .dyn_into()
        .map_err(|_| format!("'{canvas_id}' is not a canvas HTML element"))?;
    let window_attributes = WindowAttributes::default()
        .with_title(WINDOW_TITLE)
        .with_canvas(Some(canvas.clone()));
    let window = event_loop
        .create_window(window_attributes)
        .map_err(|e| format!("failed to create window: {:?}", e))?;

    let webgl2_context: WebGl2RenderingContext = canvas
        .get_context("webgl2")
        .map_err(|e| format!("failed to get WebGL2 context: {:?}", e))?
        .ok_or_else(|| "'webgl2' context is not available".to_string())?
        .dyn_into()
        .map_err(|_| "canvas does not support WebGL2".to_string())?;
    let gl = glow::Context::from_webgl2_context(webgl2_context);

    Ok((window, gl))
}
