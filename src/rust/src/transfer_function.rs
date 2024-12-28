pub struct TransferFunction {
    control_points: Vec<ControlPoint>,
    cached_colors: Vec<[f32; 4]>,
}

#[derive(Clone, Copy)]
struct ControlPoint {
    value: f32,
    color: [f32; 4],  // RGBA
}

impl Default for TransferFunction {
    fn default() -> Self {
        let mut tf = Self {
            control_points: vec![
                ControlPoint { value: 0.0, color: [0.0, 0.0, 0.0, 0.0] },
                ControlPoint { value: 1.0, color: [1.0, 1.0, 1.0, 1.0] },
            ],
            cached_colors: vec![[0.0; 4]; 256],
        };
        tf.update_cache();
        tf
    }
}

impl TransferFunction {
    pub fn get_color(&self, value: f32) -> [f32; 4] {
        // For 2D images, use direct grayscale mapping
        let v = value.clamp(0.0, 1.0);
        [v, v, v, 1.0]
    }

    pub fn get_color_3d(&self, value: f32) -> [f32; 4] {
        // For 3D volumes, use the cached transfer function
        let index = ((value.clamp(0.0, 1.0) * 255.0) as usize).min(255);
        self.cached_colors[index]
    }

    pub fn add_point(&mut self, value: f32, color: [f32; 4]) {
        let value = value.clamp(0.0, 1.0);
        let index = self.control_points
            .partition_point(|p| p.value <= value);
        
        self.control_points.insert(index, ControlPoint { value, color });
        self.update_cache();
    }

    pub fn remove_point(&mut self, index: usize) {
        if self.control_points.len() > 2 && index < self.control_points.len() {
            self.control_points.remove(index);
            self.update_cache();
        }
    }

    pub fn get_points(&self) -> &[(f32, [f32; 4])] {
        unsafe {
            std::slice::from_raw_parts(
                self.control_points.as_ptr() as *const (f32, [f32; 4]),
                self.control_points.len()
            )
        }
    }

    pub fn get_points_mut(&mut self) -> &mut [(f32, [f32; 4])] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.control_points.as_mut_ptr() as *mut (f32, [f32; 4]),
                self.control_points.len()
            )
        }
    }

    fn update_cache(&mut self) {
        for i in 0..256 {
            let value = i as f32 / 255.0;
            
            let idx = self.control_points
                .partition_point(|p| p.value <= value)
                .saturating_sub(1);
            
            if idx + 1 < self.control_points.len() {
                let p1 = &self.control_points[idx];
                let p2 = &self.control_points[idx + 1];
                
                let t = (value - p1.value) / (p2.value - p1.value);
                let mut color = [0.0; 4];
                
                for j in 0..4 {
                    color[j] = p1.color[j] * (1.0 - t) + p2.color[j] * t;
                }
                
                self.cached_colors[i] = color;
            } else {
                self.cached_colors[i] = self.control_points.last().unwrap().color;
            }
        }
    }

    // Predefined transfer functions
    pub fn grayscale() -> Self {
        let mut tf = Self::default();
        tf.update_cache();
        tf
    }

    pub fn rainbow() -> Self {
        let mut tf = Self {
            control_points: vec![
                ControlPoint { value: 0.0, color: [0.0, 0.0, 1.0, 0.0] },  // Blue
                ControlPoint { value: 0.25, color: [0.0, 1.0, 1.0, 0.25] }, // Cyan
                ControlPoint { value: 0.5, color: [0.0, 1.0, 0.0, 0.5] },  // Green
                ControlPoint { value: 0.75, color: [1.0, 1.0, 0.0, 0.75] }, // Yellow
                ControlPoint { value: 1.0, color: [1.0, 0.0, 0.0, 1.0] },  // Red
            ],
            cached_colors: vec![[0.0; 4]; 256],
        };
        tf.update_cache();
        tf
    }

    pub fn hot() -> Self {
        let mut tf = Self {
            control_points: vec![
                ControlPoint { value: 0.0, color: [0.0, 0.0, 0.0, 0.0] },
                ControlPoint { value: 0.33, color: [1.0, 0.0, 0.0, 0.33] },
                ControlPoint { value: 0.66, color: [1.0, 1.0, 0.0, 0.66] },
                ControlPoint { value: 1.0, color: [1.0, 1.0, 1.0, 1.0] },
            ],
            cached_colors: vec![[0.0; 4]; 256],
        };
        tf.update_cache();
        tf
    }

    pub fn cool() -> Self {
        let mut tf = Self {
            control_points: vec![
                ControlPoint { value: 0.0, color: [0.0, 1.0, 1.0, 0.0] },
                ControlPoint { value: 0.5, color: [0.0, 0.0, 1.0, 0.5] },
                ControlPoint { value: 1.0, color: [1.0, 0.0, 1.0, 1.0] },
            ],
            cached_colors: vec![[0.0; 4]; 256],
        };
        tf.update_cache();
        tf
    }
}