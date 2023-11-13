use super::render_core::TextRenderShared;
use super::super::super::{
    SquareShared, 
    ImagedShared, 
    Simple2DRender, 
    S2DCamera, 
};
use crate::ctx::gfx::GfxCtx;

impl<GCd: Send + Sync> Simple2DRender<GCd> for super::TypeRenderer {
    type Shared<'a> = (
        &'a SquareShared, 
        &'a ImagedShared, 
        &'a TextRenderShared, 
    );

    fn rendering<'a>(
        &mut self, 
        gfx: &GfxCtx<GCd>, 
        encoder: &mut wgpu::CommandEncoder, 
        view: &wgpu::TextureView, 
        camera: &S2DCamera, 
        shared: Self::Shared<'a>, 
    ) {
        self.atlas.write_texture(self.rdr.texture_get(), gfx, shared.1);
        self.rdr.rendering(gfx, encoder, view, camera, shared)
    }
}