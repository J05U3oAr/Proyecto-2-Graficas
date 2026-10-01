use std::time::{Duration, Instant};

use minifb::{CursorStyle, Key, KeyRepeat, MouseButton, MouseMode, Scale, Window, WindowOptions};

use crate::camera::{Camera, CameraFrame};
use crate::geometry::Vec3;
use crate::image_exporter::ImageExporter;
use crate::raytracer::TimeOfDay;
use crate::renderer::Renderer;
use crate::scene::Scene;
use crate::scene_builder::{House, SceneBuilder};

pub struct InteractiveApp {
    scene: Option<Scene>,
    renderer: Renderer,
    camera: Camera,
}

impl InteractiveApp {
    pub fn menu_preview(width: u32, height: u32, house: House) -> Vec<u32> {
        let scene = SceneBuilder::build_planet_preview(house);
        let mut renderer = Renderer::new(width, height, 4);
        renderer.set_space_background();
        let rendered = renderer.render(&scene, menu_camera(125.));
        house_menu_pixels(&rendered, width, height, house, false)
    }

    pub fn new(renderer: Renderer, camera: Camera) -> Self {
        Self {
            scene: None,
            renderer,
            camera,
        }
    }

    pub fn run(mut self) {
        let mut window = Window::new(
            "Diorama voxel | W/A/S/D mover | flechas mirar | N dia/noche | P captura | Esc salir",
            self.renderer.width() as usize,
            self.renderer.height() as usize,
            WindowOptions {
                resize: false,
                // Present pixels at their native size. Scaling a low-resolution
                // buffer was the main source of the soft, blocky appearance.
                scale: Scale::X1,
                ..WindowOptions::default()
            },
        )
        .expect("Could not create minifb window");

        let Some(house) = self.show_house_menu(&mut window) else {
            return;
        };
        window.set_title(&format!(
            "Diorama voxel | Cargando casa {}...",
            house.label()
        ));
        window.update();
        self.scene = Some(SceneBuilder::build(house));
        if house == House::Elru {
            // Start from the lake so the modern front facade is the first view.
            self.camera = Camera::from_orbit(Vec3::new(0., 10., -1.), 110., 14., 64., 52.);
        }

        if !self.show_environment_menu(&mut window, house) {
            return;
        }

        let mut pixels = self.renderer.render(
            self.scene.as_ref().expect("House scene must be loaded"),
            self.camera,
        );
        let mut display_pixels = pixels.clone();
        let mut last_frame = Instant::now();
        let mut last_camera_change = Instant::now();
        let mut night_animation_started = Instant::now();
        let mut quality_pending = false;

        while window.is_open() && !window.is_key_down(Key::Escape) {
            let now = Instant::now();
            let delta_time = (now - last_frame).as_secs_f32().min(0.1);
            last_frame = now;
            let changed = self.update_camera(&window, delta_time);

            if window.is_key_pressed(Key::N, KeyRepeat::No) {
                let time = self.renderer.toggle_time_of_day();
                pixels = self.renderer.render(
                    self.scene.as_ref().expect("House scene must be loaded"),
                    self.camera,
                );
                night_animation_started = now;
                quality_pending = false;
                println!("Modo de entorno: {}", time.label());
                let position = self.camera.position();
                window.set_title(&format!(
                    "Diorama {} | modo {} | pos ({:.1}, {:.1}, {:.1}) | N cambiar | P captura HD",
                    house.label(),
                    time.label(),
                    position.x,
                    position.y,
                    position.z,
                ));
            }

            if changed {
                pixels = self.renderer.render_preview(
                    self.scene.as_ref().expect("House scene must be loaded"),
                    self.camera,
                );
                last_camera_change = now;
                quality_pending = true;
                let position = self.camera.position();
                window.set_title(&format!(
                    "Diorama {} | {} | vista previa | pos ({:.1}, {:.1}, {:.1}) | yaw {:.0} | N cambiar",
                    house.label(),
                    self.renderer.time_of_day().label(),
                    position.x,
                    position.y,
                    position.z,
                    self.camera.yaw(),
                ));
            } else if quality_pending
                && now.duration_since(last_camera_change) >= Duration::from_millis(160)
            {
                pixels = self.renderer.render(
                    self.scene.as_ref().expect("House scene must be loaded"),
                    self.camera,
                );
                quality_pending = false;
                let position = self.camera.position();
                window.set_title(&format!(
                    "Diorama {} | {} | calidad alta | pos ({:.1}, {:.1}, {:.1}) | yaw {:.0} | N cambiar",
                    house.label(),
                    self.renderer.time_of_day().label(),
                    position.x,
                    position.y,
                    position.z,
                    self.camera.yaw(),
                ));
            }

            let capture_requested = window.is_key_pressed(Key::P, KeyRepeat::No);
            if capture_requested && quality_pending {
                pixels = self.renderer.render(
                    self.scene.as_ref().expect("House scene must be loaded"),
                    self.camera,
                );
                quality_pending = false;
            }

            display_pixels.clone_from(&pixels);
            if self.renderer.time_of_day() == TimeOfDay::Night {
                draw_animated_shooting_stars(
                    &mut display_pixels,
                    self.renderer.width(),
                    self.renderer.height(),
                    now.duration_since(night_animation_started).as_secs_f32(),
                    self.scene.as_ref().expect("House scene must be loaded"),
                    self.camera,
                );
            }

            if capture_requested {
                match ImageExporter::save(
                    "diorama.png",
                    &display_pixels,
                    self.renderer.width(),
                    self.renderer.height(),
                ) {
                    Ok(_) => println!("Captura HD guardada como diorama.png"),
                    Err(error) => eprintln!("No se pudo guardar: {error}"),
                }
            }

            window
                .update_with_buffer(
                    &display_pixels,
                    self.renderer.width() as usize,
                    self.renderer.height() as usize,
                )
                .expect("Could not update window");
        }
    }

