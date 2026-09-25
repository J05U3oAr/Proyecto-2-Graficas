use std::time::Instant;

use crate::camera::Camera;
use crate::geometry::{Vec3, Vec3Ext};
use crate::raytracer::RayTracer;
use crate::scene::Scene;

pub struct Renderer {
    width: u32,
    height: u32,
    tracer: RayTracer,
}

impl Renderer {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
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
        let started = Instant::now();
        let frame = camera.frame(self.width, self.height);
        let mut pixels = vec![0; (self.width * self.height) as usize];
        let threads = std::thread::available_parallelism()
            .map(|count| count.get())
            .unwrap_or(1)
            .min(self.height as usize)
            .max(1);
        let rows_per_thread = (self.height as usize).div_ceil(threads);

        std::thread::scope(|scope| {
            for (chunk_index, rows) in pixels
                .chunks_mut(rows_per_thread * self.width as usize)
                .enumerate()
            {
                scope.spawn(move || {
                    for (local_y, row) in rows.chunks_mut(self.width as usize).enumerate() {
                        let y = (chunk_index * rows_per_thread + local_y) as u32;
                        for (x, pixel) in row.iter_mut().enumerate() {
                            let ray = frame.ray(x as u32, y, self.width, self.height);
                            *pixel = to_pixel(self.tracer.trace(scene, ray));
                        }
                    }
                });
            }
        });

        println!(
            "Render {}x{} | {} cubos | {} hilos | {:.2?}",
            self.width,
            self.height,
            scene.cube_count(),
            threads,
            started.elapsed()
        );
        pixels
    }
}

fn to_pixel(color: Vec3) -> u32 {
    let color = color.clamp_rgb();
    let red = (color.x.powf(1. / 2.2) * 255.) as u32;
    let green = (color.y.powf(1. / 2.2) * 255.) as u32;
    let blue = (color.z.powf(1. / 2.2) * 255.) as u32;
    (red << 16) | (green << 8) | blue
}
