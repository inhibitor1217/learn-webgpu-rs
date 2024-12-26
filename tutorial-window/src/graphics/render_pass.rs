use super::{gpu, render_target};

pub struct RenderPass<'a> {
    render_pass: wgpu::RenderPass<'a>,
}

impl<'a> RenderPass<'a> {
    pub(super) fn new(
        gpu: &'a gpu::Gpu,
        command_encoder: &'a mut wgpu::CommandEncoder,
        render_target: &'a render_target::SurfaceRenderTarget,
        clear: Option<wgpu::Color>,
    ) -> Self {
        let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("render_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &render_target.target_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(clear.unwrap_or(wgpu::Color::BLACK)),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        render_pass.set_pipeline(&gpu.render_pipeline);
        render_pass.draw(0..3, 0..1);

        Self { render_pass }
    }
}
