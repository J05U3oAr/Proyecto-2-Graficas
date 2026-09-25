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
            "Diorama voxel | flechas rota/inclina | W/S zoom | P captura PNG | Esc salir",
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

        while window.is_open() && !window.is_key_down(Key::Escape) {
            let changed = self.update_camera(&window);
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
                window.set_title(&format!(
                    "Diorama | yaw {:.0}° | distancia {:.0} | flechas rota | W/S zoom | P captura",
                    self.camera.yaw(),
                    self.camera.distance()
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

    fn update_camera(&mut self, window: &Window) -> bool {
        let mut changed = false;
        if window.is_key_pressed(Key::Left, KeyRepeat::Yes)
            || window.is_key_pressed(Key::A, KeyRepeat::Yes)
        {
            self.camera.rotate(-8.);
            changed = true;
        }
        if window.is_key_pressed(Key::Right, KeyRepeat::Yes)
            || window.is_key_pressed(Key::D, KeyRepeat::Yes)
        {
            self.camera.rotate(8.);
            changed = true;
        }
        if window.is_key_pressed(Key::Up, KeyRepeat::Yes) {
            self.camera.tilt(4.);
            changed = true;
        }
        if window.is_key_pressed(Key::Down, KeyRepeat::Yes) {
            self.camera.tilt(-4.);
            changed = true;
        }
        if window.is_key_pressed(Key::W, KeyRepeat::Yes) {
            self.camera.zoom(-2.);
            changed = true;
        }
        if window.is_key_pressed(Key::S, KeyRepeat::Yes) {
            self.camera.zoom(2.);
            changed = true;
        }
        changed
    }
}
