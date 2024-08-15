use image::EncodableLayout;

use crate::gl::{self, types::*};

// Skybox containing cube-mapped texture and vertex positions for skybox
// cube.
//
// Cube-map is represented by six subtextures that must be square and the same
// size. Sampling from cube-map is done as direction from origin. Skybox is an
// application of cube-mapping where entire scene is wrapped in a large cube
// surrounding the viewer and model. A unit cube is rendered centered
// at the origin and uses the object space position as a texture coordinate
// from which to sample the cube map texture.
//
// Texture and vertex data are stored in GPU memory.
pub struct Skybox {
    pub texture_id: GLuint,
    pub vertex_array: GLuint,
    vertex_buffer: GLuint,
    index_buffer: GLuint,
}

impl Drop for Skybox {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.index_buffer);
            gl::DeleteBuffers(1, &self.vertex_buffer);
            gl::DeleteVertexArrays(1, &self.vertex_array);
            gl::DeleteTextures(1, &self.texture_id);
        }
    }
}

// Builder pattern for skybox creation, avoiding mistakes from specifying
// skybox face texture parameters out of order.
#[derive(Default)]
pub struct SkyboxBuilder {
    right_face_path: String,
    left_face_path: String,
    top_face_path: String,
    bottom_face_path: String,
    front_face_path: String,
    back_face_path: String,
}

impl SkyboxBuilder {
    pub fn new() -> SkyboxBuilder {
        SkyboxBuilder::default()
    }

    pub fn with_right(mut self, right_face_path: &str) -> SkyboxBuilder {
        self.right_face_path = right_face_path.to_string();
        self
    }

    pub fn with_left(mut self, left_face_path: &str) -> SkyboxBuilder {
        self.left_face_path = left_face_path.to_string();
        self
    }

    pub fn with_top(mut self, top_face_path: &str) -> SkyboxBuilder {
        self.top_face_path = top_face_path.to_string();
        self
    }

    pub fn with_bottom(mut self, bottom_face_path: &str) -> SkyboxBuilder {
        self.bottom_face_path = bottom_face_path.to_string();
        self
    }

    pub fn with_front(mut self, front_face_path: &str) -> SkyboxBuilder {
        self.front_face_path = front_face_path.to_string();
        self
    }

    pub fn with_back(mut self, back_face_path: &str) -> SkyboxBuilder {
        self.back_face_path = back_face_path.to_string();
        self
    }

    // Load texture faces and generate vertex and index buffers.
    pub fn build(self) -> Result<Skybox, String> {
        unsafe {
            // Create texture
            let mut texture_id: GLuint = 0;
            gl::GenTextures(1, &mut texture_id);
            gl::BindTexture(gl::TEXTURE_CUBE_MAP, texture_id);

            // Load texture images
            let texture_face_paths: [&str; 6] = [
                &self.right_face_path,
                &self.left_face_path,
                &self.top_face_path,
                &self.bottom_face_path,
                &self.front_face_path,
                &self.back_face_path,
            ];
            for (i, face_path) in texture_face_paths.iter().enumerate() {
                let img = image::open(face_path).map_err(|e| {
                    format!("unable to load skybox texture from {face_path}: {:?}", e)
                })?;
                gl::TexImage2D(
                    gl::TEXTURE_CUBE_MAP_POSITIVE_X + i as GLenum,
                    0,
                    gl::RGB as GLint,
                    img.width() as GLsizei,
                    img.height() as GLsizei,
                    0,
                    gl::RGB,
                    gl::UNSIGNED_BYTE,
                    img.to_rgb8().as_bytes().as_ptr() as *const _,
                );
            }

            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_MIN_FILTER,
                gl::LINEAR as GLint,
            );
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_MAG_FILTER,
                gl::LINEAR as GLint,
            );
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_WRAP_S,
                gl::CLAMP_TO_EDGE as GLint,
            );
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_WRAP_T,
                gl::CLAMP_TO_EDGE as GLint,
            );
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_WRAP_R,
                gl::CLAMP_TO_EDGE as GLint,
            );

            // Create buffers

            #[rustfmt::skip]
            let skybox_vertices: [GLfloat; 24] = [
                -1.0,  1.0, -1.0,
                -1.0, -1.0, -1.0,
                 1.0, -1.0, -1.0,
                 1.0,  1.0, -1.0,
                -1.0,  1.0,  1.0,
                -1.0, -1.0,  1.0,
                 1.0, -1.0,  1.0,
                 1.0,  1.0,  1.0,
            ];

            #[rustfmt::skip]
            let skybox_indices: [GLuint; 36] = [
                // Front face
                0, 1, 2,
                2, 3, 0,
                // Back face
                4, 5, 6,
                6, 7, 4,
                // Left face
                4, 5, 1,
                1, 0, 4,
                // Right face
                3, 2, 6,
                6, 7, 3,
                // Top face
                4, 0, 3,
                3, 7, 4,
                // Bottom face
                1, 5, 6,
                6, 2, 1,
            ];

            // Create vertex array
            let mut vertex_array: GLuint = 0;
            gl::GenVertexArrays(1, &mut vertex_array);
            gl::BindVertexArray(vertex_array);

            // Create vertex buffer
            let mut vertex_buffer: GLuint = 0;
            gl::GenBuffers(1, &mut vertex_buffer);
            gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffer);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (skybox_vertices.len() * size_of::<GLfloat>()) as GLsizeiptr,
                skybox_vertices.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );

            // Create index buffer
            let mut index_buffer: GLuint = 0;
            gl::GenBuffers(1, &mut index_buffer);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, index_buffer);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (skybox_indices.len() * size_of::<GLuint>()) as GLsizeiptr,
                skybox_indices.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );

            // Setup vertex array layout (just vertex positions)
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                3 * size_of::<GLfloat>() as GLsizei,
                std::ptr::null(),
            );

            Ok(Skybox {
                texture_id,
                vertex_array,
                vertex_buffer,
                index_buffer,
            })
        }
    }
}
