use crate::gl;
use cgmath::{vec3, Vector3};
use gl::types::*;

// Representation of 3D model (currently mesh only).
//
// Mesh face vertices reside in GPU memory.
// Vertices are referred by indices to avoid storing duplicated vertices.
pub struct Model {
    pub vertex_array: GLuint,
    pub indices: Vec<GLuint>,
    vertex_buffer: GLuint,
    index_buffer: GLuint,
}

// Per-vertex data containing vertex attributes for each vertex.
//
// Texture UV coordinates are omitted because none of the bundled default
// models have textures.
#[repr(C)] // Avoid Rust compiler to reorder or use different alignments for vertex fields
struct Vertex {
    pub position: Vector3<GLfloat>,
    pub normal: Vector3<GLfloat>,
}

impl Model {
    pub fn new(path: &str) -> Result<Model, String> {
        let (vertices, indices) = load_model_from_file(path)?;

        unsafe {
            // Create vertex array
            let mut vertex_array = 0;
            gl::GenVertexArrays(1, &mut vertex_array);
            gl::BindVertexArray(vertex_array);

            // Create vertex buffer
            let mut vertex_buffer = 0;
            gl::GenBuffers(1, &mut vertex_buffer);
            gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffer);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * size_of::<Vertex>()) as GLsizeiptr,
                vertices.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );

            // Create index buffer
            let mut index_buffer = 0;
            gl::GenBuffers(1, &mut index_buffer);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, index_buffer);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (indices.len() * size_of::<GLuint>()) as GLsizeiptr,
                indices.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );

            // Setup vertex array layout
            let stride = size_of::<Vertex>() as GLsizei;
            let position_vertex_attribute = 0;
            gl::EnableVertexAttribArray(position_vertex_attribute);
            gl::VertexAttribPointer(
                position_vertex_attribute,
                3,
                gl::FLOAT,
                gl::FALSE,
                stride,
                std::mem::offset_of!(Vertex, position) as *const GLvoid,
            );

            let normal_vertex_attribute = 1;
            gl::EnableVertexAttribArray(normal_vertex_attribute);
            gl::VertexAttribPointer(
                normal_vertex_attribute,
                3,
                gl::FLOAT,
                gl::FALSE,
                stride,
                std::mem::offset_of!(Vertex, normal) as *const GLvoid,
            );

            gl::BindVertexArray(0);

            Ok(Self {
                vertex_array,
                indices,
                vertex_buffer,
                index_buffer,
            })
        }
    }
}

impl Drop for Model {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.index_buffer);
            gl::DeleteBuffers(1, &self.vertex_buffer);
            gl::DeleteVertexArrays(1, &self.vertex_array);
        }
    }
}

fn load_model_from_file(path: &str) -> Result<(Vec<Vertex>, Vec<GLuint>), String> {
    let obj = tobj::load_obj(path, &tobj::GPU_LOAD_OPTIONS)
        .map_err(|e| format!("failed to load model from {path}: {:?}", e))?;

    let models = obj.0;
    let mut vertices: Vec<Vertex> = Vec::new();
    let mut indices: Vec<GLuint> = Vec::new();
    // Sometimes you get a mesh file with just a single mesh and no others.
    // The bundled default files are such meshes.
    for model in models {
        let mesh = &model.mesh;
        let vertices_count = mesh.positions.len() / 3;
        vertices.reserve(vertices_count);
        for i in 0..vertices_count {
            vertices.push(Vertex {
                position: vec3(
                    mesh.positions[i * 3],
                    mesh.positions[i * 3 + 1],
                    mesh.positions[i * 3 + 2],
                ),
                normal: vec3(
                    mesh.normals[i * 3],
                    mesh.normals[i * 3 + 1],
                    mesh.normals[i * 3 + 2],
                ),
            });
        }

        indices.extend_from_slice(&mesh.indices);
    }

    Ok((vertices, indices))
}
