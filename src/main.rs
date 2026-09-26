mod app;
mod camera;
mod cli;
mod geometry;
mod image_exporter;
mod materials;
mod raytracer;
mod renderer;
mod scene;
mod scene_builder;

use app::InteractiveApp;
use cli::AppConfig;
use image_exporter::ImageExporter;
use renderer::Renderer;
use scene_builder::SceneBuilder;

fn main() {
    let config = AppConfig::from_env();
    if config.help_requested {
        AppConfig::print_usage();
        return;
    }

    let scene = SceneBuilder::build();
    let renderer = Renderer::new(config.width, config.height, config.samples_per_pixel);
    if let Some(path) = config.output_path {
        let pixels = renderer.render(&scene, config.camera);
        ImageExporter::save(&path, &pixels, config.width, config.height)
            .expect("Could not save image");
        println!("Imagen guardada: {path}");
    } else {
        InteractiveApp::new(scene, renderer, config.camera).run();
    }
}
