pub mod ctx;
pub mod util;

pub mod prelude {
    pub use winit::{
        self, 
        window::Window, 
        event::{
            ElementState, 
            VirtualKeyCode, 
            MouseButton, 
            MouseScrollDelta, 
        }, 
    };
    pub use wgpu;
    pub use wgpu_glyph;
    pub use rodio;
    pub use nalgebra;
    pub use rusttype;
    pub use image;

    pub use crate::ctx::{
        Context, 
        frame::{
            Frame,
            FrameGlobal, 
        }, 
        gfx::{
            GfxCtx, 
            Renderer, 
        }, 
        sfx::SfxCtx, 
    };
    pub use crate::util::*;
}