use cgmath::{Point3, Vector2};
use glfw::{
    fail_on_errors, Action, Context, CursorMode, Glfw, GlfwReceiver, Key, Modifiers, MouseButton,
    PWindow, Window, WindowEvent,
};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};

use crate::{
    skybox::{Skybox, SkyboxBuilder},
    Camera, DrawProperties, Gui, Model, Renderer,
};

const WINDOW_WIDTH: u32 = 1024;
const WINDOW_HEIGHT: u32 = 768;
const WINDOW_TITLE: &str = "3D Renderer in Rust by Bálint Kiss";

// This is the granularity of how often to update logic and not to be confused
// with framerate limiting or 60 frames per second, because the main loop
// implementation uses a fixed update, variable framerate timestep algorithm.
//
// 60 logic updates per second is a common value used in games.
// - Higher update rate (120) can lead to smoother gameplay, more precise
// control, at the cost of CPU load. Keep mobile devices in mind.
// - Lower update rate (30) reduces CPU load, runs game logic less frequently,
// but can make game less responsive.
const MAX_LOGIC_UPDATE_PER_SECOND: f32 = 60.0;
const FIXED_UPDATE_TIMESTEP: f32 = 1.0 / MAX_LOGIC_UPDATE_PER_SECOND;

/// Encapsulation of renderer application lifecycle and logic update to avoid
/// polluting main().
pub struct App {
    last_mouse_pos: Vector2<f32>,
    draw_props: DrawProperties,
    skybox: Skybox,
    models: Vec<Model>,
    camera: Camera,
    gui: Gui,
    renderer: Renderer,
    events: GlfwReceiver<(f64, WindowEvent)>,
    window: PWindow,
    glfw: Glfw,
}

impl App {
    pub fn new() -> Result<Self, String> {
        // Initialize windowing system
        let mut glfw = glfw::init(glfw::fail_on_errors!())
            .map_err(|e| format!("unable to initialize windowing system: {e}"))?;
        glfw.window_hint(glfw::WindowHint::ContextVersion(4, 3));
        glfw.window_hint(glfw::WindowHint::OpenGlProfile(
            glfw::OpenGlProfileHint::Core,
        ));
        // TODO: Make window and OpenGL framebuffer resizable
        glfw.window_hint(glfw::WindowHint::Resizable(false));

        // Create window
        let (mut window, events) = glfw
            .create_window(
                WINDOW_WIDTH,
                WINDOW_HEIGHT,
                WINDOW_TITLE,
                glfw::WindowMode::Windowed,
            )
            .ok_or_else(|| "unable to create window")?;
        window.make_current();
        window.set_all_polling(true);
        window.set_mouse_button_callback(&mouse_button_callback);

        let raw = window.window_handle().unwrap().as_raw();
        match raw {
            RawWindowHandle::Win32(_) => println!("Display backend is Win32"),
            RawWindowHandle::Xlib(_) => println!("Display backend is X11"),
            RawWindowHandle::Wayland(_) => println!("Display backend is Wayland"),
            _ => (),
        }

        // Init renderer
        let renderer = Renderer::new(&mut window)
            .map_err(|e| format!("unable to initialize renderer: {e}"))?;

        // Init GUI
        let gui = Gui::new(&mut window);

        // Load resources
        let skybox = SkyboxBuilder::new()
            .with_right("assets/skybox/right.jpg")
            .with_left("assets/skybox/left.jpg")
            .with_top("assets/skybox/top.jpg")
            .with_bottom("assets/skybox/bottom.jpg")
            .with_front("assets/skybox/front.jpg")
            .with_back("assets/skybox/back.jpg")
            .build()
            .map_err(|e| format!("unable to create skybox for application: {e}"))?;
        let model_paths = [
            "assets/meshes/cube.obj",
            "assets/meshes/teapot.obj",
            "assets/meshes/bunny.obj",
        ];
        let mut models: Vec<Model> = Vec::with_capacity(model_paths.len());
        for path in &model_paths {
            let model = Model::new(path)
                .map_err(|e| format!("unable to create model from path {path}: {e}"))?;
            models.push(model);
        }

        Ok(Self {
            last_mouse_pos: Vector2::new(WINDOW_WIDTH as f32 / 2.0, WINDOW_HEIGHT as f32 / 2.0),
            draw_props: DrawProperties::default(),
            skybox,
            models,
            // Positioning and rotation accidentally imitates a right-handed 3D
            // coordinate system with positive Z going farther from model, but this
            // setting is done because of initial orientation of the loaded Stanford
            // Bunny mesh.
            camera: Camera::new(Point3::new(1.7, 1.3, 4.0), Vector2::new(240.0, -15.0)),
            gui,
            renderer,
            events,
            window,
            glfw,
        })
    }

