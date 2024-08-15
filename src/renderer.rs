use std::ffi::CStr;

use cgmath::{Deg, Euler, Matrix, Matrix3, Matrix4, Quaternion, SquareMatrix, Vector4};
use glfw::Window;

use crate::model::Model;
use crate::shader::Shader;
use crate::skybox::Skybox;
use crate::{gl, Camera, DrawProperties};
use gl::types::*;

// Separation of graphics API-dependent rendering mechanisms.
// Screen update and buffer swap is responsibility of window
pub struct Renderer {
    skybox_shader: Shader,
    model_shader: Shader,
}

impl Renderer {
    // Load OpenGL function addresses, required shaders and set OpenGL
    // capabilities.
    pub fn new(window: &mut Window) -> Result<Self, String> {
        unsafe {
            // Set OpenGL function addresses
            gl::load_with(|symbol| window.get_proc_address(symbol).cast());

            if let Some(renderer) = get_gl_string(gl::RENDERER) {
                println!("Running on {}", renderer.to_string_lossy());
            }
            if let Some(version) = get_gl_string(gl::VERSION) {
                println!("OpenGL version {}", version.to_string_lossy());
            }

            // Load shaders
            let model_shader = Shader::new(
                "assets/shaders/model_gl4.vert.glsl",
                "assets/shaders/model_gl4.frag.glsl",
            )
            .map_err(|e| format!("model shader creation failed: {:?}", e))?;

            let skybox_shader = Shader::new(
                "assets/shaders/skybox_gl4.vert.glsl",
                "assets/shaders/skybox_gl4.frag.glsl",
            )
            .map_err(|e| format!("skybox shader creation failed: {:?}", e))?;

            // Customize OpenGL capabilities
            gl::Enable(gl::DEPTH_TEST);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

            Ok(Self {
                skybox_shader,
                model_shader,
            })
        }
    }

    // Setup viewport, clear screen and draw entities
    pub fn draw(
        &mut self,
        window: &Window,
        camera: &Camera,
        draw_props: &DrawProperties,
        models: &Vec<Model>,
        skybox: &Skybox,
    ) {
        unsafe {
            // Viewport setup
            //
            // Always query framebuffer size even if the window is not resizable. You'll
            // never know how framebuffer size might differ from window size, especially
            // on high-DPI displays. Not doing so can lead to display bugs like clipping
            // top part of the view.
            let (framebuffer_width, framebuffer_height) = window.get_framebuffer_size();
            gl::Viewport(0, 0, framebuffer_width, framebuffer_height);
            let projection = cgmath::perspective(
                cgmath::Deg(draw_props.field_of_view),
                framebuffer_width as f32 / framebuffer_height as f32,
                0.1,
                100.0,
            );

            // Clear screen
            gl::ClearColor(
                draw_props.background_color[0],
                draw_props.background_color[1],
                draw_props.background_color[2],
                1.0,
            );
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            // Draw entities
            self.draw_model(&projection, &camera, &draw_props, &models);
            if draw_props.skybox_enabled {
                self.draw_skybox(&projection, &camera, &skybox);
            }
        }
    }

