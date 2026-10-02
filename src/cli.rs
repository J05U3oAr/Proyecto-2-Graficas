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
    pub cycle_time: Option<f32>,
    pub help_requested: bool,
    pub menu_preview: bool,
    pub house: House,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let args: Vec<String> = env::args().collect();
        let house = match value_after(&args, "--world")
            .map(|name| name.to_ascii_lowercase())
            .as_deref()
        {
            Some("elru") => House::Elru,
            Some("auropl") => House::Auropl,
            _ => House::Ve7,
        };
        let (target_y, target_z, yaw, pitch, distance) = match house {
            House::Ve7 => (1., 0., 35., 22., 60.),
            House::Elru => (10., -1., 110., 14., 64.),
            House::Auropl => (20., 0., 90., -12., 58.),
        };
        Self {
            width: parse(&args, "--width", 720.).max(32.) as u32,
            height: parse(&args, "--height", 480.).max(32.) as u32,
            samples_per_pixel: parse(&args, "--samples", 4.).clamp(1., 16.) as u32,
            camera: Camera::from_orbit(
                Vec3::new(
                    0.,
                    parse(&args, "--target-y", target_y),
                    parse(&args, "--target-z", target_z),
                ),
                parse(&args, "--yaw", yaw),
                parse(&args, "--pitch", pitch),
                parse(&args, "--distance", distance),
                52.,
            ),
            output_path: value_after(&args, "--output"),
            time_of_day: if args
                .iter()
                .any(|arg| arg == "--sunrise" || arg == "--amanecer")
            {
                TimeOfDay::Sunrise
            } else if args
                .iter()
                .any(|arg| arg == "--sunset" || arg == "--atardecer")
            {
                TimeOfDay::Sunset
            } else if args.iter().any(|arg| arg == "--night" || arg == "--noche") {
                TimeOfDay::Night
            } else {
                TimeOfDay::Day
            },
            cycle_time: value_after(&args, "--hour")
                .and_then(|value| value.parse::<f32>().ok())
                .map(|hour| ((hour - 6.) / 24.).rem_euclid(1.)),
            help_requested: args.iter().any(|arg| arg == "--help" || arg == "-h"),
            menu_preview: args.iter().any(|arg| arg == "--menu-preview"),
            house,
        }
    }

    pub fn print_usage() {
        println!("Encuadre: --target-y ALTURA --target-z PROFUNDIDAD. Cada mundo tiene su propia vista inicial.");
        println!("Vista del selector: --menu-preview --output menu.png");
        println!("cargo run --release -- [--world ve7|elru|auropl] [--width 720] [--height 480] [--samples 4] [--yaw 35] [--pitch 22] [--distance 60] [--night | --sunrise | --sunset | --hour 18] [--output diorama.png]");
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
