use std::rc::Rc;
use log::error;
use pixels::{Pixels, SurfaceTexture};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use winit::event::{Event, VirtualKeyCode};
use winit_input_helper::WinitInputHelper;
use winit::dpi::{LogicalSize, PhysicalSize};

const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
     #[cfg(target_arch = "wasm32")] {
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
    let mut renderer = Renderer{width: WIDTH, height: HEIGHT};
    let mut input = WinitInputHelper::new();
    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            renderer.draw(pixels.get_frame());
            if pixels
                .render()
                .map_err(|e| error!("pixels.render() failed: {}", e))
                .is_err()
            {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Resize the window
            if let Some(PhysicalSize {width, height}) = input.window_resized() {
                pixels.resize_surface(width, height);
                let scale = 20;
                pixels.resize_buffer(width / scale, height / scale);
                renderer.resize(width / scale, height / scale);
                log::info!("window resized {:?} : {}", (width, height), pixels.get_frame().len());
            }
            // Update internal state and request a redraw
            window.request_redraw();
        }
    });
}

struct Renderer {
    width: u32,
    height: u32
}

impl Renderer {
    fn draw(&self, frame: &mut [u8]) {
        assert_eq!((frame.len() / 4) as u32, self.width * self.height);
        frame.chunks_exact_mut(4).enumerate().for_each(|(i, pixel)| {
            let (y, x) = self.cal_coords(i);
            Self::draw_checkerboard(x, y, pixel);
        });
    }
    fn draw_checkerboard(x: u32, y: u32, pixel: &mut [u8]) {
        let color = if (x % 2 == 0) ^ (y % 2 == 0) {0} else {255};
        let rgba = [color, color, color, 0xff];
        pixel.copy_from_slice(&rgba);
    }
    fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }
    fn cal_coords(&self, i: usize) -> (u32, u32){
        let i = i as u32;
        (i / self.width, i % self.width)
    }
}