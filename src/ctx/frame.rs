use winit::{
    event::{
        VirtualKeyCode, 
        ElementState, 
        MouseButton, 
        MouseScrollDelta
    }, 
    event_loop::ControlFlow
};

/// コンテキスト処理を抽象化し、また使いまわしが出来るようにするフレーム
pub trait Frame: Sized + Send + Sync + 'static {
    type Initializer;
    fn window_builder() -> winit::window::WindowBuilder;
    fn new(
        initializer: Self::Initializer, 
        window: &winit::window::Window, 
        gfx: &super::gfx::GfxCtx, 
        sfx: &super::sfx::SfxCtx, 
    ) -> Result<Self, Box<dyn std::error::Error>>;
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
    fn rendering<'r>(
        &mut self, 
        render_chain: super::gfx::RenderingChain<'r>, 
    ) -> super::gfx::RenderingChain<'r>;
    fn update(
        &mut self, 
        window: &winit::window::Window, 
        ctrl: &mut ControlFlow, 
        gfx: &super::gfx::GfxCtx, 
        sfx: &super::sfx::SfxCtx, 
    ) -> Result<(), Box<dyn std::error::Error>>;
}