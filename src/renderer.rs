use std::time::Instant;

use crate::camera::Camera;
use crate::geometry::{Vec3, Vec3Ext};
use crate::raytracer::RayTracer;
use crate::scene::Scene;

pub struct Renderer {
    width: u32,
    height: u32,
    samples_per_pixel: u32,
    tracer: RayTracer,
}

impl Renderer {
    pub fn new(width: u32, height: u32, samples_per_pixel: u32) -> Self {
        Self {
            width,
            height,
            samples_per_pixel: samples_per_pixel.clamp(1, 16),
            tracer: RayTracer::default(),
        }
    }
    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn render(&self, scene: &Scene, camera: Camera) -> Vec<u32> {
        self.render_at(
            scene,
            camera,
            self.width,
            self.height,
            self.samples_per_pixel,
            0.18,
        )
    }

    /// A reduced preview keeps spectator movement responsive. The app replaces
    /// it with the refined image as soon as the camera stops.
    pub fn render_preview(&self, scene: &Scene, camera: Camera) -> Vec<u32> {
        let preview_width = (self.width / 2).max(1);
        let preview_height = (self.height / 2).max(1);
        let preview = self.render_at(scene, camera, preview_width, preview_height, 1, 0.04);
        scale_bilinear(
            &preview,
            preview_width,
            preview_height,
            self.width,
            self.height,
        )
    }

    fn render_at(
        &self,
        scene: &Scene,
        camera: Camera,
        width: u32,
        height: u32,
        samples_per_pixel: u32,
        sharpness: f32,
    ) -> Vec<u32> {
        let started = Instant::now();
        let frame = camera.frame(width, height);
        let sample_offsets: Vec<(f32, f32)> = (0..samples_per_pixel)
            .map(|sample| sample_offset(sample, samples_per_pixel))
            .collect();
        let mut colors = vec![Vec3::zeros(); (width * height) as usize];
        let threads = std::thread::available_parallelism()
            .map(|count| count.get())
            .unwrap_or(1)
            .min(height as usize)
            .max(1);
        let rows_per_thread = (height as usize).div_ceil(threads);

        std::thread::scope(|scope| {
            let sample_offsets = &sample_offsets;
            for (chunk_index, rows) in colors
                .chunks_mut(rows_per_thread * width as usize)
                .enumerate()
            {
                scope.spawn(move || {
                    for (local_y, row) in rows.chunks_mut(width as usize).enumerate() {
                        let y = (chunk_index * rows_per_thread + local_y) as u32;
                        for (x, color) in row.iter_mut().enumerate() {
                            let mut accumulated = Vec3::zeros();
                            for &(offset_x, offset_y) in sample_offsets {
                                let ray = if samples_per_pixel == 1 {
                                    frame.ray(x as u32, y, width, height)
                                } else {
                                    frame.ray_sample(x as u32, y, width, height, offset_x, offset_y)
                                };
                                accumulated += self.tracer.trace(scene, ray);
                            }
                            *color = accumulated / samples_per_pixel as f32;
                        }
                    }
                });
            }
        });

        let pixels = sharpen_and_pack(&colors, width, height, sharpness);

        println!(
            "Render {}x{} | {} muestras/pixel | {} cubos | {} hilos | {:.2?}",
            width,
            height,
            samples_per_pixel,
            scene.cube_count(),
            threads,
            started.elapsed()
        );
        pixels
    }
}

fn scale_bilinear(
    pixels: &[u32],
    source_width: u32,
    source_height: u32,
    width: u32,
    height: u32,
) -> Vec<u32> {
    if source_width == width && source_height == height {
        return pixels.to_vec();
    }

    let source_width = source_width as usize;
    let source_height = source_height as usize;
    let width = width as usize;
    let height = height as usize;
    let mut scaled = vec![0; width * height];

    for y in 0..height {
        let source_y = ((y as f32 + 0.5) * source_height as f32 / height as f32 - 0.5)
            .clamp(0., (source_height - 1) as f32);
        let y0 = source_y.floor() as usize;
        let y1 = (y0 + 1).min(source_height - 1);
        let fy = source_y - y0 as f32;

        for x in 0..width {
            let source_x = ((x as f32 + 0.5) * source_width as f32 / width as f32 - 0.5)
                .clamp(0., (source_width - 1) as f32);
            let x0 = source_x.floor() as usize;
            let x1 = (x0 + 1).min(source_width - 1);
            let fx = source_x - x0 as f32;
            let top_left = pixels[y0 * source_width + x0];
            let top_right = pixels[y0 * source_width + x1];
            let bottom_left = pixels[y1 * source_width + x0];
            let bottom_right = pixels[y1 * source_width + x1];

            let mut channels = [0.; 3];
            for (channel, shift) in channels.iter_mut().zip([16, 8, 0]) {
                let top = channel_value(top_left, shift) * (1. - fx)
                    + channel_value(top_right, shift) * fx;
                let bottom = channel_value(bottom_left, shift) * (1. - fx)
                    + channel_value(bottom_right, shift) * fx;
                *channel = top * (1. - fy) + bottom * fy;
            }
            scaled[y * width + x] = ((channels[0].round() as u32) << 16)
                | ((channels[1].round() as u32) << 8)
                | channels[2].round() as u32;
        }
    }
    scaled
}

fn channel_value(pixel: u32, shift: u32) -> f32 {
    ((pixel >> shift) & 255) as f32
}

fn sample_offset(sample: u32, sample_count: u32) -> (f32, f32) {
    if sample_count == 1 {
        return (0.5, 0.5);
    }

    // A deterministic low-discrepancy sequence distributes arbitrary sample
    // counts evenly without temporal noise or a random-number dependency.
    (
        radical_inverse(sample + 1, 2),
        radical_inverse(sample + 1, 3),
    )
}

fn radical_inverse(mut value: u32, base: u32) -> f32 {
    let inverse_base = 1. / base as f32;
    let mut factor = inverse_base;
    let mut result = 0.;
    while value > 0 {
        result += (value % base) as f32 * factor;
        value /= base;
        factor *= inverse_base;
    }
    result
}

fn sharpen_and_pack(colors: &[Vec3], width: u32, height: u32, amount: f32) -> Vec<u32> {
    let mut pixels = Vec::with_capacity(colors.len());
    let width = width as usize;
    let height = height as usize;

    for y in 0..height {
        for x in 0..width {
            let index = y * width + x;
            let color = if x == 0 || y == 0 || x + 1 == width || y + 1 == height {
                colors[index]
            } else {
                let neighbors = (colors[index - 1]
                    + colors[index + 1]
                    + colors[index - width]
                    + colors[index + width])
                    * 0.25;
                colors[index] + (colors[index] - neighbors) * amount
            };
            pixels.push(to_pixel(color));
        }
    }
    pixels
}

fn to_pixel(color: Vec3) -> u32 {
    let color = color.clamp_rgb();
    let red = (color.x.powf(1. / 2.2) * 255.).round() as u32;
    let green = (color.y.powf(1. / 2.2) * 255.).round() as u32;
    let blue = (color.z.powf(1. / 2.2) * 255.).round() as u32;
    (red << 16) | (green << 8) | blue
}
