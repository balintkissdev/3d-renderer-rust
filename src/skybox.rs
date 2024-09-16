use std::sync::Arc;

use cfg_if::cfg_if;
use glow::{Buffer, HasContext, Texture, VertexArray};
use image::{DynamicImage, EncodableLayout};

/// Skybox containing cube-mapped texture and vertex positions for skybox
/// cube.
///
/// Cube-map is represented by six subtextures that must be square and the same
/// size. Sampling from cube-map is done as direction from origin. Skybox is an
/// application of cube-mapping where entire scene is wrapped in a large cube
/// surrounding the viewer and model. A unit cube is rendered centered
/// at the origin and uses the object space position as a texture coordinate
/// from which to sample the cube map texture.
///
/// Texture and vertex data are stored in GPU memory.
pub struct Skybox {
    gl: Arc<glow::Context>,
    pub texture: glow::Texture,
    pub vertex_array: VertexArray,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
}

impl Drop for Skybox {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_buffer(self.index_buffer);
            self.gl.delete_buffer(self.vertex_buffer);
            self.gl.delete_vertex_array(self.vertex_array);
            self.gl.delete_texture(self.texture);
        }
    }
}

cfg_if! { if #[cfg(not(target_arch = "wasm32"))] {
    #[derive(Default)]
    pub struct SkyboxFileBuilder {
        right_face_path: String,
        left_face_path: String,
        top_face_path: String,
        bottom_face_path: String,
        front_face_path: String,
        back_face_path: String,
    }

    impl SkyboxFileBuilder {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn with_right(mut self, right_face_path: &str) -> Self {
            self.right_face_path = right_face_path.to_string();
            self
        }

        pub fn with_left(mut self, left_face_path: &str) -> Self {
            self.left_face_path = left_face_path.to_string();
            self
        }

        pub fn with_top(mut self, top_face_path: &str) -> Self {
            self.top_face_path = top_face_path.to_string();
            self
        }

        pub fn with_bottom(mut self, bottom_face_path: &str) -> Self {
            self.bottom_face_path = bottom_face_path.to_string();
            self
        }

        pub fn with_front(mut self, front_face_path: &str) -> Self {
            self.front_face_path = front_face_path.to_string();
            self
        }

        pub fn with_back(mut self, back_face_path: &str) -> Self {
            self.back_face_path = back_face_path.to_string();
            self
        }

        pub fn build(self, gl: Arc<glow::Context>) -> Result<Skybox, String> {
            unsafe {
                let texture = gl.create_texture().unwrap();
                gl.bind_texture(glow::TEXTURE_CUBE_MAP, Some(texture));
                self.read_images_from_files(&gl).map_err(|e| {
                    format!("unable to create skybox texture: {:?}", e)
                })?;
                Ok(setup_shader_plumbing(gl, texture))
            }
        }

        fn read_images_from_files(&self, gl: &glow::Context) -> Result<(), String> {
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
                create_texture(&gl, i, &img);
            }

            Ok(())
        }
    }
} else {
    #[derive(Default)]
    pub struct SkyboxBufferBuilder {
        right_face_data: &'static [u8],
        left_face_data: &'static [u8],
        top_face_data: &'static [u8],
        bottom_face_data: &'static [u8],
        front_face_data: &'static [u8],
        back_face_data: &'static [u8],
    }

    impl SkyboxBufferBuilder {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn with_right(mut self, right_face_data: &'static [u8]) -> Self {
            self.right_face_data = right_face_data;
            self
        }

        pub fn with_left(mut self, left_face_data: &'static [u8]) -> Self {
            self.left_face_data = left_face_data;
            self
        }

        pub fn with_top(mut self, top_face_data: &'static [u8]) -> Self {
            self.top_face_data = top_face_data;
            self
        }

        pub fn with_bottom(mut self, bottom_face_data: &'static [u8]) -> Self {
            self.bottom_face_data = bottom_face_data;
            self
        }

        pub fn with_front(mut self, front_face_data: &'static [u8]) -> Self {
            self.front_face_data = front_face_data;
            self
        }

        pub fn with_back(mut self, back_face_data: &'static [u8]) -> Self {
            self.back_face_data = back_face_data;
            self
        }

        pub fn build(self, gl: Arc<glow::Context>) -> Result<Skybox, String> {
            unsafe {
                let texture = gl.create_texture().unwrap();
                gl.bind_texture(glow::TEXTURE_CUBE_MAP, Some(texture));
                self.read_images_from_buffers(&gl).map_err(|e| {
                    format!("unable to create skybox texture: {:?}", e)
                })?;
                Ok(setup_shader_plumbing(gl, texture))
            }
        }

        fn read_images_from_buffers(&self, gl: &glow::Context) -> Result<(), String> {
            let texture_face_paths: [&'static [u8]; 6] = [
                &self.right_face_data,
                &self.left_face_data,
                &self.top_face_data,
                &self.bottom_face_data,
                &self.front_face_data,
                &self.back_face_data,
            ];
            for (i, face_path) in texture_face_paths.iter().enumerate() {
                use image::ImageReader;
                let img = ImageReader::new(std::io::Cursor::new(face_path))
                    .with_guessed_format()
                    .map_err(|e| format!("failed to guess format for face {}: {:?}", i, e))?
                    .decode()
                    .map_err(|e| format!("failed to convert image for face {}: {:?}", i, e))?;
                create_texture(&gl, i, &img);
            }

            Ok(())
        }
    }
}}

