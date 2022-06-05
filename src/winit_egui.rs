use egui_wgpu::renderer;
use pixels::{Pixels, PixelsContext};

/// Everything you need to paint egui with [`wgpu`] on [`winit`].
///
/// Alternatively you can use [`crate::renderer`] directly.
pub struct Painter {
    surface_config: wgpu::SurfaceConfiguration,
    egui_rpass: renderer::RenderPass,
}

impl Painter {
    /// Creates a [`wgpu`] surface for the given window, and things required to render egui onto it.
    ///
    /// # Safety
    /// The given `window` must outlive the returned [`Painter`].
    pub fn new(window: &winit::window::Window, pixels: &Pixels, msaa_samples: u32) -> Self {
        let size = window.inner_size();
        let surface_format = pixels.render_texture_format();
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width as u32,
            height: size.height as u32,
            present_mode: wgpu::PresentMode::Fifo,
        };

        let egui_rpass = renderer::RenderPass::new(pixels.device(), surface_format, msaa_samples);

        Self {
            surface_config,
            egui_rpass,
        }
    }

    pub fn on_window_resized(&mut self, width_in_pixels: u32, height_in_pixels: u32) {
        self.surface_config.width = width_in_pixels;
        self.surface_config.height = height_in_pixels;
    }

    pub fn paint_and_update_textures(
        &mut self,
        pixels_per_point: f32,
        clipped_primitives: &[egui::ClippedPrimitive],
        textures_delta: &egui::TexturesDelta,
        context: &PixelsContext,
        render_target: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        // Upload all resources for the GPU.
        let screen_descriptor = renderer::ScreenDescriptor {
            size_in_pixels: [self.surface_config.width, self.surface_config.height],
            pixels_per_point,
        };

        for (id, image_delta) in &textures_delta.set {
            self.egui_rpass
                .update_texture(&context.device, &context.queue, *id, image_delta);
        }
        for id in &textures_delta.free {
            self.egui_rpass.free_texture(id);
        }

        self.egui_rpass.update_buffers(
            &context.device,
            &context.queue,
            clipped_primitives,
            &screen_descriptor,
        );

        // Record all render passes.
        self.egui_rpass.execute(
            encoder,
            render_target,
            clipped_primitives,
            &screen_descriptor,
            None,
        );
    }
}
