use std::env;

use crate::camera::Camera;
use crate::geometry::Vec3;
use crate::raytracer::TimeOfDay;
use crate::scene_builder::House;

pub struct AppConfig {
    pub width: u32,
    pub height: u32,
    pub samples_per_pixel: u32,
    pub camera: Camera,
    pub output_path: Option<String>,
    pub time_of_day: TimeOfDay,
    pub help_requested: bool,
    pub menu_preview: bool,
    pub house: House,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let args: Vec<String> = env::args().collect();
        let elru = value_after(&args, "--world").as_deref() == Some("elru");
        Self {
            width: parse(&args, "--width", 720.).max(32.) as u32,
            height: parse(&args, "--height", 480.).max(32.) as u32,
            samples_per_pixel: parse(&args, "--samples", 4.).clamp(1., 16.) as u32,
            camera: Camera::from_orbit(
                Vec3::new(
                    0.,
                    parse(&args, "--target-y", if elru { 10. } else { 1. }),
                    parse(&args, "--target-z", if elru { -1. } else { 0. }),
                ),
                parse(&args, "--yaw", if elru { 110. } else { 35. }),
                parse(&args, "--pitch", if elru { 14. } else { 22. }),
                parse(&args, "--distance", if elru { 64. } else { 60. }),
                52.,
            ),
            output_path: value_after(&args, "--output"),
            time_of_day: if args.iter().any(|arg| arg == "--night" || arg == "--noche") {
                TimeOfDay::Night
            } else {
                TimeOfDay::Day
            },
            help_requested: args.iter().any(|arg| arg == "--help" || arg == "-h"),
            menu_preview: args.iter().any(|arg| arg == "--menu-preview"),
            house: match value_after(&args, "--world").as_deref() {
                Some("elru") => House::Elru,
                _ => House::Ve7,
            },
        }
    }

    pub fn print_usage() {
        println!("Encuadre: --target-y ALTURA --target-z PROFUNDIDAD. Elru usa su propia vista frontal por defecto.");
        println!("Vista del selector: --menu-preview --output menu.png");
        println!("cargo run --release -- [--world ve7|elru] [--width 720] [--height 480] [--samples 4] [--yaw 35] [--pitch 22] [--distance 60] [--night] [--output diorama.png]");
    }
}

fn parse(args: &[String], name: &str, default: f32) -> f32 {
    value_after(args, name)
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

fn value_after(args: &[String], name: &str) -> Option<String> {
    args.iter()
        .position(|arg| arg == name)
        .and_then(|index| args.get(index + 1))
        .cloned()
}
