use wasm_bindgen::prelude::*;
use anyhow::Result;
use log::info;
use nalgebra as na;

pub mod camera;
pub mod renderer;
pub mod transfer_function;
pub mod tiff_loader;

use camera::Camera;
use renderer::VolumeRenderer;
use transfer_function::TransferFunction;

#[derive(Default)]
pub struct VolumeData {
    pub raw_data: Vec<f32>,
    pub dimensions: (usize, usize, usize),
    pub value_range: (f32, f32),
}

impl VolumeData {
    pub fn load_tiff_from_memory(&mut self, data: &[u8]) -> Result<()> {
        let slices = tiff_loader::load_tiff_from_memory(data)?;

        if slices.is_empty() {
            return Err(anyhow::anyhow!("No valid slices found in TIFF"));
        }

        let width = slices[0].width;
        let height = slices[0].height;
        let depth = slices.len();

        let total_size = width
            .checked_mul(height)
            .and_then(|wh| wh.checked_mul(depth))
            .ok_or_else(|| anyhow::anyhow!("Integer overflow in size calculation"))?;

        const MAX_SIZE: usize = 256 * 1024 * 1024 / 4; // 256MB limit
        if total_size > MAX_SIZE {
            return Err(anyhow::anyhow!("Image data too large to fit in memory"));
        }

        // Check slice compatibility
        for slice in &slices[1..] {
            if slice.width != width || slice.height != height {
                return Err(anyhow::anyhow!("Inconsistent slice dimensions"));
            }
        }

        let is_16bit = slices[0].data.iter().any(|&x| x > 255_u16);
        let max_possible = if is_16bit { 65535.0 } else { 255.0 };

        let mut combined_data = Vec::with_capacity(total_size);

        for slice in &slices {
            combined_data.extend(slice.data.iter().map(|&v| v as f32));
        }

        self.raw_data = combined_data;
        self.dimensions = (width, height, depth);
        self.value_range = (0.0, max_possible);

        info!("Loaded volume: {}x{}x{}", width, height, depth);
        info!("Value range: {} to {}", 0.0, max_possible);

        Ok(())
    }

    pub fn sample(&self, x: usize, y: usize, z: usize) -> Option<f32> {
        let (width, height, depth) = self.dimensions;
        if x >= width || y >= height || z >= depth {
            return None;
        }

        let index = z * width * height + y * width + x;
        self.raw_data.get(index).copied()
    }

    pub fn get_normalized_value(&self, value: f32) -> f32 {
        let (min, max) = self.value_range;
        if max == min {
            return 0.0;
        }
        (value - min) / (max - min)
    }
}

#[wasm_bindgen]
pub struct VolumeViewer {
    volume_data: Option<VolumeData>,
    camera: Camera,
    renderer: VolumeRenderer,
    transfer_func: TransferFunction,
}

#[wasm_bindgen]
impl VolumeViewer {
    #[wasm_bindgen(constructor)]
    pub fn new(width: usize, height: usize) -> Result<VolumeViewer, JsValue> {
        console_error_panic_hook::set_once();
        
        if width == 0 || height == 0 {
            return Err(JsValue::from_str("Invalid viewer dimensions"));
        }

        const MAX_DIMENSION: usize = 16384;
        if width > MAX_DIMENSION || height > MAX_DIMENSION {
            return Err(JsValue::from_str("Viewer dimensions too large"));
        }
        
        Ok(Self {
            volume_data: None,
            camera: Camera::default(),
            renderer: VolumeRenderer::new(width, height),
            transfer_func: TransferFunction::default(),
        })
    }

    #[wasm_bindgen]
    pub fn load_volume(&mut self, data: &[u8]) -> Result<js_sys::Array, JsValue> {
        let mut volume = VolumeData::default();
        volume.load_tiff_from_memory(data)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let dimensions = volume.dimensions;
        let result = js_sys::Array::new();
        result.push(&JsValue::from_f64(dimensions.0 as f64));
        result.push(&JsValue::from_f64(dimensions.1 as f64));
        result.push(&JsValue::from_f64(dimensions.2 as f64));

        self.volume_data = Some(volume);
        Ok(result)
    }

    #[wasm_bindgen]
    pub fn render(&mut self) -> Vec<u8> {
        if let Some(ref volume) = self.volume_data {
            self.renderer.render(volume, &self.camera, &self.transfer_func);
            self.renderer.framebuffer.clone()
        } else {
            vec![0; self.renderer.width * self.renderer.height * 4]
        }
    }

    #[wasm_bindgen]
    pub fn orbit(&mut self, delta_theta: f32, delta_phi: f32) -> Result<(), JsValue> {
        if !delta_theta.is_finite() || !delta_phi.is_finite() {
            return Err(JsValue::from_str("Invalid orbit parameters"));
        }

        let volume = self.volume_data.as_ref()
            .ok_or_else(|| JsValue::from_str("No volume data loaded"))?;

        let (width, height, depth) = volume.dimensions;
        if width == 0 || height == 0 || depth == 0 {
            return Err(JsValue::from_str("Invalid volume dimensions"));
        }

        let delta_theta = delta_theta.clamp(-1.0, 1.0);
        let delta_phi = delta_phi.clamp(-1.0, 1.0);

        self.camera.orbit(delta_theta, delta_phi);
        Ok(())
    }

    #[wasm_bindgen]
    pub fn zoom(&mut self, delta: f32) -> Result<(), JsValue> {
        if !delta.is_finite() {
            return Err(JsValue::from_str("Invalid zoom delta"));
        }

        let volume = self.volume_data.as_ref()
            .ok_or_else(|| JsValue::from_str("No volume data loaded"))?;

        let (width, height, depth) = volume.dimensions;
        if width == 0 || height == 0 || depth == 0 {
            return Err(JsValue::from_str("Invalid volume dimensions"));
        }

        let clamped_delta = delta.clamp(-1.0, 1.0);
        self.camera.zoom(clamped_delta);
        Ok(())
    }

    #[wasm_bindgen]
    pub fn resize(&mut self, width: usize, height: usize) -> Result<(), JsValue> {
        if width == 0 || height == 0 {
            return Err(JsValue::from_str("Invalid resize dimensions"));
        }

        const MAX_DIMENSION: usize = 16384;
        if width > MAX_DIMENSION || height > MAX_DIMENSION {
            return Err(JsValue::from_str("Resize dimensions too large"));
        }

        self.renderer = VolumeRenderer::new(width, height);
        Ok(())
    }

    #[wasm_bindgen]
    pub fn pan(&mut self, delta: &[f32]) -> Result<(), JsValue> {
        if delta.len() != 2 {
            return Err(JsValue::from_str("Pan delta must be an array of 2 numbers"));
        }

        if !delta.iter().all(|&x| x.is_finite()) {
            return Err(JsValue::from_str("Invalid pan parameters"));
        }

        let volume = self.volume_data.as_ref()
            .ok_or_else(|| JsValue::from_str("No volume data loaded"))?;

        let (width, height, depth) = volume.dimensions;
        if width == 0 || height == 0 || depth == 0 {
            return Err(JsValue::from_str("Invalid volume dimensions"));
        }

        let pan_delta = na::Vector2::new(delta[0], delta[1]);
        
        let dx = pan_delta[0].clamp(-10.0, 10.0);
        let dy = pan_delta[1].clamp(-10.0, 10.0);
        let clamped_delta = na::Vector2::new(dx, dy);

        self.camera.pan(&clamped_delta);
        Ok(())
    }
}