    fn show_house_menu(&self, window: &mut Window) -> Option<House> {
        let mut house = House::ALL[0];
        let mut planet_scene = SceneBuilder::build_planet_preview(house);
        let mut angle = 125.;
        let mut planet_camera = menu_camera(angle);
        let mut menu_renderer = Renderer::new(self.renderer.width(), self.renderer.height(), 2);
        menu_renderer.set_space_background();
        let mut planet_pixels = menu_renderer.render(&planet_scene, planet_camera);
        let mut planet_frame = planet_camera.frame(self.renderer.width(), self.renderer.height());
        let mut last_orbit = Instant::now();
        let mut auto_rotate = true;
        let mut was_hovered = false;
        let mut mouse_was_down = false;
        window.set_title("Diorama voxel | A/D cambiar mundo | Clic para entrar");

        while window.is_open() && !window.is_key_down(Key::Escape) {
            let change = if window.is_key_pressed(Key::A, KeyRepeat::No) {
                -1
            } else if window.is_key_pressed(Key::D, KeyRepeat::No) {
                1
            } else {
                0
            };
            if change != 0 {
                let index = House::ALL
                    .iter()
                    .position(|candidate| *candidate == house)
                    .expect("Current house must be selectable")
                    as isize;
                let next = (index + change).rem_euclid(House::ALL.len() as isize) as usize;
                house = House::ALL[next];
                planet_scene = SceneBuilder::build_planet_preview(house);
                planet_pixels = menu_renderer.render(&planet_scene, planet_camera);
                planet_frame = planet_camera.frame(self.renderer.width(), self.renderer.height());
                was_hovered = false;
            }

            let hovered = window
                .get_mouse_pos(MouseMode::Discard)
                .map(|(x, y)| {
                    let ray = planet_frame.ray(
                        x as u32,
                        y as u32,
                        self.renderer.width(),
                        self.renderer.height(),
                    );
                    planet_scene.hit(ray, f32::INFINITY).is_some()
                })
                .unwrap_or(false);
            let mouse_down = window.get_mouse_down(MouseButton::Left);
            let planet_clicked = hovered && mouse_down && !mouse_was_down;
            mouse_was_down = mouse_down;
            window.set_cursor_style(if hovered {
                CursorStyle::OpenHand
            } else {
                CursorStyle::Arrow
            });

            if planet_clicked || window.is_key_pressed(Key::Enter, KeyRepeat::No) {
                window.set_cursor_style(CursorStyle::Arrow);
                println!("Casa seleccionada: {}", house.label());
                return Some(house);
            }

            if window.is_key_pressed(Key::Space, KeyRepeat::No) {
                auto_rotate = !auto_rotate;
            }
            let orbit_input = if window.is_key_down(Key::Left) {
                -1.
            } else {
                0.
            } + if window.is_key_down(Key::Right) {
                1.
            } else {
                0.
            };
            let elapsed = last_orbit.elapsed().as_secs_f32();
            if elapsed >= 0.12 {
                let speed = if orbit_input != 0. {
                    orbit_input * 35.
                } else if auto_rotate && !hovered {
                    6.
                } else {
                    0.
                };
                if speed != 0. {
                    angle += speed * elapsed.min(0.25);
                    planet_camera = menu_camera(angle);
                    planet_pixels = menu_renderer.render_preview(&planet_scene, planet_camera);
                    planet_frame =
                        planet_camera.frame(self.renderer.width(), self.renderer.height());
                } else if hovered && !was_hovered {
                    planet_pixels = menu_renderer.render(&planet_scene, planet_camera);
                }
                last_orbit = Instant::now();
                was_hovered = hovered;
            }
            let pixels = house_menu_pixels(
                &planet_pixels,
                self.renderer.width(),
                self.renderer.height(),
                house,
                hovered,
            );
            window
                .update_with_buffer(
                    &pixels,
                    self.renderer.width() as usize,
                    self.renderer.height() as usize,
                )
                .expect("Could not update house menu");
            std::thread::sleep(Duration::from_millis(16));
        }
        None
    }

