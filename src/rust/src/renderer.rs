use nalgebra as na;
use log::{info, debug};
use crate::{VolumeData, camera::Camera, transfer_function::TransferFunction};

pub struct VolumeRenderer {
    pub framebuffer: Vec<u8>,
    pub width: usize,
    pub height: usize,
    ray_step: f32,
}

struct Ray {
    origin: na::Point3<f32>,
    direction: na::Vector3<f32>,
}

impl Ray {
    fn at(&self, t: f32) -> na::Point3<f32> {
        self.origin + self.direction * t
    }

    fn intersect_box(&self, min: &na::Point3<f32>, max: &na::Point3<f32>) -> Option<(f32, f32)> {
        let mut t_min = f32::NEG_INFINITY;
        let mut t_max = f32::INFINITY;
        
        for i in 0..3 {
            let inv_dir = 1.0 / self.direction[i];
            let mut t0 = (min[i] - self.origin[i]) * inv_dir;
            let mut t1 = (max[i] - self.origin[i]) * inv_dir;
            
            if inv_dir < 0.0 {
                std::mem::swap(&mut t0, &mut t1);
            }
            
            t_min = t_min.max(t0);
            t_max = t_max.min(t1);
            
            if t_max <= t_min {
                return None;
            }
        }
        
        Some((t_min, t_max))
    }
}

impl VolumeRenderer {
    pub fn new(width: usize, height: usize) -> Self {
        info!("Creating VolumeRenderer with dimensions: {}x{}", width, height);
        Self {
            framebuffer: vec![0; width * height * 4],
            width,
            height,
            ray_step: 0.005,
        }
    }

    pub fn render(&mut self, volume: &VolumeData, camera: &Camera, transfer_func: &TransferFunction) {
        debug!("Starting volume render with dimensions: {:?}", volume.dimensions);
        
        let aspect_ratio = self.width as f32 / self.height as f32;
        let view = camera.view_matrix();
        let proj = camera.projection_matrix(aspect_ratio);
        let view_proj = proj * view;
        let inv_view_proj = view_proj.try_inverse().unwrap();

        // Scale volume bounds based on dimensions
        let max_dim = volume.dimensions.0.max(volume.dimensions.1).max(volume.dimensions.2) as f32;
        let scale = 1.0 / max_dim;
        let volume_min = na::Point3::new(
            -0.5 * volume.dimensions.0 as f32 * scale,
            -0.5 * volume.dimensions.1 as f32 * scale,
            -0.5 * volume.dimensions.2 as f32 * scale
        );
        let volume_max = na::Point3::new(
            0.5 * volume.dimensions.0 as f32 * scale,
            0.5 * volume.dimensions.1 as f32 * scale,
            0.5 * volume.dimensions.2 as f32 * scale
        );

        debug!("Volume bounds: min={:?}, max={:?}", volume_min, volume_max);

        let mut hit_count = 0;
        let sample_count = 0;

        for y in 0..self.height {
            for x in 0..self.width {
                let ray = self.generate_ray(x, y, &inv_view_proj);
                let color = if volume.dimensions.2 == 1 {
                    // 2D image mode
                    self.cast_ray_2d(&ray, volume, transfer_func, volume_min, volume_max)
                } else {
                    // 3D volume mode
                    self.cast_ray_3d(&ray, volume, transfer_func, volume_min, volume_max)
                };
                
                if color[3] > 0 {
                    hit_count += 1;
                }
                
                let idx = (y * self.width + x) * 4;
                self.framebuffer[idx..idx + 4].copy_from_slice(&color);
            }
        }

        debug!("Render complete. Hit count: {}, Sample count: {}", hit_count, sample_count);
    }