fn create_texture(gl: &glow::Context, i: usize, img: &DynamicImage) {
    unsafe {
        gl.tex_image_2d(
            glow::TEXTURE_CUBE_MAP_POSITIVE_X + i as u32,
            0,
            glow::RGB as i32,
            img.width() as i32,
            img.height() as i32,
            0,
            glow::RGB,
            glow::UNSIGNED_BYTE,
            Some(img.to_rgb8().as_bytes()),
        );
    }
}

unsafe fn setup_shader_plumbing(gl: Arc<glow::Context>, texture: Texture) -> Skybox {
    gl.tex_parameter_i32(
        glow::TEXTURE_CUBE_MAP,
        glow::TEXTURE_MIN_FILTER,
        glow::LINEAR as i32,
    );
    gl.tex_parameter_i32(
        glow::TEXTURE_CUBE_MAP,
        glow::TEXTURE_MAG_FILTER,
        glow::LINEAR as i32,
    );
    gl.tex_parameter_i32(
        glow::TEXTURE_CUBE_MAP,
        glow::TEXTURE_WRAP_S,
        glow::CLAMP_TO_EDGE as i32,
    );
    gl.tex_parameter_i32(
        glow::TEXTURE_CUBE_MAP,
        glow::TEXTURE_WRAP_T,
        glow::CLAMP_TO_EDGE as i32,
    );
    gl.tex_parameter_i32(
        glow::TEXTURE_CUBE_MAP,
        glow::TEXTURE_WRAP_R,
        glow::CLAMP_TO_EDGE as i32,
    );

    // Create buffers

    #[rustfmt::skip]
    let skybox_vertices: [f32; 24] = [
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
    let skybox_indices: [u32; 36] = [
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
    let vertex_array = gl.create_vertex_array().unwrap();
    gl.bind_vertex_array(Some(vertex_array));

    // Create vertex buffer
    let vertex_buffer = gl.create_buffer().unwrap();
    gl.bind_buffer(glow::ARRAY_BUFFER, Some(vertex_buffer));
    let (_, vertices_bytes, _) = skybox_vertices.align_to::<u8>();
    gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, vertices_bytes, glow::STATIC_DRAW);

    // Create index buffer
    let index_buffer = gl.create_buffer().unwrap();
    gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(index_buffer));
    let (_, indices_bytes, _) = skybox_indices.align_to::<u8>();
    gl.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, indices_bytes, glow::STATIC_DRAW);

    // Setup vertex array layout (just vertex positions)
    let position_vertex_attribute = 0;
    let stride = 3 * size_of::<f32>() as i32;
    gl.enable_vertex_attrib_array(position_vertex_attribute);
    gl.vertex_attrib_pointer_f32(position_vertex_attribute, 3, glow::FLOAT, false, stride, 0);

    Skybox {
        gl,
        texture,
        vertex_array,
        vertex_buffer,
        index_buffer,
    }
}
