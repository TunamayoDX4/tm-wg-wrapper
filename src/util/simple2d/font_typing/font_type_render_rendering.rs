use super::super::{
    SquareShared, 
    ImagedShared, 
    img_obj::ImgObjRenderShared, 
    Simple2DRender
};

impl Simple2DRender for super::FontTypeRender {
    type Shared<'a> = (
        &'a SquareShared, 
        &'a ImagedShared, 
        &'a ImgObjRenderShared, 
    );

    fn rendering<'a>(
        &mut self, 
        gfx: &crate::ctx::gfx::GfxCtx, 
        encoder: &mut wgpu::CommandEncoder, 
        view: &wgpu::TextureView, 
        camera: &crate::prelude::simple2d::shared::S2DCamera, 
        shared: Self::Shared<'a>, 
    ) {
        self.renderer.rendering(gfx, encoder, view, camera, shared)
    }
}