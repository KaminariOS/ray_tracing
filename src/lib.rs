use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "window")] {
use std::rc::Rc;
use pixels::{Pixels, SurfaceTexture};
use winit::dpi::{LogicalSize, PhysicalSize};
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;
mod gui;
use gui::Framework;
mod winit_egui;
    }
}
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::geo::create_random_scene;
use crate::ray::{Hittable, Ray};
use crate::renderer::Renderer;

mod camera;
mod geo;
mod material;
mod rand_gen;
mod ray;
mod renderer;
mod types;
mod aabb;
mod texture;

extern crate nalgebra as na;
const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
#[cfg(feature = "window")]
pub async fn run() {
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init_with_level(log::Level::Info).expect("Could't initialize logger");
    }

    let event_loop = EventLoop::new();
    let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
    let window = WindowBuilder::new()
        .with_inner_size(size)
        .with_title("ray_tracing")
        .build(&event_loop)
        .expect("WindowBuilder error");
    let window = Rc::new(window);

    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;
        use winit::platform::web::WindowExtWebSys;

        // Retrieve current width and height dimensions of browser client window
        let get_window_size = || {
            let client_window = web_sys::window().unwrap();
            LogicalSize::new(
                client_window.inner_width().unwrap().as_f64().unwrap(),
                client_window.inner_height().unwrap().as_f64().unwrap(),
            )
        };

        let window = Rc::clone(&window);

        // Initialize winit window with current dimensions of browser client
        window.set_inner_size(get_window_size());

        let client_window = web_sys::window().unwrap();

        // Attach winit canvas to body element
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| doc.body())
            .and_then(|body| {
                body.append_child(&web_sys::Element::from(window.canvas()))
                    .ok()
            })
            .expect("couldn't append canvas to document body");

        // Listen for resize event on browser client. Adjust winit window dimensions
        // on event trigger
        let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move |_e: web_sys::Event| {
            let size = get_window_size();
            window.set_inner_size(size)
        }) as Box<dyn FnMut(_)>);
        client_window
            .add_event_listener_with_callback("resize", closure.as_ref().unchecked_ref())
            .unwrap();
        closure.forget();
    }

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture =
            SurfaceTexture::new(window_size.width, window_size.height, window.as_ref());
        Pixels::new_async(WIDTH, HEIGHT, surface_texture)
            .await
            .expect("Pixels error")
    };
    let mut framework = Framework::new(&window, &pixels);
    let mut renderer = Renderer::new(WIDTH, HEIGHT, create_random_scene());
    renderer.update_from_gui(&framework.gui, &mut pixels);
    let mut input = WinitInputHelper::new();
    // let mut last = instant::Instant::now();
    event_loop.run(move |event, _, control_flow| {
        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            if let Some(scale_factor) = input.scale_factor() {
                framework.scale_factor = scale_factor as f32;
            }

            framework.save_img(&renderer, &mut pixels);
            // Resize the window
            if let Some(PhysicalSize { width, height }) = input.window_resized() {
                renderer.resize(width, height, &mut pixels);
                framework.resize(width, height);
                log::info!(
                    "window resized {:?} : {}",
                    (width, height),
                    pixels.get_frame().len()
                );
            }
            // Update internal state and request a redraw
            window.request_redraw();
        }
        match event {
            Event::WindowEvent { event, .. } => {
                // Update egui inputs
                framework.handle_event(&event);
            }
            // Draw the current frame
            Event::RedrawRequested(_) => {
                // let now = instant::Instant::now();
                // let dt = now - last;
                renderer.draw(pixels.get_frame());
                renderer.dirty = framework.gui.updated();

                // Prepare egui
                framework.prepare(&window);
                let render_result = pixels.render_with(|encoder, render_target, context| {
                    // Render the world texture
                    context.scaling_renderer.render(encoder, render_target);
                    framework.render(context, render_target, encoder);
                    // Render egui
                    Ok(())
                });
                // Render everything together
                if renderer.dirty {
                    renderer.update_from_gui(&framework.gui, &mut pixels);
                }
                // Basic error handling
                if render_result
                    .map_err(|e| log::error!("pixels.render() failed: {}", e))
                    .is_err()
                {
                    *control_flow = ControlFlow::Exit;
                }
            }
            _ => (),
        }
    });
}

#[cfg(feature = "windowless")]
pub fn image_mode() {
    let scale = option_env!("SCALE").unwrap_or("1").parse::<u32>().unwrap();
    let (width, height) = (WIDTH / scale, HEIGHT / scale);
    let mut renderer = Renderer::new(width, height, create_random_scene());
    renderer.multisample = option_env!("SAMPLE")
        .unwrap_or("4")
        .parse::<usize>()
        .unwrap();
    renderer.max_depth = option_env!("DEPTH")
        .unwrap_or("10")
        .parse::<usize>()
        .unwrap();
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
}
