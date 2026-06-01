use std::path::Path;
use crate::color::extractor::{ColorExtractingStrategy, ColorPalette, ThemeMode};

pub struct NativeExtractor;

impl NativeExtractor {
    pub fn new() -> Self {
        Self
    }

    /// Reads an image, shrinks it to a manageable size, 
    /// and returns it as a 2D matrix of RGB pixels.
    fn get_pixel_matrix(&self, image_path: &Path, max_dimension: u32) -> Result<Vec<Vec<[u8; 3]>>, String> {
        // 1. Open the image (automatically handles various formats)
        let img = image::open(image_path)
            .map_err(|e| format!("Failed to open image: {}", e))?;

        // 2. Shrink the image significantly for performance.
        // `thumbnail` is an optimized function that resizes the image while preserving aspect ratio.
        // If an image is 4K, this will quickly scale it down to `max_dimension` while retaining general color data.
        let small_img = img.thumbnail(max_dimension, max_dimension);
        
        // 3. Convert all images (even PNGs with transparency) into a unified standard RGB format
        let rgb_img = small_img.to_rgb8();
        let (width, height) = rgb_img.dimensions();

        // 4. Construct our 2D matrix (Vec of Vecs where each element is [R, G, B])
        let mut matrix = Vec::with_capacity(height as usize);
        
        for y in 0..height {
            let mut row = Vec::with_capacity(width as usize);
            for x in 0..width {
                let pixel = rgb_img.get_pixel(x, y);
                // pixel.0 gives us the underlying [u8; 3] array
                row.push(pixel.0); 
            }
            matrix.push(row);
        }

        Ok(matrix)
    }
}

impl ColorExtractingStrategy for NativeExtractor {
    fn extract(&self, image_path: &Path, _mode: ThemeMode) -> Result<ColorPalette, String> {
        
        // We use 64x64 or 128x128 as a maximum dimension. 
        // A 64x64 image has 4,096 pixels, which is PLENTY of data for clustering a color palette 
        // but is practically instant to process, even if the source was a massive 4K wallpaper.
        let _pixel_matrix = self.get_pixel_matrix(image_path, 64)?;

        // TODO: Pass this matrix into a clustering algorithm (like K-Means) to find the dominant colors.
        
        // For now, we just return an error so the application knows it's pending.
        Err("Native extraction logic is not yet implemented".to_string())
    }
}