    fn show_environment_menu(&mut self, window: &mut Window, house: House) -> bool {
        let mut selection = self.renderer.time_of_day();
        window.set_title(&format!(
            "Diorama voxel | Casa {} | Flechas elegir entorno | Enter comenzar",
            house.label()
        ));

        while window.is_open() && !window.is_key_down(Key::Escape) {
            if window.is_key_pressed(Key::Left, KeyRepeat::No)
                || window.is_key_pressed(Key::D, KeyRepeat::No)
            {
                selection = TimeOfDay::Day;
            }
            if window.is_key_pressed(Key::Right, KeyRepeat::No)
                || window.is_key_pressed(Key::N, KeyRepeat::No)
            {
                selection = TimeOfDay::Night;
            }
            if window.is_key_pressed(Key::Enter, KeyRepeat::No) {
                self.renderer.set_time_of_day(selection);
                println!("Entorno seleccionado: {}", selection.label());
                return true;
            }

            let pixels = menu_pixels(self.renderer.width(), self.renderer.height(), selection);
            window
                .update_with_buffer(
                    &pixels,
                    self.renderer.width() as usize,
                    self.renderer.height() as usize,
                )
                .expect("Could not update main menu");
            std::thread::sleep(Duration::from_millis(16));
        }
        false
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

#[derive(Clone, Copy)]
struct ShootingStarPattern {
    delay: f32,
    period: f32,
    duration: f32,
    start: (f32, f32),
    end: (f32, f32),
}

fn draw_animated_shooting_stars(
    pixels: &mut [u32],
    width: u32,
    height: u32,
    elapsed: f32,
    scene: &Scene,
    camera: Camera,
) {
    let patterns = [
        ShootingStarPattern {
            delay: 1.4,
            period: 7.2,
            duration: 1.05,
            start: (0.12, 0.15),
            end: (0.64, 0.38),
        },
        ShootingStarPattern {
            delay: 4.1,
            period: 9.4,
            duration: 0.90,
            start: (0.82, 0.11),
            end: (0.39, 0.30),
        },
        ShootingStarPattern {
            delay: 7.0,
            period: 12.1,
            duration: 1.20,
            start: (0.25, 0.08),
            end: (0.73, 0.27),
        },
    ];
    let frame = camera.frame(width, height);

    for pattern in patterns {
        let Some(progress) =
            shooting_star_progress(elapsed, pattern.delay, pattern.period, pattern.duration)
        else {
            continue;
        };
        let fade = (progress * std::f32::consts::PI).sin().sqrt();

        // Separated points form a short-lived particle trail instead of a
        // solid stripe. The moving head remains recognisable as a star.
        for particle in (1..=7).rev() {
            let particle_progress = progress - particle as f32 * 0.018;
            if particle_progress < 0. {
                continue;
            }
            let (x, y) = shooting_star_position(pattern, particle_progress);
            let strength = (1. - particle as f32 / 8.).powf(1.4) * 0.58 * fade;
            draw_sky_particle(
                pixels,
                width,
                height,
                frame,
                scene,
                x,
                y,
                1,
                (205, 220, 255),
                strength,
            );
        }

        let (head_x, head_y) = shooting_star_position(pattern, progress);
        draw_sky_particle(
            pixels,
            width,
            height,
            frame,
            scene,
            head_x,
            head_y,
            3,
            (130, 175, 255),
            0.18 * fade,
        );
        draw_sky_particle(
            pixels,
            width,
            height,
            frame,
            scene,
            head_x,
            head_y,
            1,
            (255, 248, 220),
            0.96 * fade,
        );
    }
}

fn shooting_star_progress(elapsed: f32, delay: f32, period: f32, duration: f32) -> Option<f32> {
    if elapsed < delay {
        return None;
    }
    let active_time = (elapsed - delay) % period;
    (active_time <= duration).then_some(active_time / duration)
}

fn shooting_star_position(pattern: ShootingStarPattern, progress: f32) -> (f32, f32) {
    let x = pattern.start.0 + (pattern.end.0 - pattern.start.0) * progress;
    let arc = progress * (1. - progress) * 0.035;
    let y = pattern.start.1 + (pattern.end.1 - pattern.start.1) * progress + arc;
    (x, y)
}

#[allow(clippy::too_many_arguments)]
fn draw_sky_particle(
    pixels: &mut [u32],
    width: u32,
    height: u32,
    frame: CameraFrame,
    scene: &Scene,
    normalized_x: f32,
    normalized_y: f32,
    radius: i32,
    color: (u8, u8, u8),
    opacity: f32,
) {
    let center_x = (normalized_x * width as f32).round() as i32;
    let center_y = (normalized_y * height as f32).round() as i32;
    let radius_squared = (radius * radius).max(1) as f32;

    for offset_y in -radius..=radius {
        for offset_x in -radius..=radius {
            let distance_squared = (offset_x * offset_x + offset_y * offset_y) as f32;
            if distance_squared > radius_squared {
                continue;
            }
            let x = center_x + offset_x;
            let y = center_y + offset_y;
            if x < 0 || y < 0 || x >= width as i32 || y >= height as i32 {
                continue;
            }

            let ray = frame.ray(x as u32, y as u32, width, height);
            if scene.hit(ray, f32::INFINITY).is_some() {
                continue;
            }
            let edge_fade = 1. - distance_squared / (radius_squared + 0.5);
            let index = y as usize * width as usize + x as usize;
            pixels[index] = blend_rgb(pixels[index], color, opacity * edge_fade);
        }
    }
}

fn blend_rgb(background: u32, foreground: (u8, u8, u8), opacity: f32) -> u32 {
    let opacity = opacity.clamp(0., 1.);
    let background_red = ((background >> 16) & 255) as f32;
    let background_green = ((background >> 8) & 255) as f32;
    let background_blue = (background & 255) as f32;
    let blend = |background: f32, foreground: u8| {
        (background * (1. - opacity) + foreground as f32 * opacity).round() as u32
    };
    (blend(background_red, foreground.0) << 16)
        | (blend(background_green, foreground.1) << 8)
        | blend(background_blue, foreground.2)
}

fn house_menu_pixels(
    rendered_planet: &[u32],
    width: u32,
    height: u32,
    house: House,
    hovered: bool,
) -> Vec<u32> {
    let width = width as usize;
    let height = height as usize;
    let mut pixels = rendered_planet.to_vec();
    if pixels.len() != width * height {
        pixels.resize(width * height, 0);
    }

    let scale = (width / 240).clamp(1, 4);
    draw_centered_text(
        &mut pixels,
        width,
        height,
        height / 16,
        "MUNDOS",
        scale,
        rgb(255, 255, 255),
    );
    draw_centered_text(
        &mut pixels,
        width,
        height,
        height / 16 + scale * 9,
        "A Y D CAMBIAN EL MUNDO",
        (scale - 1).max(1),
        rgb(190, 211, 241),
    );

    let card_width = (width / 3).max(30);
    let card_height = (height / 12).max(24);
    let card_x = width.saturating_sub(card_width) / 2;
    let card_y = height * 4 / 5;
    draw_menu_card(
        &mut pixels,
        width,
        height,
        card_x,
        card_y,
        card_width,
        card_height,
        house.label(),
        hovered,
        scale,
    );
    draw_centered_text(
        &mut pixels,
        width,
        height,
        (card_y + card_height + scale * 5).min(height.saturating_sub(scale * 7)),
        if hovered {
            "HAZ CLIC PARA VISITAR"
        } else {
            "CLIC EN EL MUNDO PARA ENTRAR"
        },
        (scale - 1).max(1),
        rgb(224, 232, 247),
    );
    draw_centered_text(
        &mut pixels,
        width,
        height,
        height.saturating_sub(scale * 8),
        "A D MUNDO   FLECHAS GIRAR   ESPACIO PAUSAR",
        (scale - 1).max(1),
        rgb(125, 151, 184),
    );
    pixels
}

fn menu_camera(angle: f32) -> Camera {
    Camera::from_orbit(Vec3::new(0.5, 0., 0.5), angle, 27., 72., 40.)
}

fn menu_pixels(width: u32, height: u32, selection: TimeOfDay) -> Vec<u32> {
    let width = width as usize;
    let height = height as usize;
    let mut pixels = vec![0; width * height];

    for y in 0..height {
        let t = y as f32 / height.max(1) as f32;
        let (top, bottom) = match selection {
            TimeOfDay::Day => ((31, 92, 174), (157, 211, 245)),
            TimeOfDay::Night => ((3, 7, 27), (28, 43, 83)),
        };
        let color = rgb(
            lerp(top.0, bottom.0, t),
            lerp(top.1, bottom.1, t),
            lerp(top.2, bottom.2, t),
        );
        pixels[y * width..(y + 1) * width].fill(color);
    }

    match selection {
        TimeOfDay::Day => {
            draw_square(
                &mut pixels,
                width,
                height,
                width * 5 / 6,
                height / 5,
                30,
                rgb(255, 224, 135),
            );
            for &(x, y, radius) in &[
                (width / 8, height / 5, 28),
                (width / 8 + 34, height / 5 + 5, 35),
                (width / 8 + 75, height / 5, 25),
                (width * 2 / 3, height / 3, 24),
                (width * 2 / 3 + 30, height / 3 + 4, 31),
            ] {
                draw_square(&mut pixels, width, height, x, y, radius, rgb(225, 239, 249));
            }
        }
        TimeOfDay::Night => {
            for index in 0..90usize {
                let x = (index * 83 + index * index * 17 + 31) % width.max(1);
                let y = (index * 47 + index * index * 11 + 19) % (height * 2 / 3).max(1);
                let color = if index % 7 == 0 {
                    rgb(150, 188, 255)
                } else {
                    rgb(244, 238, 211)
                };
                draw_square(
                    &mut pixels,
                    width,
                    height,
                    x,
                    y,
                    if index % 11 == 0 { 2 } else { 1 },
                    color,
                );
            }
            draw_square(
                &mut pixels,
                width,
                height,
                width * 5 / 6,
                height / 5,
                34,
                rgb(220, 229, 255),
            );
            draw_square(
                &mut pixels,
                width,
                height,
                width * 5 / 6 + 14,
                (height / 5).saturating_sub(8),
                31,
                rgb(10, 17, 46),
            );
        }
    }

    let scale = (width / 180).clamp(1, 4);
    draw_centered_text(
        &mut pixels,
        width,
        height,
        height / 8,
        "DIORAMA VOXEL",
        scale,
        rgb(255, 255, 255),
    );
    draw_centered_text(
        &mut pixels,
        width,
        height,
        height / 8 + scale * 11,
        "SELECCIONA EL ENTORNO",
        (scale - 1).max(1),
        rgb(218, 230, 249),
    );

    let card_y = height * 3 / 5;
    let card_height = (height / 5).max(38);
    let gap = (width / 30).max(8);
    let card_width = (width / 3).max(10);
    let left_x = width / 2 - card_width - gap / 2;
    let right_x = width / 2 + gap / 2;
    draw_menu_card(
        &mut pixels,
        width,
        height,
        left_x,
        card_y,
        card_width,
        card_height,
        "DIA",
        selection == TimeOfDay::Day,
        scale,
    );
    draw_menu_card(
        &mut pixels,
        width,
        height,
        right_x,
        card_y,
        card_width,
        card_height,
        "NOCHE",
        selection == TimeOfDay::Night,
        scale,
    );
    draw_centered_text(
        &mut pixels,
        width,
        height,
        (card_y + card_height + scale * 6).min(height.saturating_sub(scale * 7)),
        "FLECHAS PARA ELEGIR  ENTER PARA COMENZAR",
        (scale - 1).max(1),
        rgb(230, 236, 248),
    );
    pixels
}

#[allow(clippy::too_many_arguments)]
fn draw_menu_card(
    pixels: &mut [u32],
    width: usize,
    height: usize,
    x: usize,
    y: usize,
    card_width: usize,
    card_height: usize,
    label: &str,
    selected: bool,
    scale: usize,
) {
    let border = if selected {
        rgb(255, 213, 94)
    } else {
        rgb(117, 139, 177)
    };
    let fill = if selected {
        rgb(35, 52, 83)
    } else {
        rgb(20, 31, 58)
    };
    fill_rect(pixels, width, height, x, y, card_width, card_height, border);
    let inset = if selected { 4 } else { 2 };
    fill_rect(
        pixels,
        width,
        height,
        x + inset,
        y + inset,
        card_width.saturating_sub(inset * 2),
        card_height.saturating_sub(inset * 2),
        fill,
    );
    let text_width = label.chars().count() * 6 * scale - scale;
    draw_text(
        pixels,
        width,
        height,
        x + card_width.saturating_sub(text_width) / 2,
        y + card_height.saturating_sub(7 * scale) / 2,
        label,
        scale,
        if selected {
            rgb(255, 230, 151)
        } else {
            rgb(210, 220, 240)
        },
    );
}

#[allow(clippy::too_many_arguments)]
fn fill_rect(
    pixels: &mut [u32],
    width: usize,
    height: usize,
    x: usize,
    y: usize,
    rect_width: usize,
    rect_height: usize,
    color: u32,
) {
    for row in y..(y + rect_height).min(height) {
        let start = row * width + x.min(width);
        let end = row * width + (x + rect_width).min(width);
        pixels[start..end].fill(color);
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_square(
    pixels: &mut [u32],
    width: usize,
    height: usize,
    center_x: usize,
    center_y: usize,
    radius: usize,
    color: u32,
) {
    let x = center_x.saturating_sub(radius);
    let y = center_y.saturating_sub(radius);
    fill_rect(
        pixels,
        width,
        height,
        x,
        y,
        (center_x + radius + 1).min(width).saturating_sub(x),
        (center_y + radius + 1).min(height).saturating_sub(y),
        color,
    );
}

#[allow(clippy::too_many_arguments)]
fn draw_centered_text(
    pixels: &mut [u32],
    width: usize,
    height: usize,
    y: usize,
    text: &str,
    scale: usize,
    color: u32,
) {
    let text_width = text.chars().count() * 6 * scale - scale;
    draw_text(
        pixels,
        width,
        height,
        width.saturating_sub(text_width) / 2,
        y,
        text,
        scale,
        color,
    );
}

#[allow(clippy::too_many_arguments)]
fn draw_text(
    pixels: &mut [u32],
    width: usize,
    height: usize,
    x: usize,
    y: usize,
    text: &str,
    scale: usize,
    color: u32,
) {
    for (character_index, character) in text.chars().enumerate() {
        for (row, bits) in glyph(character.to_ascii_uppercase()).iter().enumerate() {
            for column in 0..5 {
                if bits & (1 << (4 - column)) != 0 {
                    fill_rect(
                        pixels,
                        width,
                        height,
                        x + (character_index * 6 + column) * scale,
                        y + row * scale,
                        scale,
                        scale,
                        color,
                    );
                }
            }
        }
    }
}

fn glyph(character: char) -> [u8; 7] {
    match character {
        'A' => [14, 17, 17, 31, 17, 17, 17],
        'B' => [30, 17, 17, 30, 17, 17, 30],
        'C' => [14, 17, 16, 16, 16, 17, 14],
        'D' => [30, 17, 17, 17, 17, 17, 30],
        'E' => [31, 16, 16, 30, 16, 16, 31],
        'F' => [31, 16, 16, 30, 16, 16, 16],
        'G' => [14, 17, 16, 23, 17, 17, 14],
        'H' => [17, 17, 17, 31, 17, 17, 17],
        'I' => [14, 4, 4, 4, 4, 4, 14],
        'J' => [7, 2, 2, 2, 18, 18, 12],
        'K' => [17, 18, 20, 24, 20, 18, 17],
        'L' => [16, 16, 16, 16, 16, 16, 31],
        'M' => [17, 27, 21, 21, 17, 17, 17],
        'N' => [17, 25, 21, 19, 17, 17, 17],
        'O' => [14, 17, 17, 17, 17, 17, 14],
        'P' => [30, 17, 17, 30, 16, 16, 16],
        'Q' => [14, 17, 17, 17, 21, 18, 13],
        'R' => [30, 17, 17, 30, 20, 18, 17],
        'S' => [15, 16, 16, 14, 1, 1, 30],
        'T' => [31, 4, 4, 4, 4, 4, 4],
        'U' => [17, 17, 17, 17, 17, 17, 14],
        'V' => [17, 17, 17, 17, 17, 10, 4],
        'W' => [17, 17, 17, 21, 21, 21, 10],
        'X' => [17, 17, 10, 4, 10, 17, 17],
        'Y' => [17, 17, 10, 4, 4, 4, 4],
        'Z' => [31, 1, 2, 4, 8, 16, 31],
        '0' => [14, 17, 19, 21, 25, 17, 14],
        '1' => [4, 12, 4, 4, 4, 4, 14],
        '2' => [14, 17, 1, 2, 4, 8, 31],
        '3' => [30, 1, 1, 14, 1, 1, 30],
        '4' => [2, 6, 10, 18, 31, 2, 2],
        '5' => [31, 16, 16, 30, 1, 1, 30],
        '6' => [14, 16, 16, 30, 17, 17, 14],
        '7' => [31, 1, 2, 4, 8, 8, 8],
        '8' => [14, 17, 17, 14, 17, 17, 14],
        '9' => [14, 17, 17, 15, 1, 1, 14],
        _ => [0; 7],
    }
}

fn lerp(from: u8, to: u8, amount: f32) -> u8 {
    (from as f32 + (to as f32 - from as f32) * amount).round() as u8
}

fn rgb(red: u8, green: u8, blue: u8) -> u32 {
    ((red as u32) << 16) | ((green as u32) << 8) | blue as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_renders_both_environments_at_window_size() {
        let day = menu_pixels(720, 480, TimeOfDay::Day);
        let night = menu_pixels(720, 480, TimeOfDay::Night);
        assert_eq!(day.len(), 720 * 480);
        assert_eq!(night.len(), 720 * 480);
        assert_ne!(day, night);
    }

    #[test]
    fn house_menu_renders_ve7_at_window_size() {
        let rendered_planet = vec![0; 720 * 480];
        assert_eq!(
            house_menu_pixels(&rendered_planet, 720, 480, House::Ve7, false).len(),
            720 * 480
        );
        assert_ne!(glyph('7'), [0; 7]);
    }

    #[test]
    fn center_click_ray_hits_the_3d_ve7_planet() {
        let scene = SceneBuilder::build_planet_preview(House::Ve7);
        let camera = menu_camera(125.);
        let ray = camera.frame(720, 480).ray(360, 240, 720, 480);
        assert!(scene.hit(ray, f32::INFINITY).is_some());
        for (x, y) in [(0, 0), (719, 0), (0, 479), (719, 479), (360, 420)] {
            let outside = camera.frame(720, 480).ray(x, y, 720, 480);
            assert!(scene.hit(outside, f32::INFINITY).is_none());
        }
        assert!(scene.cube_count() > 1_000);
    }

    #[test]
    fn elru_is_a_clickable_cubic_world_with_a_lake() {
        let scene = SceneBuilder::build(House::Elru);
        let camera = Camera::from_orbit(Vec3::new(0., 2., 0.), 90., 18., 65., 52.);
        let center_ray = camera.frame(720, 480).ray(360, 240, 720, 480);
        assert!(scene.hit(center_ray, f32::INFINITY).is_some());
        assert!(scene.cube_count() > 10_000);
    }

    #[test]
    fn menu_supports_minimum_cli_resolution() {
        assert_eq!(menu_pixels(32, 32, TimeOfDay::Day).len(), 32 * 32);
        assert_eq!(menu_pixels(32, 32, TimeOfDay::Night).len(), 32 * 32);
    }

    #[test]
    fn shooting_stars_appear_briefly_instead_of_staying_as_lines() {
        assert_eq!(shooting_star_progress(0.5, 1.0, 7.0, 1.0), None);
        assert!(shooting_star_progress(1.5, 1.0, 7.0, 1.0).is_some());
        assert_eq!(shooting_star_progress(2.5, 1.0, 7.0, 1.0), None);
    }
}
