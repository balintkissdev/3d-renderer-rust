use cgmath::{Array, Matrix, Matrix3, Matrix4, Point3, Vector3};
use std::ffi::CString;

use crate::gl;
use gl::types::*;

pub struct Shader {
    shader_program: GLuint,
    subroutine_indices: Vec<GLuint>,
}

// Wrapper around shader with helper operations
// for loading, compiling, binding, uniform value update.
impl Shader {
    pub fn new(vertex_shader_path: &str, fragment_shader_path: &str) -> Result<Self, String> {
        unsafe {
            // Compile vertex shader
            let vertex_shader = compile(vertex_shader_path, gl::VERTEX_SHADER).map_err(|e| {
                format!("failed to compile vertex shader {vertex_shader_path}: {e}")
            })?;

            // Compile fragment shader
            let fragment_shader =
                compile(fragment_shader_path, gl::FRAGMENT_SHADER).map_err(|e| {
                    format!("failed to compile fragment shader {fragment_shader_path}: {e}")
                })?;

            // Link shader program
            let shader_program = gl::CreateProgram();
            gl::AttachShader(shader_program, vertex_shader);
            gl::AttachShader(shader_program, fragment_shader);
            gl::LinkProgram(shader_program);
            // Vertex and fragment shader not needed anymore after linked as part for
            // program.
            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(fragment_shader);
            if check_linker_errors(shader_program) {
                Ok(Self {
                    shader_program,
                    subroutine_indices: Vec::new(),
                })
            } else {
                return Err("failed to link shader program".to_string());
            }
        }
    }

    // Bind shader to graphics pipeline to use for draw calls.
    pub fn r#use(&self) {
        unsafe {
            gl::UseProgram(self.shader_program);
        }
    }

    pub fn set_uniform<T: Uniform>(&self, name: &str, v: &T) {
        unsafe {
            let c_str = CString::new(name).unwrap();
            let uniform_location =
                gl::GetUniformLocation(self.shader_program, c_str.as_ptr() as *const GLchar);
            v.set_uniform(uniform_location);
        }
    }

    // Change subroutines to use in shader based on list of subroutine names.
    //
    // Subroutines are analogous to C function pointers and is an efficient way
    // to customize parts of the shader program to execute.
    //
    // Shader subroutines are only supported from OpenGL 4.0+ and are not
    // available in OpenGL ES 3.0.
    pub fn update_subroutines(&mut self, shader_type: GLenum, names: &[&str]) {
        // TODO: Clearing subroutine indices on every frame update is slow
        self.subroutine_indices.clear();

        for &name in names {
            let c_name = CString::new(name).unwrap();
            let index = unsafe {
                gl::GetSubroutineIndex(self.shader_program, shader_type, c_name.as_ptr())
            };
            self.subroutine_indices.push(index);
        }

        unsafe {
            gl::UniformSubroutinesuiv(
                shader_type,
                self.subroutine_indices.len() as GLsizei,
                self.subroutine_indices.as_ptr(),
            );
        }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            if self.shader_program != 0 {
                gl::DeleteProgram(self.shader_program);
                self.shader_program = 0; // Prevent double delete
            }
        }
    }
}

unsafe fn compile(shader_path: &str, shader_type: GLenum) -> Result<GLuint, String> {
    let src = std::fs::read_to_string(shader_path)
        .map_err(|e| format!("unable to read GLSL file: {e}"))?;
    let c_str = CString::new(src.as_bytes())
        .map_err(|e| format!("failed to convert GLSL source to C string: {e}"))?;
    let shader = gl::CreateShader(shader_type);
    gl::ShaderSource(shader, 1, &c_str.as_ptr(), std::ptr::null());
    gl::CompileShader(shader);

    if check_compile_errors(shader, shader_type) {
        Ok(shader)
    } else {
        Err("failed to compile GLSL code".to_string())
    }
}

unsafe fn check_compile_errors(shader_id: GLuint, shader_type: GLenum) -> bool {
    let mut success: GLint = 0;
    gl::GetShaderiv(shader_id, gl::COMPILE_STATUS, &mut success);

    if success == 0 {
        let mut message_length: GLint = 0;
        gl::GetShaderiv(shader_id, gl::INFO_LOG_LENGTH, &mut message_length);

        let mut message_buffer: Vec<u8> = Vec::with_capacity(message_length as usize);
        gl::GetShaderInfoLog(
            shader_id,
            message_length,
            std::ptr::null_mut(),
            message_buffer.as_mut_ptr() as *mut GLchar,
        );
        let message_type = if shader_type == gl::VERTEX_SHADER {
            "vertex"
        } else {
            "fragment"
        };
        let message = std::str::from_utf8(&message_buffer).unwrap();
        eprintln!("{message_type} shader compile error: {message}");
    }

    success != 0
}

unsafe fn check_linker_errors(shader_id: GLuint) -> bool {
    let mut success: GLint = 0;
    gl::GetProgramiv(shader_id, gl::LINK_STATUS, &mut success);

    if success == 0 {
        let mut message_length: GLint = 0;
        gl::GetProgramiv(shader_id, gl::INFO_LOG_LENGTH, &mut message_length);

        let mut message_buffer: Vec<u8> = Vec::with_capacity(message_length as usize);
        gl::GetProgramInfoLog(
            shader_id,
            message_length,
            std::ptr::null_mut(),
            message_buffer.as_mut_ptr() as *mut GLchar,
        );
        let message = std::str::from_utf8(&message_buffer).unwrap();
        eprintln!("shader link error: {message}");
    }

    success != 0
}

pub trait Uniform {
    unsafe fn set_uniform(&self, uniform_location: GLint);
}

impl Uniform for i32 {
    unsafe fn set_uniform(&self, uniform_location: GLint) {
        gl::Uniform1i(uniform_location, *self);
    }
}

impl Uniform for [f32; 3] {
    unsafe fn set_uniform(&self, uniform_location: GLint) {
        gl::Uniform3fv(uniform_location, 1, self.as_ptr());
    }
}

impl Uniform for Point3<f32> {
    unsafe fn set_uniform(&self, uniform_location: GLint) {
        gl::Uniform3fv(uniform_location, 1, self.as_ptr());
    }
}

impl Uniform for Vector3<f32> {
    unsafe fn set_uniform(&self, uniform_location: GLint) {
        gl::Uniform3fv(uniform_location, 1, self.as_ptr());
    }
}

impl Uniform for Matrix3<f32> {
    unsafe fn set_uniform(&self, uniform_location: GLint) {
        gl::UniformMatrix3fv(uniform_location, 1, gl::FALSE, self.as_ptr());
    }
}

impl Uniform for Matrix4<f32> {
    unsafe fn set_uniform(&self, uniform_location: GLint) {
        gl::UniformMatrix4fv(uniform_location, 1, gl::FALSE, self.as_ptr());
    }
}
