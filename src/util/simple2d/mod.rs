pub mod instance;
pub mod raw;
pub mod types;
pub mod shared;

/// レンダラ
pub mod img_obj;
pub mod font_typing;

pub use instance::{
    Instance, 
    InstanceGen, 
    buffer::InstanceArray, 
};
pub use types::Camera;
pub use shared::{
    ImagedShared, 
    SquareShared, 
    S2DCamera, 
};

pub mod entity_holder;
pub mod physic;
pub mod util;

/// シンプルな2Dレンダラー
pub trait Simple2DRender<GCd> where
    Self: Send + Sync + Sized + 'static, 
    GCd: Send + Sync,     
{
    type Shared<'a>: Send + Sync + Sized;
    fn rendering<'a>(
        &mut self, 
        gfx: &crate::ctx::gfx::GfxCtx<GCd>, 
        encoder: &mut wgpu::CommandEncoder, 
        view: &wgpu::TextureView, 
        camera: &shared::S2DCamera, 
        shared: Self::Shared<'a>, 
    );
}