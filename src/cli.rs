use std::env;

use crate::camera::Camera;
use crate::geometry::Vec3;

pub struct AppConfig {
    pub width: u32,
    pub height: u32,
    pub camera: Camera,
    pub output_path: Option<String>,
    pub help_requested: bool,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let args: Vec<String> = env::args().collect();
        Self {
            width: parse(&args, "--width", 480.).max(32.) as u32,
            height: parse(&args, "--height", 320.).max(32.) as u32,
            camera: Camera::new(
                Vec3::new(0., 1., 0.),
                parse(&args, "--yaw", 35.),
                parse(&args, "--pitch", 22.),
                parse(&args, "--distance", 26.),
                52.,
            ),
            output_path: value_after(&args, "--output"),
            help_requested: args.iter().any(|arg| arg == "--help" || arg == "-h"),
        }
    }

    pub fn print_usage() {
        println!("cargo run --release -- [--width 480] [--height 320] [--yaw 35] [--pitch 22] [--distance 26] [--output diorama.png]");
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
