// src/image_util.rs
use image::{DynamicImage, GenericImageView, imageops::FilterType, ImageError};

// We make the struct and fields public so other files can read them
pub struct MyImage {
    pub data: DynamicImage,
    pub path: String,
}

impl MyImage {
    pub fn load(path: String) -> Result<Self, ImageError> {
        let data = image::open(&path)?;
        Ok(MyImage { data, path })
    }

   // downsample the image to a 200x200 grid of RGB values
    pub fn to_sampled_grid(&self) -> [[[u8; 3]; 200]; 200] {
        let small_img = self.data.resize_exact(200, 200, FilterType::Triangle);
        let rgb_buffer = small_img.to_rgb8();
        let mut grid = [[[0u8; 3]; 200]; 200];

        for y in 0..200 {
            for x in 0..200 {
                let pixel = rgb_buffer.get_pixel(x, y);
                grid[y as usize][x as usize] = pixel.0;
            }
        }
        grid
    }
}
