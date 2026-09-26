use std::time::Instant;

use minifb::{Key, KeyRepeat, Scale, Window, WindowOptions};

use crate::camera::Camera;
use crate::image_exporter::ImageExporter;
use crate::renderer::Renderer;
use crate::scene::Scene;

pub struct InteractiveApp {
    scene: Scene,
    renderer: Renderer,
    camera: Camera,
}

impl InteractiveApp {
    pub fn new(scene: Scene, renderer: Renderer, camera: Camera) -> Self {
        Self {
            scene,
            renderer,
            camera,
        }
    }

    pub fn run(mut self) {
        let mut window = Window::new(
            "Diorama voxel | W/A/S/D mover | Space/Ctrl altura | Shift turbo | flechas mirar | P captura | Esc salir",
            self.renderer.width() as usize,
            self.renderer.height() as usize,
            WindowOptions {
                resize: false,
                scale: Scale::X2,
                ..WindowOptions::default()
            },
        )
        .expect("Could not create minifb window");
        let mut pixels = self.renderer.render(&self.scene, self.camera);
        let mut last_frame = Instant::now();

        while window.is_open() && !window.is_key_down(Key::Escape) {
            let now = Instant::now();
            let delta_time = (now - last_frame).as_secs_f32().min(0.1);
            last_frame = now;
            let changed = self.update_camera(&window, delta_time);

            if window.is_key_pressed(Key::P, KeyRepeat::No) {
                match ImageExporter::save(
                    "diorama.png",
                    &pixels,
                    self.renderer.width(),
                    self.renderer.height(),
                ) {
                    Ok(_) => println!("Captura guardada como diorama.png"),
                    Err(error) => eprintln!("No se pudo guardar: {error}"),
                }
            }

            if changed {
                pixels = self.renderer.render(&self.scene, self.camera);
                let position = self.camera.position();
                window.set_title(&format!(
                    "Diorama | pos ({:.1}, {:.1}, {:.1}) | yaw {:.0} | W/A/S/D mover | Space/Ctrl altura | P captura",
                    position.x,
                    position.y,
                    position.z,
                    self.camera.yaw(),
                ));
            }

            window
                .update_with_buffer(
                    &pixels,
                    self.renderer.width() as usize,
                    self.renderer.height() as usize,
                )
                .expect("Could not update window");
        }
    }

    fn update_camera(&mut self, window: &Window, delta_time: f32) -> bool {
        let mut forward = 0.0_f32;
        let mut right = 0.0_f32;
        let mut up = 0.0_f32;
        let mut yaw = 0.0_f32;
        let mut pitch = 0.0_f32;

        if window.is_key_down(Key::W) {
            forward += 1.;
        }
        if window.is_key_down(Key::S) {
            forward -= 1.;
        }
        if window.is_key_down(Key::D) {
            right += 1.;
        }
        if window.is_key_down(Key::A) {
            right -= 1.;
        }
        if window.is_key_down(Key::Space) {
            up += 1.;
        }
        if window.is_key_down(Key::LeftCtrl) || window.is_key_down(Key::RightCtrl) {
            up -= 1.;
        }
        if window.is_key_down(Key::Right) {
            yaw += 1.;
        }
        if window.is_key_down(Key::Left) {
            yaw -= 1.;
        }
        if window.is_key_down(Key::Up) {
            pitch += 1.;
        }
        if window.is_key_down(Key::Down) {
            pitch -= 1.;
        }

        let moving = forward != 0. || right != 0. || up != 0.;
        let looking = yaw != 0. || pitch != 0.;
        if !moving && !looking {
            return false;
        }

        let speed = if window.is_key_down(Key::LeftShift) || window.is_key_down(Key::RightShift) {
            36.
        } else {
            12.
        };
        let input_length = (forward * forward + right * right + up * up).sqrt().max(1.);
        self.camera.translate(
            forward / input_length * speed * delta_time,
            right / input_length * speed * delta_time,
            up / input_length * speed * delta_time,
        );
        self.camera
            .rotate(yaw * 90. * delta_time, pitch * 70. * delta_time);
        true
    }
}
