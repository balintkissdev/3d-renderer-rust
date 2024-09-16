use std::sync::Arc;

use cfg_if::cfg_if;
use cgmath::{Matrix, Matrix3, Matrix4, Point3, Vector3};
use glow::*;

/// Wrapper around shader with helper operations
/// for loading, compiling, binding and uniform value update.
pub struct Shader {
    gl: Arc<glow::Context>,
    shader_program: glow::Program,

    #[cfg(not(target_arch = "wasm32"))]
    subroutine_indices: Vec<u32>,
}

impl Shader {
    pub fn new(
        gl: Arc<glow::Context>,
        vertex_shader_src: &str,
        fragment_shader_src: &str,
    ) -> Result<Self, String> {
        unsafe {
            let vertex_shader = compile(&gl, vertex_shader_src, glow::VERTEX_SHADER)
                .map_err(|e| format!("failed to compile vertex shader: {e}"))?;
            let fragment_shader = compile(&gl, fragment_shader_src, glow::FRAGMENT_SHADER)
                .map_err(|e| format!("failed to compile fragment shader: {e}"))?;

            let shader_program = gl
                .create_program()
                .map_err(|e| format!("cannot create shader program: {e}"))?;
            gl.attach_shader(shader_program, vertex_shader);
            gl.attach_shader(shader_program, fragment_shader);
            gl.link_program(shader_program);
            if !gl.get_program_link_status(shader_program) {
                return Err(format!(
                    "failed to link shader program: {}",
                    gl.get_program_info_log(shader_program)
                ));
            }

            cfg_if! { if #[cfg(not(target_arch = "wasm32"))] {
                Ok(Self {
                    gl,
                    shader_program,
                    subroutine_indices: Vec::new(),
                })
            } else {
                Ok(Self {
                    gl,
                    shader_program,
                })

            }}
        }
    }

    /// Bind shader to graphics pipeline to use for draw calls.
    pub fn r#use(&self) {
        unsafe {
            self.gl.use_program(Some(self.shader_program));
        }
    }

    pub fn set_uniform<T: Uniform>(&self, name: &str, v: &T) {
        unsafe {
            let uniform_location = self.gl.get_uniform_location(self.shader_program, name);
            v.set_uniform(&self.gl, uniform_location.unwrap());
        }
    }

    /// Change subroutines to use in shader based on list of subroutine names.
    ///
    /// Subroutines are analogous to C function pointers and is an efficient way
    /// to customize parts of the shader program to execute.
    ///
    /// Shader subroutines are only supported from OpenGL 4.0+ and are not
    /// available in OpenGL ES 3.0.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn update_subroutines(&mut self, shader_type: u32, names: &[&str]) {
        // TODO: Clearing subroutine indices on every frame update is slow
        self.subroutine_indices.clear();

        for &name in names {
            let index = unsafe {
                self.gl
                    .get_subroutine_index(self.shader_program, shader_type, name)
            };
            self.subroutine_indices.push(index);
        }

        unsafe {
            self.gl
                .uniform_subroutines_u32_slice(shader_type, &self.subroutine_indices);
        }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_program(self.shader_program);
        }
    }
}

unsafe fn compile(
    gl: &glow::Context,
    shader_src: &str,
    shader_type: u32,
) -> Result<glow::Shader, String> {
    let shader = gl
        .create_shader(shader_type)
        .map_err(|e| format!("cannot create shader: {e}"))?;
    gl.shader_source(shader, &shader_src);
    gl.compile_shader(shader);
    if !gl.get_shader_compile_status(shader) {
        return Err(format!(
            "failed to compile GLSL code: {}",
            gl.get_shader_info_log(shader)
        ));
    }

    Ok(shader)
}

pub trait Uniform {
    unsafe fn set_uniform(&self, gl: &glow::Context, uniform_location: UniformLocation);
}

impl Uniform for bool {
    unsafe fn set_uniform(&self, gl: &glow::Context, uniform_location: UniformLocation) {
        gl.uniform_1_i32(Some(&uniform_location), *self as i32);
    }
}

impl Uniform for i32 {
    unsafe fn set_uniform(&self, gl: &glow::Context, uniform_location: UniformLocation) {
        gl.uniform_1_i32(Some(&uniform_location), *self);
    }
}

impl Uniform for [f32; 3] {
    unsafe fn set_uniform(&self, gl: &glow::Context, uniform_location: UniformLocation) {
        gl.uniform_3_f32(Some(&uniform_location), self[0], self[1], self[2]);
    }
}

impl Uniform for Point3<f32> {
    unsafe fn set_uniform(&self, gl: &glow::Context, uniform_location: UniformLocation) {
        gl.uniform_3_f32(Some(&uniform_location), self.x, self.y, self.z);
    }
}

impl Uniform for Vector3<f32> {
    unsafe fn set_uniform(&self, gl: &glow::Context, uniform_location: UniformLocation) {
        gl.uniform_3_f32(Some(&uniform_location), self.x, self.y, self.z);
    }
}

impl Uniform for Matrix3<f32> {
    unsafe fn set_uniform(&self, gl: &glow::Context, uniform_location: UniformLocation) {
        let slice = std::slice::from_raw_parts(self.as_ptr(), 9);
        gl.uniform_matrix_3_f32_slice(Some(&uniform_location), false, slice);
    }
}

impl Uniform for Matrix4<f32> {
    unsafe fn set_uniform(&self, gl: &glow::Context, uniform_location: UniformLocation) {
        let slice = std::slice::from_raw_parts(self.as_ptr(), 16);
        gl.uniform_matrix_4_f32_slice(Some(&uniform_location), false, slice);
    }
}
