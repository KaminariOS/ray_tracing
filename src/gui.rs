use crate::winit_egui::Painter;
use crate::Renderer;
use egui::{ClippedPrimitive, Context, TexturesDelta};
use image::ColorType;
use pixels::{Pixels, PixelsContext};
use winit::window::Window;

/// Manages all state required for rendering egui over `Pixels`.
pub(crate) struct Framework {
    // State for egui.
    egui_ctx: Context,
    egui_state: egui_winit::State,
    painter: Painter,
    paint_jobs: Vec<ClippedPrimitive>,
    textures: TexturesDelta,
    // State for the GUI
    pub(crate) gui: Gui,
    pub scale_factor: f32,
}

/// Example application state. A real application will need a lot more state than this.
impl Framework {
    /// Create egui.
    pub(crate) fn new(window: &Window, pixels: &Pixels) -> Self {
        let painter = Painter::new(window, pixels, 1);
        let scale_factor = window.scale_factor() as f32;
        let egui_ctx = Context::default();
        let egui_state = egui_winit::State::from_pixels_per_point(
            pixels.device().limits().max_texture_dimension_2d as usize,
            scale_factor,
        );

        let textures = TexturesDelta::default();
        let gui = Gui::new();

        Self {
            egui_ctx,
            egui_state,
            painter,
            paint_jobs: Vec::new(),
            textures,
            gui,
            scale_factor,
        }
    }

    /// Handle input events from the window manager.
    pub(crate) fn handle_event(&mut self, event: &winit::event::WindowEvent) {
        self.egui_state.on_event(&self.egui_ctx, event);
    }

    /// Resize egui.
    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.painter.on_window_resized(width, height);
        }
    }

    /// Prepare egui.
    pub(crate) fn prepare(&mut self, window: &Window) {
        // Run the egui frame and create all paint jobs to prepare for rendering.
        let raw_input = self.egui_state.take_egui_input(window);
        let output = self.egui_ctx.run(raw_input, |egui_ctx| {
            // Draw the demo application.
            self.gui.ui(egui_ctx);
        });

        self.textures.append(output.textures_delta);
        self.egui_state
            .handle_platform_output(window, &self.egui_ctx, output.platform_output);
        self.paint_jobs = self.egui_ctx.tessellate(output.shapes);
    }

    /// Render egui.
    pub(crate) fn render(
        &mut self,
        context: &PixelsContext,
        render_target: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        self.painter.paint_and_update_textures(
            self.scale_factor,
            &self.paint_jobs,
            &self.textures,
            context,
            render_target,
            encoder,
        );

        // Cleanup
        self.textures.clear();
    }

    pub fn save_img(&mut self, renderer: &Renderer, pixels: &mut Pixels) {
        if self.gui.save_img {
            self.gui.save_img = false;
            image::save_buffer(
                "screenshot.png",
                pixels.get_frame(),
                renderer.width,
                renderer.height,
                ColorType::Rgba8,
            )
            .ok();
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct Gui {
    /// Only show the egui window when true.
    window_open: bool,
    my_boolean: bool,
    pub scale: u32,
    pub save_img: bool,
    pub sample_count: usize,
    pub max_depth: usize,
    pre: Option<Box<Gui>>
}



impl Gui {
    /// Create a `Gui`.
    fn new() -> Self {
        let mut cur = Self {
            window_open: false,
            my_boolean: false,
            scale: 10,
            save_img: false,
            sample_count: 4,
            max_depth: 10,
            pre: None
        };
        cur.pre = Some(Box::new(cur.clone()));
        cur
    }

    pub fn updated(&mut self) -> bool {

        if let Some(pre) = self.pre.take() {
            let dirty =  *pre != *self;
            self.pre = Some(pre);
            return dirty
        }
        false
    }

    pub fn update(&mut self) {
        self.pre.take();
        self.pre = Some(Box::new(self.clone()));
    }
    /// Create the UI using egui.
    fn ui(&mut self, ctx: &Context) {
        egui::TopBottomPanel::top("menubar_container").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("About...").clicked() {
                        self.window_open = true;
                        ui.close_menu();
                    }
                })
            });
        });

        egui::Window::new("Hello, egui!")
            .open(&mut self.window_open)
            .show(ctx, |ui| {
                ui.label("This example demonstrates using egui with pixels.");
                ui.label("Made with 💖 in San Francisco!");

                ui.separator();

                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x /= 2.0;
                    ui.label("Learn more about egui at");
                    ui.hyperlink("https://docs.rs/egui");
                });
            });
        egui::Window::new("df").show(ctx, |ui| {
            ui.add(egui::Label::new("Hello World!"));
            ui.label("A shorter and more convenient way to add a label.");
            if ui.button("Take a screenshot").clicked() {
                self.save_img = true;
            }
            ui.label("Scale");
            ui.add(egui::Slider::new(&mut self.scale, 1..=20));
            // ui.add(egui::DragValue::new(&mut self.scale));
            ui.label("SampleCount");
            ui.add(egui::Slider::new(&mut self.sample_count, 1..=50));
            ui.label("Max depth");
            ui.add(egui::Slider::new(&mut self.max_depth, 1..=50));
            ui.checkbox(&mut self.my_boolean, "Checkbox");
        });
    }
}