    fn draw_model(
        &mut self,
        projection: &Matrix4<f32>,
        camera: &Camera,
        draw_props: &DrawProperties,
        models: &Vec<Model>,
    ) {
        assert_eq!(models.len(), 3);
        let model = &models[draw_props.selected_model_index];

        // Set model draw shader
        self.model_shader.r#use();

        unsafe {
            // Set vertex input
            gl::BindVertexArray(model.vertex_array);

            // Concat matrix transformations on CPU to avoid unnecessary multiplications
            // in GLSL. Results would be the same for all vertices.
            let model_matrix = calculate_model_matrix(&draw_props.model_rotation);
            let view = camera.calculate_view_matrix();
            let mvp = projection * view * model_matrix;
            let normal_matrix = calculate_normal_matrix(&model_matrix);

            // Transfer uniforms
            self.model_shader.set_uniform("u_model", &model_matrix);
            self.model_shader.set_uniform("u_mvp", &mvp);
            self.model_shader
                .set_uniform("u_normalMatrix", &normal_matrix);
            self.model_shader
                .set_uniform("u_color", &draw_props.model_color);
            self.model_shader
                .set_uniform("u_light.direction", &draw_props.light_direction);
            self.model_shader
                .set_uniform("u_viewPos", camera.position());

            // Set OpenGL 4.x subroutines
            let diffuse_subroutine = if draw_props.diffuse_enabled {
                "DiffuseEnabled"
            } else {
                "Disabled"
            };
            let specular_subroutine = if draw_props.specular_enabled {
                "SpecularEnabled"
            } else {
                "Disabled"
            };
            self.model_shader.update_subroutines(
                gl::FRAGMENT_SHADER,
                &[diffuse_subroutine, specular_subroutine],
            );

            // Display in either normal- or wireframe mode
            gl::PolygonMode(
                gl::FRONT_AND_BACK,
                if draw_props.wireframe_mode_enabled {
                    gl::LINE
                } else {
                    gl::FILL
                },
            );

            // Issue draw call
            gl::DrawElements(
                gl::TRIANGLES,
                model.indices.len() as GLsizei,
                gl::UNSIGNED_INT,
                std::ptr::null(),
            );

            // Reset state
            gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
            gl::BindVertexArray(0);
        }
    }

    fn draw_skybox(&self, projection: &Matrix4<f32>, camera: &Camera, skybox: &Skybox) {
        unsafe {
            // Skybox needs to be drawn at the end of the rendering pipeline for
            // efficiency, not the other way around before objects (like in Painter's
            // Algorithm).
            //
            // Allow skybox pixel depths to pass depth test even when depth buffer is
            // filled with maximum 1.0 depth values. Everything drawn before skybox
            // will be displayed in front of skybox.
            gl::DepthFunc(gl::LEQUAL);
            // Set skybox shader
            self.skybox_shader.r#use();
            gl::BindVertexArray(skybox.vertex_array);

            // Set skybox texture
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_CUBE_MAP, skybox.texture_id);

            let mut normalized_view = camera.calculate_view_matrix();
            // Remove camera position transformations by nullifying column 4, but keep rotation in the
            // view matrix. If you don't do this,
            // skybox will be shown as a shrinked down cube around model.
            normalized_view.w = Vector4::new(0.0, 0.0, 0.0, 0.0);
            // Concat matrix transformations on CPU to avoid unnecessary
            // multiplications in GLSL. Results would be the same for all vertices.
            let projection_view = projection * normalized_view;

            // Transfer uniforms
            self.skybox_shader
                .set_uniform("u_projectionView", &projection_view);
            let texture_unit = 0;
            self.skybox_shader
                .set_uniform("u_skyboxTexture", &texture_unit);

            // Issue draw call
            gl::DrawElements(gl::TRIANGLES, 36, gl::UNSIGNED_INT, std::ptr::null());

            // Reset state
            gl::BindVertexArray(0);
            gl::DepthFunc(gl::LESS); // Reset depth testing to default
        }
    }
}

fn get_gl_string(name: GLenum) -> Option<&'static CStr> {
    unsafe {
        let s = gl::GetString(name);
        (!s.is_null()).then(|| CStr::from_ptr(s.cast()))
    }
}

fn calculate_model_matrix(rotation: &[f32; 3]) -> Matrix4<f32> {
    // Avoid Gimbal-lock by converting Euler angles to quaternions
    let q = Quaternion::from(Euler {
        x: Deg(rotation[0]),
        y: Deg(rotation[1]),
        z: Deg(rotation[2]),
    });

    Matrix4::from(q)
}

fn calculate_normal_matrix(m: &Matrix4<f32>) -> Matrix3<f32> {
    let inverse_transpose = m.invert().unwrap().transpose();
    Matrix3::new(
        inverse_transpose.x.x,
        inverse_transpose.x.y,
        inverse_transpose.x.z,
        inverse_transpose.y.x,
        inverse_transpose.y.y,
        inverse_transpose.y.z,
        inverse_transpose.z.x,
        inverse_transpose.z.y,
        inverse_transpose.z.z,
    )
}
