use cgmath::{InnerSpace, Matrix4, Point3, Vector2, Vector3};

// TODO: Make them configurable
const MOVEMENT_SPEED: f32 = 2.5;
const LOOK_SENSITIVITY: f32 = 0.1;

// Normalized mapping of positive Y axis in world coordinate space, always
// pointing upwards in the viewport (x:0, y:1, z:0). Required to determine
// the Right vector (mapping of positive X axis) when creating
// the view matrix.
const UP_VECTOR: Vector3<f32> = Vector3 {
    x: 0.0,
    y: 1.0,
    z: 0.0,
};

// Decoupling of camera view position and rotation manipulation.
//
// Application-side logic accepts user input and updates viewing properties
// through movement and look operations while renderer accesses the resulting
// view matrix to use for applying Model-View-Projection transformation.
pub struct Camera {
    // Camera location in world coordinate space. Also known as "eye
    // position".
    position: Point3<f32>,
    // Rotation elements are stored as Euler angles. Looking along X axis
    // (left/right, snapped around Y axis) is known as "yaw". Looking along Y
    // axis (up/down, snapped around X axis) us known as "pitch".
    //
    // Rolling around Z axis (like an aeroplane or spaceship) is omitted.
    rotation: Vector2<f32>,
    // Direction vector storing the rotations computed from mouse movements.
    // Determines where the camera should point at.
    direction: Vector3<f32>,
}

impl Camera {
    pub fn new(position: Point3<f32>, rotation: Vector2<f32>) -> Self {
        let mut camera = Self {
            position,
            rotation,
            direction: Vector3::new(0.0, 0.0, 0.0),
        };
        // Avoid camera jump on first mouselook.
        camera.update_direction();
        camera
    }

    pub fn move_forward(&mut self, delta_time: f32) {
        self.position += MOVEMENT_SPEED * self.direction * delta_time;
    }

    pub fn move_backward(&mut self, delta_time: f32) {
        self.position -= MOVEMENT_SPEED * self.direction * delta_time;
    }

    pub fn strafe_left(&mut self, delta_time: f32) {
        // If you don't normalize, you move fast or slow depending on camera
        // direction.
        self.position -= self.direction.cross(UP_VECTOR).normalize() * MOVEMENT_SPEED * delta_time;
    }

    pub fn strafe_right(&mut self, delta_time: f32) {
        self.position += self.direction.cross(UP_VECTOR).normalize() * MOVEMENT_SPEED * delta_time;
    }

    pub fn ascend(&mut self, delta_time: f32) {
        self.position += MOVEMENT_SPEED * UP_VECTOR * delta_time;
    }

    pub fn descend(&mut self, delta_time: f32) {
        self.position -= MOVEMENT_SPEED * UP_VECTOR * delta_time;
    }

    // Apply mouse input changes to change camera direction. Offsets are mouse
    // cursor distances from the center of the view.
    pub fn look(&mut self, x_offset: f32, y_offset: f32) {
        self.rotation.x += x_offset * LOOK_SENSITIVITY;
        // Wrap to keep rotation degrees displayed between 0 and 360 on debug UI
        self.rotation.x = wrap_yaw(self.rotation.x);

        self.rotation.y += y_offset * LOOK_SENSITIVITY;
        // Avoid user to do a backflip
        self.rotation.y = self.rotation.y.clamp(-89.0, 89.0);
        self.update_direction();
    }

    pub fn calculate_view_matrix(&self) -> Matrix4<f32> {
        let eye = self.position;
        let target = self.position + self.direction;
        // OpenGL uses right-handed coordinate system.
        Matrix4::look_at_rh(eye, target, UP_VECTOR)
    }

    pub fn position(&self) -> &Point3<f32> {
        &self.position
    }

    pub fn rotation(&self) -> &Vector2<f32> {
        &self.rotation
    }

    fn update_direction(&mut self) {
        let rotation_x_radians = self.rotation.x.to_radians();
        let rotation_y_radians = self.rotation.y.to_radians();
        self.direction.x = rotation_x_radians.cos() * rotation_y_radians.cos();
        self.direction.y = rotation_y_radians.sin();
        self.direction.z = rotation_x_radians.sin() * rotation_y_radians.cos();
        self.direction = self.direction.normalize();
    }
}

fn wrap_yaw(yaw: f32) -> f32 {
    let max = 359.0;
    let min = 0.0;
    if max < yaw {
        min
    } else if yaw < min {
        max
    } else {
        yaw
    }
}
