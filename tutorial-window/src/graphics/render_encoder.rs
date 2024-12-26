use super::{gpu, render_pass, render_target};

pub struct RenderEncoder<'a> {
    gpu: &'a gpu::Gpu,
    encoder: wgpu::CommandEncoder,
}

impl<'a> RenderEncoder<'a> {
    pub fn new(gpu: &'a gpu::Gpu) -> Self {
        let encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render_encoder"),
            });

        Self { gpu, encoder }
    }

    pub fn render_pass<'b>(
        &'b mut self,
        render_target: &'b render_target::SurfaceRenderTarget,
        clear: Option<wgpu::Color>,
    ) -> render_pass::RenderPass<'b> {
        render_pass::RenderPass::new(self.gpu, &mut self.encoder, render_target, clear)
    }

    pub fn finish(self) {
        self.gpu
            .queue
            .submit(std::iter::once(self.encoder.finish()));
    }
}
