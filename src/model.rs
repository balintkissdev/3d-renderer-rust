use std::sync::Arc;

use cgmath::{vec3, Vector3};
use glow::{Buffer, HasContext, VertexArray};

/// Representation of 3D model (currently mesh only).
///
/// Mesh face vertices reside in GPU memory.
/// Vertices are referred by indices to avoid storing duplicated vertices.
pub struct Model {
    gl: Arc<glow::Context>,
    pub vertex_array: VertexArray,
    pub indices: Vec<u32>,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
}

/// Per-vertex data containing vertex attributes for each vertex.
///
/// Texture UV coordinates are omitted because none of the bundled default
/// models have textures.
#[repr(C)] // Avoid Rust compiler to reorder or use different alignments for vertex fields
struct Vertex {
    pub position: Vector3<f32>,
    pub normal: Vector3<f32>,
}

impl Model {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn create_from_file(gl: Arc<glow::Context>, path: &str) -> Result<Model, String> {
        let (vertices, indices) = load_obj_from_file(path)?;
        let (vertex_array, vertex_buffer, index_buffer) =
            setup_shader_plumbing(&gl, &vertices, &indices);

        Ok(Self {
            gl,
            vertex_array,
            indices,
            vertex_buffer,
            index_buffer,
        })
    }

    #[cfg(target_arch = "wasm32")]
    pub fn create_from_buffer(
        gl: Arc<glow::Context>,
        data: &'static [u8],
    ) -> Result<Model, String> {
        let (vertices, indices) =
            load_obj_from_buffer(data).map_err(|e| format!("failed to load model: {:?}", e))?;
        let (vertex_array, vertex_buffer, index_buffer) =
            setup_shader_plumbing(&gl, &vertices, &indices);

        Ok(Self {
            gl,
            vertex_array,
            indices,
            vertex_buffer,
            index_buffer,
        })
    }
}

impl Drop for Model {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_buffer(self.index_buffer);
            self.gl.delete_buffer(self.vertex_buffer);
            self.gl.delete_vertex_array(self.vertex_array);
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn load_obj_from_file(path: &str) -> Result<(Vec<Vertex>, Vec<u32>), String> {
    let obj = tobj::load_obj(path, &tobj::GPU_LOAD_OPTIONS)
        .map_err(|e| format!("failed to load model from {path}: {:?}", e))?;

    Ok(process_obj(&obj.0))
}

#[cfg(target_arch = "wasm32")]
fn load_obj_from_buffer(data: &'static [u8]) -> Result<(Vec<Vertex>, Vec<u32>), String> {
    let obj = tobj::load_obj_buf(&mut &data[..], &tobj::GPU_LOAD_OPTIONS, |_mtl_path| {
        Ok(Default::default())
    })
    .map_err(|e| format!("failed to load model: {:?}", e))?;

    Ok(process_obj(&obj.0))
}

fn process_obj(models: &Vec<tobj::Model>) -> (Vec<Vertex>, Vec<u32>) {
    let mut vertices: Vec<Vertex> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
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

    (vertices, indices)
}

fn setup_shader_plumbing(
    gl: &glow::Context,
    vertices: &Vec<Vertex>,
    indices: &Vec<u32>,
) -> (VertexArray, Buffer, Buffer) {
    unsafe {
        // Create vertex array
        let vertex_array = gl.create_vertex_array().unwrap();
        gl.bind_vertex_array(Some(vertex_array));

        // Create vertex buffer
        let vertex_buffer = gl.create_buffer().unwrap();
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vertex_buffer));
        let (_, vertices_bytes, _) = vertices.align_to::<u8>();
        gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, vertices_bytes, glow::STATIC_DRAW);

        // Create index buffer
        let index_buffer = gl.create_buffer().unwrap();
        gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(index_buffer));
        let (_, indices_bytes, _) = indices.align_to::<u8>();
        gl.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, indices_bytes, glow::STATIC_DRAW);

        // Setup vertex array layout
        let position_vertex_attribute = 0;
        let stride = size_of::<Vertex>() as i32;
        gl.enable_vertex_attrib_array(position_vertex_attribute);
        gl.vertex_attrib_pointer_f32(position_vertex_attribute, 3, glow::FLOAT, false, stride, 0);

        let normal_vertex_attribute = 1;
        gl.enable_vertex_attrib_array(normal_vertex_attribute);
        gl.vertex_attrib_pointer_f32(
            1,
            3,
            glow::FLOAT,
            false,
            stride,
            std::mem::offset_of!(Vertex, normal) as i32,
        );

        gl.bind_vertex_array(None);

        (vertex_array, vertex_buffer, index_buffer)
    }
}