    fn generate_ray(&self, x: usize, y: usize, inv_view_proj: &na::Matrix4<f32>) -> Ray {
        let ndc_x = (2.0 * x as f32 / self.width as f32) - 1.0;
        let ndc_y = 1.0 - (2.0 * y as f32 / self.height as f32);
        
        let near = inv_view_proj.transform_point(&na::Point3::new(ndc_x, ndc_y, -1.0));
        let far = inv_view_proj.transform_point(&na::Point3::new(ndc_x, ndc_y, 1.0));
        
        Ray {
            origin: near,
            direction: (far - near).normalize(),
        }
    }

    fn cast_ray_2d(
        &self,
        ray: &Ray,
        volume: &VolumeData,
        transfer_func: &TransferFunction,
        volume_min: na::Point3<f32>,
        volume_max: na::Point3<f32>,
    ) -> [u8; 4] {
        if let Some((t_min, t_max)) = ray.intersect_box(&volume_min, &volume_max) {
            // For 2D, sample at the intersection point
            let pos = ray.at((t_min + t_max) * 0.5);
            
            // Convert from normalized coordinates to image coordinates
            let x = ((pos.x + 0.5) * volume.dimensions.0 as f32).clamp(0.0, volume.dimensions.0 as f32 - 1.0);
            let y = ((pos.y + 0.5) * volume.dimensions.1 as f32).clamp(0.0, volume.dimensions.1 as f32 - 1.0);
            
            let x_idx = x.floor() as usize;
            let y_idx = y.floor() as usize;
            
            if let Some(value) = volume.sample(x_idx, y_idx, 0) {
                let normalized = volume.get_normalized_value(value);
                let color = transfer_func.get_color(normalized);
                
                return [
                    (color[0] * 255.0) as u8,
                    (color[1] * 255.0) as u8,
                    (color[2] * 255.0) as u8,
                    255,  // Full opacity for 2D
                ];
            }
        }
        [0, 0, 0, 0]
    }

    fn cast_ray_3d(
        &self,
        ray: &Ray,
        volume: &VolumeData,
        transfer_func: &TransferFunction,
        volume_min: na::Point3<f32>,
        volume_max: na::Point3<f32>,
    ) -> [u8; 4] {
        if let Some((t_min, t_max)) = ray.intersect_box(&volume_min, &volume_max) {
            let mut color = [0.0f32; 4];
            let mut alpha = 0.0f32;
            let mut t = t_min;
            
            while t < t_max && alpha < 0.99 {
                let pos = ray.at(t);
                
                // Convert from normalized space to volume space
                let max_dim = volume.dimensions.0.max(volume.dimensions.1).max(volume.dimensions.2) as f32;
                let scale = max_dim;
                let sample_pos = na::Point3::new(
                    (pos.x * scale + 0.5 * volume.dimensions.0 as f32).clamp(0.0, volume.dimensions.0 as f32 - 1.0),
                    (pos.y * scale + 0.5 * volume.dimensions.1 as f32).clamp(0.0, volume.dimensions.1 as f32 - 1.0),
                    (pos.z * scale + 0.5 * volume.dimensions.2 as f32).clamp(0.0, volume.dimensions.2 as f32 - 1.0),
                );
                
                let x = sample_pos.x.floor() as usize;
                let y = sample_pos.y.floor() as usize;
                let z = sample_pos.z.floor() as usize;
                
                if let Some(value) = volume.sample(x, y, z) {
                    let normalized = volume.get_normalized_value(value);
                    let sample_color = transfer_func.get_color(normalized);
                    
                    // Front-to-back compositing
                    let a = sample_color[3] * self.ray_step * 10.0 * (1.0 - alpha);
                    for i in 0..3 {
                        color[i] += sample_color[i] * a;
                    }
                    alpha += a;
                }
                
                t += self.ray_step;
            }
            
            // Convert to u8
            [
                (color[0] * 255.0) as u8,
                (color[1] * 255.0) as u8,
                (color[2] * 255.0) as u8,
                ((alpha * 5.0).min(1.0) * 255.0) as u8,
            ]
        } else {
            [0, 0, 0, 0]
        }
    }
}