use image::{Rgb, RgbImage};

pub struct ImageExporter;

impl ImageExporter {
    pub fn save(path: &str, pixels: &[u32], width: u32, height: u32) -> image::ImageResult<()> {
        let mut image = RgbImage::new(width, height);
        for y in 0..height {
            for x in 0..width {
                let pixel = pixels[(y * width + x) as usize];
                image.put_pixel(
                    x,
                    y,
                    Rgb([
                        ((pixel >> 16) & 255) as u8,
                        ((pixel >> 8) & 255) as u8,
                        (pixel & 255) as u8,
                    ]),
                );
            }
        }
        image.save(path)
    }
}
