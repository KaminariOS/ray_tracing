use crate::renderer::Renderer;
use crate::scene::select_scene;
use crate::{HEIGHT, WIDTH};
use clap::Parser;
use crate::camera::Camera;

/// Get ray tracing parameters
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, default_value_t = 50)]
    max_depth: usize,
    #[clap(short, long, default_value_t = 100)]
    sample_count: usize,
    #[clap(short, long, default_value_t = 10)]
    down_scale: u32,
    #[clap(long, default_value = "random")]
    scene: String,
}

pub fn image_mode() {
    let args = Args::parse();
    let scale = args.down_scale;
    let (width, height) = (WIDTH / scale, HEIGHT / scale);
    let camera = Camera::select_camera(width as f32 / height as f32, &args.scene);
    let mut renderer = Renderer::new(width, height, select_scene(&args.scene), camera);
    renderer.multisample = args.sample_count;
    renderer.max_depth = args.max_depth;
    let mut pixels = vec![0; (width * height * 4) as usize];
    renderer.draw(&mut pixels);
    image::save_buffer(
        "screenshot.png",
        &pixels,
        renderer.width,
        renderer.height,
        image::ColorType::Rgba8,
    )
    .ok();
    std::process::Command::new("sh")
        .arg("-c")
        .arg("play /usr/share/sounds/Oxygen-Im-New-Mail.ogg")
        .output()
        .expect("Failed to play");
}
