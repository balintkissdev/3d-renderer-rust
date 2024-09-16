use cfg_if::cfg_if;

// Collection of constants related to asset access.
//
// Majority of assets are accessed from file system on native builds and embedded into WASM binary
// on web target.
//
// TODO: Switch to Fetch API on web target instead of embedding assets into binary.

cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
        // Slight increase in startup time because lack of file system read calls for shader code.
        // No significant size increase in binary.
        pub mod shader {
            pub const MODEL_VERTEX_SRC: &str = include_str!("../assets/shaders/model_gl4.vert.glsl");
            pub const MODEL_FRAGMENT_SRC: &str = include_str!("../assets/shaders/model_gl4.frag.glsl");
            pub const SKYBOX_VERTEX_SRC: &str = include_str!("../assets/shaders/skybox_gl4.vert.glsl");
            pub const SKYBOX_FRAGMENT_SRC: &str = include_str!("../assets/shaders/skybox_gl4.frag.glsl");
        }

        pub mod skybox {
            pub const RIGHT_FACE_PATH: &str = "assets/skybox/right.jpg";
            pub const LEFT_FACE_PATH: &str = "assets/skybox/left.jpg";
            pub const TOP_FACE_PATH: &str = "assets/skybox/top.jpg";
            pub const BOTTOM_FACE_PATH: &str = "assets/skybox/bottom.jpg";
            pub const FRONT_FACE_PATH: &str = "assets/skybox/front.jpg";
            pub const BACK_FACE_PATH: &str = "assets/skybox/back.jpg";
        }

        pub mod model {
            pub const CUBE_PATH: &str = "assets/meshes/cube.obj";
            pub const TEAPOT_PATH: &str = "assets/meshes/teapot.obj";
            pub const BUNNY_PATH: &str = "assets/meshes/bunny.obj";
        }
    }
    else {
        pub mod shader {
            pub const MODEL_VERTEX_SRC: &str = include_str!("../assets/shaders/model_gles3.vert.glsl");
            pub const MODEL_FRAGMENT_SRC: &str = include_str!("../assets/shaders/model_gles3.frag.glsl");
            pub const SKYBOX_VERTEX_SRC: &str = include_str!("../assets/shaders/skybox_gles3.vert.glsl");
            pub const SKYBOX_FRAGMENT_SRC: &str = include_str!("../assets/shaders/skybox_gles3.frag.glsl");
        }

        pub mod skybox {
            pub const RIGHT_FACE_BYTES: &'static [u8] = include_bytes!("../assets/skybox/right.jpg");
            pub const LEFT_FACE_BYTES: &'static [u8] = include_bytes!("../assets/skybox/left.jpg");
            pub const TOP_FACE_BYTES: &'static [u8] = include_bytes!("../assets/skybox/top.jpg");
            pub const BOTTOM_FACE_BYTES: &'static [u8] = include_bytes!("../assets/skybox/bottom.jpg");
            pub const FRONT_FACE_BYTES: &'static [u8] = include_bytes!("../assets/skybox/front.jpg");
            pub const BACK_FACE_BYTES: &'static [u8] = include_bytes!("../assets/skybox/back.jpg");
        }

        pub mod model {
            pub const CUBE_BYTES: &'static [u8] = include_bytes!("../assets/meshes/cube.obj");
            pub const TEAPOT_BYTES: &'static [u8] = include_bytes!("../assets/meshes/teapot.obj");
            pub const BUNNY_BYTES: &'static [u8] = include_bytes!("../assets/meshes/bunny.obj");
        }
    }
}
