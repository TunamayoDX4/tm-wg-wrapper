use winit::{
    event::{
        VirtualKeyCode, 
        ElementState, 
        MouseButton, 
        MouseScrollDelta
    }, 
    event_loop::ControlFlow
};

pub trait FrameGlobal<GCd> where
    Self: Sized + Send + Sync + 'static, 
    GCd: Send + Sync, 
{
    fn update(
        &mut self, 
        gfx: &super::gfx::GfxCtx<GCd>, 
        sfx: &super::sfx::SfxCtx, 
    ) -> Result<(), Box<dyn std::error::Error>>;
}

/// コンテキスト処理を抽象化し、また使いまわしが出来るようにするフレーム
pub trait Frame<I, GCd> where
    Self: Sized + Send + Sync + 'static, 
    GCd: Send + Sync, 
{
    type FrG: FrameGlobal<GCd>;

    fn window_builder() -> winit::window::WindowBuilder;
    fn new(
        initializer: I, 
        window: &winit::window::Window, 
        gfx: &super::gfx::GfxCtx<GCd>, 
        sfx: &super::sfx::SfxCtx, 
    ) -> Result<(
        Self::FrG, 
        Self
    ), Box<dyn std::error::Error>>;
    fn input_key(
        &mut self, 
        keycode: VirtualKeyCode, 
        state: ElementState, 
    );
    fn input_mouse_button(
        &mut self, 
        button: MouseButton, 
        state: ElementState, 
    );
    fn input_mouse_motion(
        &mut self, 
        delta: (f64, f64), 
    );
    fn input_mouse_scroll(
        &mut self, 
        delta: MouseScrollDelta, 
    );
    fn window_resizing(
        &mut self, 
        size: winit::dpi::PhysicalSize<u32>, 
    );
    fn rendering<'r, 'f>(
        &mut self, 
        render_chain: super::gfx::RenderingChain<'r, 'f, GCd, Self::FrG>, 
    ) -> super::gfx::RenderingChain<'r, 'f, GCd, Self::FrG>;
    fn update(
        &mut self, 
        ctrl: &mut ControlFlow, 
        fglob: &Self::FrG, 
        gfx: &super::gfx::GfxCtx<GCd>, 
        sfx: &super::sfx::SfxCtx, 
    ) -> Result<(), Box<dyn std::error::Error>>;
}