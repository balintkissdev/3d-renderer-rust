use std::sync::Arc;

use cfg_if::cfg_if;
use cgmath::{Deg, Euler, Matrix, Matrix3, Matrix4, Quaternion, SquareMatrix, Vector4, Zero};
use glow::HasContext;
use winit::window::Window;

use crate::{assets, model::Model, shader::Shader, skybox::Skybox, Camera, DrawProperties};

/// Separation of graphics API-dependent rendering mechanisms.
/// Screen update and buffer swap is responsibility of window
pub struct Renderer {
    gl: Arc<glow::Context>,
    projection: Matrix4<f32>,
    skybox_shader: Shader,
    model_shader: Shader,
}

impl Renderer {
    /// Load required shaders and set OpenGL
    /// capabilities.
    pub fn new(gl: Arc<glow::Context>) -> Result<Self, String> {
        unsafe {
            println!("Running on {}", gl.get_parameter_string(glow::RENDERER));
            println!("OpenGL version {}", gl.get_parameter_string(glow::VERSION));

            // Load shaders
            let model_shader = Shader::new(
                gl.clone(),
                &assets::shader::MODEL_VERTEX_SRC,
                &assets::shader::MODEL_FRAGMENT_SRC,
            )
            .map_err(|e| format!("model shader creation failed: {:?}", e))?;

            let skybox_shader = Shader::new(
                gl.clone(),
                &assets::shader::SKYBOX_VERTEX_SRC,
                &assets::shader::SKYBOX_FRAGMENT_SRC,
            )
            .map_err(|e| format!("skybox shader creation failed: {:?}", e))?;

            // Customize OpenGL capabilities
            gl.enable(glow::BLEND);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
            // Although in theory back-face culling would make sense from a performance point of
            // view, the display of the Utah Teapot where you can look into the inside would be
            // bugged.
            gl.disable(glow::CULL_FACE);

            Ok(Self {
                gl,
                projection: Matrix4::zero(),
                skybox_shader,
                model_shader,
            })
        }
    }

    /// Setup viewport, clear screen and draw entities
    pub fn draw(
        &mut self,
        window: &Window,
        camera: &Camera,
        draw_props: &DrawProperties,
        models: &Vec<Model>,
        skybox: &Skybox,
    ) {
        unsafe {
            // Update viewport because of Field of View change
            let framebuffer_size = window.inner_size();
            self.resize(
                framebuffer_size.width,
                framebuffer_size.height,
                draw_props.field_of_view,
            );

            // Restore depth testing (egui disables it)
            self.gl.enable(glow::DEPTH_TEST);

            // Clear screen
            self.gl.clear_color(
                draw_props.background_color[0],
                draw_props.background_color[1],
                draw_props.background_color[2],
                1.0,
            );
            self.gl
                .clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);

            // Draw entities
            self.draw_model(&camera, &draw_props, &models);
            if draw_props.skybox_enabled {
                self.draw_skybox(&camera, &skybox);
            }
        }
    }

    pub fn resize(&mut self, physical_width: u32, physical_height: u32, field_of_view: f32) {
        // Always query framebuffer size even if the window is not resizable. You'll
        // never know how framebuffer size might differ from window size, especially
        // on high-DPI displays. Not doing so can lead to display bugs like clipping
        // top part of the view.
        //
        // Physical screen size means the actual count of pixels taking DPI into account.
        unsafe {
            self.gl
                .viewport(0, 0, physical_width as i32, physical_height as i32);
            self.projection = cgmath::perspective(
                cgmath::Deg(field_of_view),
                physical_width as f32 / physical_height as f32,
                0.1,
                100.0,
            );
        }
    }

    fn draw_model(&mut self, camera: &Camera, draw_props: &DrawProperties, models: &Vec<Model>) {
        assert_eq!(models.len(), 3);
        let model = &models[draw_props.selected_model_index];

        // Set model draw shader
        self.model_shader.r#use();

        unsafe {
            // Set vertex input
            self.gl.bind_vertex_array(Some(model.vertex_array));

            // Concat matrix transformations on CPU to avoid unnecessary multiplications
            // in GLSL. Results would be the same for all vertices.
            let model_matrix = calculate_model_matrix(&draw_props.model_rotation);
            let view = camera.calculate_view_matrix();
            let mvp = self.projection * view * model_matrix;
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

            cfg_if! {
                // Native OpenGL 4 features
                if #[cfg(not(target_arch = "wasm32"))] {
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
                        glow::FRAGMENT_SHADER,
                        &[diffuse_subroutine, specular_subroutine],
                    );

                    // Display in either normal- or wireframe mode
                    self.gl.polygon_mode(
                        glow::FRONT_AND_BACK,
                        if draw_props.wireframe_mode_enabled {
                            glow::LINE
                        } else {
                            glow::FILL
                        },
                    );
                }
                // WebGL features
                else {
                   self.model_shader
                    .set_uniform("u_adsProps.diffuseEnabled", &draw_props.diffuse_enabled);
                    self.model_shader
                    .set_uniform("u_adsProps.specularEnabled", &draw_props.specular_enabled);
                }
            }

            // Issue draw call
            self.gl.draw_elements(
                glow::TRIANGLES,
                model.indices.len() as i32,
                glow::UNSIGNED_INT,
                0,
            );

            // Reset state
            #[cfg(not(target_arch = "wasm32"))]
            {
                self.gl.polygon_mode(glow::FRONT_AND_BACK, glow::FILL);
            }
            self.gl.bind_vertex_array(None);
        }
    }

    fn draw_skybox(&self, camera: &Camera, skybox: &Skybox) {
        unsafe {
            // Disable face culling for skybox
            self.gl.disable(glow::CULL_FACE);

            // Skybox needs to be drawn at the end of the rendering pipeline for
            // efficiency, not the other way around before objects (like in Painter's
            // Algorithm).
            //
            // Allow skybox pixel depths to pass depth test even when depth buffer is
            // filled with maximum 1.0 depth values. Everything drawn before skybox
            // will be displayed in front of skybox.
            // gl::DepthFunc(gl::LEQUAL);
            self.gl.depth_func(glow::LEQUAL);
            // Set skybox shader
            self.skybox_shader.r#use();
            self.gl.bind_vertex_array(Some(skybox.vertex_array));

            // Set skybox texture
            self.gl.active_texture(glow::TEXTURE0);
            self.gl
                .bind_texture(glow::TEXTURE_CUBE_MAP, Some(skybox.texture));

            let mut normalized_view = camera.calculate_view_matrix();
            // Remove camera position transformations by nullifying column 4, but keep rotation in the
            // view matrix. If you don't do this,
            // skybox will be shown as a shrinked down cube around model.
            normalized_view.w = Vector4::new(0.0, 0.0, 0.0, 0.0);
            // Concat matrix transformations on CPU to avoid unnecessary
            // multiplications in GLSL. Results would be the same for all vertices.
            let projection_view = self.projection * normalized_view;

            // Transfer uniforms
            self.skybox_shader
                .set_uniform("u_projectionView", &projection_view);
            let texture_unit = 0;
            self.skybox_shader
                .set_uniform("u_skyboxTexture", &texture_unit);

            // Issue draw call
            self.gl
                .draw_elements(glow::TRIANGLES, 36, glow::UNSIGNED_INT, 0);

            // Reset state
            self.gl.bind_vertex_array(None);
            self.gl.depth_func(glow::LESS); // Reset depth testing to default
            self.gl.enable(glow::CULL_FACE);
        }
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
