use anyhow::Result;
use std::io::Cursor;
use image::{DynamicImage, imageops};
use tiff::decoder::{Decoder, DecodingResult};
use log::{debug, info};

pub struct ImageInfo {
    pub data: Vec<u16>,
    pub width: usize,
    pub height: usize,
}

fn handle_exif_orientation(img: DynamicImage) -> DynamicImage {
    DynamicImage::ImageRgba8(imageops::rotate180(&img.to_rgba8()))
}

fn convert_rgb_to_grayscale(rgb_data: &[u8], width: usize, height: usize) -> Vec<u16> {
    let mut grayscale = Vec::with_capacity(width * height);
    
    for y in 0..height {
        for x in 0..width {
            let base_idx = (y * width + x) * 3;
            
            if base_idx + 2 < rgb_data.len() {
                let r = rgb_data[base_idx] as f32;
                let g = rgb_data[base_idx + 1] as f32;
                let b = rgb_data[base_idx + 2] as f32;
                
                let gray = (0.2989 * r + 0.5870 * g + 0.1140 * b) as u16;
                grayscale.push(gray);
            } else {
                grayscale.push(0);
            }
        }
    }
    
    grayscale
}

pub fn load_tiff_from_memory(data: &[u8]) -> Result<Vec<ImageInfo>> {
    let mut decoder = Decoder::new(Cursor::new(data))?;
    let mut slices: Vec<ImageInfo> = Vec::new();
    
    let dimensions = decoder.dimensions()?;
    let width = dimensions.0 as usize;
    let height = dimensions.1 as usize;
    
    if width > 8192 || height > 8192 {
        return Err(anyhow::anyhow!("Image dimensions too large"));
    }
    
    const MAX_SLICES: usize = 512;
    
    loop {
        match decoder.read_image()? {
            DecodingResult::U8(data) => {
                let slice_data = match decoder.colortype()? {
                    tiff::ColorType::RGB(8) => {
                        info!("Converting RGB to grayscale");
                        convert_rgb_to_grayscale(&data, width, height)
                    },
                    tiff::ColorType::Gray(8) => {
                        info!("Converting 8-bit grayscale");
                        data.into_iter().map(|v| u16::from(v)).collect()
                    },
                    _ => {
                        return Err(anyhow::anyhow!("Unsupported color format"));
                    }
                };
                
                slices.push(ImageInfo { data: slice_data, width, height });
            },
            DecodingResult::U16(data) => {
                slices.push(ImageInfo { data, width, height });
            },
            _ => break,
        }
        
        if slices.len() >= MAX_SLICES {
            debug!("Maximum number of slices reached");
            break;
        }
        
        if !decoder.more_images() {
            debug!("No more images in TIFF");
            break;
        }
        
        decoder.next_image()?;
    }
    
    if slices.is_empty() {
        return Err(anyhow::anyhow!("No valid image data found in TIFF"));
    }
    
    debug!("Successfully loaded {} slices", slices.len());
    Ok(slices)
}








