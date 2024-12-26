pub struct SurfaceRenderTarget {
    pub(super) surface_texture: wgpu::SurfaceTexture,
    pub(super) target_view: wgpu::TextureView,
}

impl SurfaceRenderTarget {
    pub fn finish(self) {
        self.surface_texture.present();
    }
}