    pub fn run(&mut self) {
        // Frame-rate independent loop with fixed update, variable framerate.
        //
        // A naive calculation and passing of a deltaTime introduces floating point
        // precision errors, leading to choppy camera movement and unstable logic
        // even on high framerate. Here, think of it as renderer dictating time, and
        // logic update adapting to it.
        //
        // Prefer steady_clock over high_resolution_clock, because
        // high_resolution_clock could lie.
        let mut previous_time = std::time::Instant::now();
        // How much application "clock" is behind real time. Also known as
        // "accumulator"
        let mut lag: f32 = 0.0;
        while !self.window.should_close() {
            let current_time = std::time::Instant::now();
            let elapsed_time = (current_time - previous_time).as_secs_f32();
            previous_time = current_time;
            lag += elapsed_time;

            self.handle_input();

            while lag >= FIXED_UPDATE_TIMESTEP {
                lag -= FIXED_UPDATE_TIMESTEP;
            }

            self.renderer.draw(
                &self.window,
                &self.camera,
                &self.draw_props,
                &self.models,
                &self.skybox,
            );
            self.gui
                .draw(&mut self.window, &self.camera, &mut self.draw_props);
            self.window.swap_buffers();
        }
    }

    fn handle_input(&mut self) {
        self.glfw.poll_events();

        // Propagate events to GUI
        for (_, event) in glfw::flush_messages(&self.events) {
            match event {
                _ => self.gui.handle_event(&event),
            }
        }

        // Keyboard input
        if self.window.get_key(Key::Escape) == Action::Press {
            self.window.set_should_close(true);
        }
        if self.window.get_key(Key::W) == Action::Press {
            self.camera.move_forward(FIXED_UPDATE_TIMESTEP);
        }
        if self.window.get_key(Key::S) == Action::Press {
            self.camera.move_backward(FIXED_UPDATE_TIMESTEP);
        }
        if self.window.get_key(Key::A) == Action::Press {
            self.camera.strafe_left(FIXED_UPDATE_TIMESTEP);
        }
        if self.window.get_key(Key::D) == Action::Press {
            self.camera.strafe_right(FIXED_UPDATE_TIMESTEP);
        }
        if self.window.get_key(Key::Space) == Action::Press {
            self.camera.ascend(FIXED_UPDATE_TIMESTEP);
        }
        if self.window.get_key(Key::C) == Action::Press {
            self.camera.descend(FIXED_UPDATE_TIMESTEP);
        }

        // Mouse look
        let (current_mouse_pos_x, current_mouse_pos_y) = self.window.get_cursor_pos();
        if self.window.get_mouse_button(glfw::MouseButtonRight) == Action::Release {
            // Always save position even when not holding down mouse button to avoid
            // sudden jumps when initiating turning
            self.last_mouse_pos.x = current_mouse_pos_x as f32;
            self.last_mouse_pos.y = current_mouse_pos_y as f32;
        } else {
            let x_offset = current_mouse_pos_x as f32 - self.last_mouse_pos.x;
            // Reversed because y is bottom to up
            let y_offset = self.last_mouse_pos.y - current_mouse_pos_y as f32;
            self.last_mouse_pos.x = current_mouse_pos_x as f32;
            self.last_mouse_pos.y = current_mouse_pos_y as f32;
            self.camera.look(x_offset, y_offset);
        }
    }
}

fn mouse_button_callback(
    window: &mut Window,
    button: MouseButton,
    action: Action,
    _modifiers: Modifiers,
) {
    // Initiate mouse look on right mouse button press
    if button == glfw::MouseButtonRight {
        if action == Action::Press {
            if window.get_cursor_mode() == CursorMode::Normal {
                // Cursor disable is required to temporarily center it for
                // mouselook
                window.set_cursor_mode(CursorMode::Disabled);
            }
        // Stop mouse look on release, give cursor back. Cursor position stays
        // the same as before mouse look.
        } else {
            window.set_cursor_mode(CursorMode::Normal);
        }
    }
}
