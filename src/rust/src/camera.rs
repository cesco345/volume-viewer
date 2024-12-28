// use nalgebra as na;

// pub struct Camera {
//     pub position: na::Point3<f32>,
//     pub target: na::Point3<f32>,
//     pub up: na::Vector3<f32>,
//     pub fov: f32,
//     pub near: f32,
//     pub far: f32,
//     pub orbit_angles: na::Vector2<f32>,  // theta (yaw), phi (pitch)
//     pub distance: f32,
// }

// impl Default for Camera {
//     fn default() -> Self {
//         Self {
//             position: na::Point3::new(0.0, 0.0, 5.0),
//             target: na::Point3::origin(),
//             up: na::Vector3::new(0.0, 1.0, 0.0),
//             fov: 45.0_f32.to_radians(),
//             near: 0.1,
//             far: 100.0,
//             orbit_angles: na::Vector2::new(0.0, std::f32::consts::FRAC_PI_4),
//             distance: 5.0,
//         }
//     }
// }

// impl Camera {
//     pub fn view_matrix(&self) -> na::Matrix4<f32> {
//         na::Matrix4::look_at_rh(&self.position, &self.target, &self.up)
//     }

//     pub fn projection_matrix(&self, aspect_ratio: f32) -> na::Matrix4<f32> {
//         na::Matrix4::new_perspective(aspect_ratio, self.fov, self.near, self.far)
//     }

//     pub fn orbit(&mut self, delta_theta: f32, delta_phi: f32) {
//         // Update yaw angle (horizontal rotation)
//         self.orbit_angles.x += delta_theta;
        
//         // Update pitch angle (vertical rotation)
//         let new_phi = self.orbit_angles.y + delta_phi;
//         self.orbit_angles.y = new_phi;
        
//         // Keep yaw in [0, 2π]
//         if self.orbit_angles.x < 0.0 {
//             self.orbit_angles.x += 2.0 * std::f32::consts::PI;
//         } else if self.orbit_angles.x > 2.0 * std::f32::consts::PI {
//             self.orbit_angles.x -= 2.0 * std::f32::consts::PI;
//         }
        
//         self.update_position();
//     }

//     pub fn zoom(&mut self, delta: f32) {
//         self.distance = (self.distance * (1.0 + delta)).max(self.near * 2.0);
//         self.update_position();
//     }

//     pub fn pan(&mut self, delta: &na::Vector2<f32>) {
//         let forward = (self.target - self.position).normalize();
//         let right = forward.cross(&self.up).normalize();
//         // Calculate up vector in the camera's local space
//         let camera_up = right.cross(&forward).normalize();
        
//         self.target += right * delta.x * self.distance * 0.001 
//                     + camera_up * delta.y * self.distance * 0.001;
//         self.update_position();
//     }

//     fn update_position(&mut self) {
//         let theta = self.orbit_angles.x; // Horizontal angle
//         let phi = self.orbit_angles.y;   // Vertical angle
        
//         // Calculate the position using quaternion rotation
//         let rotation = na::UnitQuaternion::from_euler_angles(0.0, phi, theta);
//         let initial_pos = na::Vector3::new(0.0, 0.0, self.distance);
//         let rotated_pos = rotation * initial_pos;
        
//         self.position = self.target + rotated_pos;
        
//         // Update up vector based on rotation
//         let initial_up = na::Vector3::new(0.0, 1.0, 0.0);
//         self.up = rotation * initial_up;
//     }
// }
use nalgebra as na;

pub struct Camera {
    pub position: na::Point3<f32>,
    pub target: na::Point3<f32>,
    pub up: na::Vector3<f32>,
    pub fov: f32,
    pub near: f32,
    pub far: f32,
    pub orbit_angles: na::Vector2<f32>,  // theta (yaw), phi (pitch)
    pub distance: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: na::Point3::new(0.0, 0.0, 5.0),
            target: na::Point3::origin(),
            up: na::Vector3::new(0.0, 1.0, 0.0),
            fov: 45.0_f32.to_radians(),
            near: 0.1,
            far: 100.0,
            orbit_angles: na::Vector2::new(0.0, std::f32::consts::FRAC_PI_4),
            distance: 5.0,
        }
    }
}

impl Camera {
    pub fn view_matrix(&self) -> na::Matrix4<f32> {
        na::Matrix4::look_at_rh(&self.position, &self.target, &self.up)
    }

    pub fn projection_matrix(&self, aspect_ratio: f32) -> na::Matrix4<f32> {
        na::Matrix4::new_perspective(aspect_ratio, self.fov, self.near, self.far)
    }

    pub fn orbit(&mut self, delta_theta: f32, delta_phi: f32) {
        // Update yaw angle (horizontal rotation)
        self.orbit_angles.x += delta_theta;
        
        // Update pitch angle (vertical rotation)
        let new_phi = self.orbit_angles.y + delta_phi;
        self.orbit_angles.y = new_phi;
        
        // Keep yaw in [0, 2π]
        if self.orbit_angles.x < 0.0 {
            self.orbit_angles.x += 2.0 * std::f32::consts::PI;
        } else if self.orbit_angles.x > 2.0 * std::f32::consts::PI {
            self.orbit_angles.x -= 2.0 * std::f32::consts::PI;
        }
        
        self.update_position();
    }

    pub fn zoom(&mut self, delta: f32) {
        self.distance = (self.distance * (1.0 + delta)).max(self.near * 2.0);
        self.update_position();
    }

    pub fn pan(&mut self, delta: &na::Vector2<f32>) {
        let forward = (self.target - self.position).normalize();
        let right = forward.cross(&self.up).normalize();
        let camera_up = right.cross(&forward).normalize();
        
        self.target += right * delta.x * self.distance * 0.001 
                    + camera_up * delta.y * self.distance * 0.001;
        self.update_position();
    }

    fn update_position(&mut self) {
        let theta = self.orbit_angles.x; // Horizontal angle
        let phi = self.orbit_angles.y;   // Vertical angle
        
        // Calculate the position using quaternion rotation
        let rotation = na::UnitQuaternion::from_euler_angles(0.0, phi, theta);
        let initial_pos = na::Vector3::new(0.0, 0.0, self.distance);
        let rotated_pos = rotation * initial_pos;
        
        self.position = self.target + rotated_pos;
        
        // Update up vector based on rotation
        let initial_up = na::Vector3::new(0.0, 1.0, 0.0);
        self.up = rotation * initial_up;
    }

    pub fn reset(&mut self) {
        self.position = na::Point3::new(0.0, 0.0, 5.0);
        self.target = na::Point3::origin();
        self.up = na::Vector3::new(0.0, 1.0, 0.0);
        self.orbit_angles = na::Vector2::new(0.0, std::f32::consts::FRAC_PI_4);
        self.distance = 5.0;
        self.update_position();
    }
}