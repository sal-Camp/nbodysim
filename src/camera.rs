use winit::event::*;

/// Camera struct to hold our camera's values
pub struct Camera {
    /// Where the camera is looking "from"
    pub eye: cgmath::Point3<f32>,
    /// What the camera is looking at
    pub target: cgmath::Point3<f32>,
    /// The camera's local up axis
    pub up: cgmath::Vector3<f32>,
    /// The camera's aspect ratio
    pub aspect: f32,
    /// The camera's field of view
    pub fovy: f32,
    /// Znear and Zfar describe our clipping distance
    pub znear: f32,
    pub zfar: f32,
}

/// Since wgpu and cgmath are built for different cooridinate systems,
/// we'll use this matrix to convert between them.
#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

impl Camera {
    /// Defines a new camera
    pub fn new(width: f32, height: f32) -> Self {
        // Positioning the camera above and behind the world space origin
        let eye = (0.0, 1.0, 2.0).into();
        // Setting the camera to look at the origin
        let target = (0.0, 0.0, 0.0).into();
        // Determining our up direction
        let up = cgmath::Vector3::unit_y();
        let aspect = width as f32 / height as f32;
        let fovy = 45.0;
        let znear = 0.1;
        let zfar = 100.0;

        Self {
            eye,
            target,
            up,
            aspect,
            fovy,
            znear,
            zfar,
        }
    }
    fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        // View moves the world to be at the position and rotation of the camera
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        // Proj wraps the scene to give depth
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);
        // Using our conversion matrix to convert our camera cooridinates
        return OPENGL_TO_WGPU_MATRIX * proj * view;
    }
}

// This ensures Rust will store data the same way as C would for shader compatibility
#[repr(C)]
// Deriving the following traits for our camera uniform
// Allows us to store the uniform in a buffer
// Pod ensures the struct the struct follows certain constraints such as using #[repr(C)]
// Zeroable ensures a type can be "zeroed" out
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    // cgmath & bytemuck don't work together
    // So convert mat4 to a 4x4 f32 array
    view_proj: [[f32; 4]; 4],
}

use cgmath::*;

impl CameraUniform {
    /// Declares a new camera uniform
    pub fn new() -> Self {
        Self {
            // This essentially converts a matrix into our view_proj array
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    /// Updates the camera's view projection as needed by rebuilding it
    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}

/// The struct that defines our keybindings and camera sensitivity
pub struct CameraController {
    /// The camera's speed at which it moves
    speed: f32,
    // The following are our keybinding bools
    is_up_pressed: bool,
    is_down_pressed: bool,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
}

impl CameraController {
    /// Defines a new camera with the parameterized speed and all key presses set to false
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            is_up_pressed: false,
            is_down_pressed: false,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
        }
    }

    /// Parses our keyboard events and performs actions as required per key
    pub fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
                    VirtualKeyCode::Space => {
                        self.is_up_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::LShift => {
                        self.is_down_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::W | VirtualKeyCode::Up => {
                        self.is_forward_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::A | VirtualKeyCode::Left => {
                        self.is_left_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::S | VirtualKeyCode::Down => {
                        self.is_backward_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::D | VirtualKeyCode::Right => {
                        self.is_right_pressed = is_pressed;
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    /// If a key is pressed, will update the camera as necessary
    pub fn update_camera(&self, camera: &mut Camera) {
        // Definding our forward vector
        let forward = camera.target - camera.eye;
        // Normalizing the forward vector
        let forward_norm = forward.normalize();
        // Defining the magnitude of the forward vector
        let forward_mag = forward.magnitude();

        // Prevents glitching when camera gets too close to the
        // center of the scene.
        if self.is_forward_pressed && forward_mag > self.speed {
            camera.eye += forward_norm * self.speed;
        }
        if self.is_backward_pressed {
            camera.eye -= forward_norm * self.speed;
        }

        let right = forward_norm.cross(camera.up);

        // Redo the calculations if up/down is pressed
        let forward = camera.target - camera.eye;
        let forward_mag = forward.magnitude();

        if self.is_right_pressed {
            // Ensures the distance between the eye and target is consistent
            camera.eye = camera.target - (forward + right * self.speed).normalize() * forward_mag;
        }
        if self.is_left_pressed {
            camera.eye = camera.target - (forward - right * self.speed).normalize() * forward_mag;
        }
    }
}
