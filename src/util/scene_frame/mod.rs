use crate::ctx::{
    frame::Frame, 
    gfx::{
        GfxCtx, 
        RenderingChain, 
    }, 
    sfx::SfxCtx, 
};
use winit::{
    window::{
        Window, 
        WindowBuilder, 
    }, 
    event::{
        VirtualKeyCode, 
        MouseButton, 
        MouseScrollDelta, 
        ElementState, 
    }, 
};

pub mod instance;
pub mod stack;

pub trait Scene: Sized + Send + Sync {
    type InitV;
    type Rdr: Send + Sync;
    type FrG: Send + Sync + crate::ctx::frame::FrameGlobal<
        Self::Rdr, 
    >;
    type PopV: Send + Sync;

    /// ウィンドウビルダの出力
    fn window_builder() -> WindowBuilder;

    /// 初期処理
    fn init_proc(
        init_v: Self::InitV, 
        window: &Window, 
        gfx: &GfxCtx<Self::Rdr>, 
        sfx: &SfxCtx, 
    ) -> Result<
        (
            Vec<Self>, 
            Self::FrG, 
        ), 
        Box<dyn std::error::Error>
    >;

    /// キー入力
    fn input_key(
        &mut self, 
        keycode: VirtualKeyCode, 
        state: ElementState, 
    );

    /// マウス入力
    fn input_mouse_button(
        &mut self, 
        button: MouseButton, 
        state: ElementState, 
    );

    /// マウス動作入力
    fn input_mouse_motion(
        &mut self, 
        delta: (f64, f64), 
    );

    /// マウススクロール入力
    fn input_mouse_scroll(
        &mut self, 
        delta: MouseScrollDelta, 
    );

    /// ウィンドウのリサイズ
    fn window_resizing(
        &mut self, 
        size: winit::dpi::PhysicalSize<u32>, 
    );

    /// 実際の処理
    fn process(
        &mut self, 
        depth: usize, 
        is_top: bool, 
        fglob: &Self::FrG, 
        gfx: &GfxCtx<Self::Rdr>, 
        sfx: &SfxCtx, 
    ) -> Result<
        SceneProcOp<Self>, 
        Box<dyn std::error::Error>
    >;

    /// 描画を要するか
    fn require_rendering(
        &self, 
        depth: usize, 
        is_top: bool, 
    ) -> bool;

    /// 実際の描画
    fn rendering<'a, 'b>(
        &mut self, 
        render_chain: RenderingChain<'a, 'b, Self::Rdr, Self::FrG>, 
        depth: usize, 
        is_top: bool,  
    ) -> RenderingChain<'a, 'b, Self::Rdr, Self::FrG>;

    /// ポップ時の処理
    fn pop(self) -> Self::PopV;

    /// フォアグラウンドに戻った時の処理
    fn return_foreground(&mut self, popv: Self::PopV);
}

/// シーン処理時の出力コマンド
pub enum SceneProcOp<S: Scene> {
    Nop, 
    StkCtl(SceneStackCtrlOp<S>), 
}

/// シーン処理時のスタック制御コマンド
pub enum SceneStackCtrlOp<S: Scene> {
    /// スタックにシーンをプッシュする
    Push(S), 
    
    /// スタックからシーンをポップする
    Pop, 

    /// スタックを空にして新しくプッシュする
    PopAll(S), 

    /// スタックを空にして離脱する
    Exit, 
}

/// シーンフレームの制御パラメータ
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneFrameCtrlParam {
    /// 処理を終了し、離脱する
    Exit(i32), 

    /// 処理を続行する
    Continue, 
}

/// シーン・フレーム
pub struct SceneFrame<S: Scene> {
    scenes: stack::SceneStack<S>, 
}
impl<S> Frame<S::InitV, S::Rdr> for SceneFrame<S> where
    S: Scene, 
{
    type FrG = S::FrG;

    fn window_builder() -> winit::window::WindowBuilder {
        S::window_builder()
    }

    fn new(
        initializer: S::InitV, 
        window: &winit::window::Window, 
        gfx: &GfxCtx<S::Rdr>, 
        sfx: &SfxCtx, 
    ) -> Result<(Self::FrG, Self), Box<dyn std::error::Error>> {
        let (
            scenes, 
            fparam, 
        ) = S::init_proc(
            initializer, 
            window, 
            gfx, 
            sfx
        )?;
        let scenes = stack::SceneStack::new(
            scenes
        );

        let r = (
            fparam, 
            Self {
                scenes, 
            }
        );

        Ok(r)
    }

    fn input_key(
        &mut self, 
        keycode: VirtualKeyCode, 
        state: ElementState, 
    ) {
        self.scenes.input_key(keycode, state)
    }

    fn input_mouse_button(
        &mut self, 
        button: MouseButton, 
        state: ElementState, 
    ) {
        self.scenes.input_mouse_button(button, state)
    }

    fn input_mouse_motion(
        &mut self, 
        delta: (f64, f64), 
    ) {
        self.scenes.input_mouse_motion(delta)
    }

    fn input_mouse_scroll(
        &mut self, 
        delta: MouseScrollDelta, 
    ) {
        self.scenes.input_mouse_scroll(delta)
    }

    fn window_resizing(
        &mut self, 
        size: winit::dpi::PhysicalSize<u32>, 
    ) {
        self.scenes.window_resizing(size);
    }

    fn rendering<'r, 'f>(
        &mut self, 
        render_chain: RenderingChain<'r, 'f, S::Rdr, Self::FrG>, 
    ) -> RenderingChain<'r, 'f, S::Rdr, Self::FrG> {
        self.scenes.rendering(
            render_chain, 
        )
    }

    fn update(
        &mut self, 
        ctrl: &mut winit::event_loop::ControlFlow, 
        fglob: &Self::FrG, 
        gfx: &GfxCtx<S::Rdr>, 
        sfx: &SfxCtx, 
    ) -> Result<(), Box<dyn std::error::Error>> {
        match self.scenes.process(
            fglob, 
            gfx, 
            sfx, 
        )? {
            SceneFrameCtrlParam::Exit(code) => ctrl.set_exit_with_code(code),
            SceneFrameCtrlParam::Continue => {},
        };

        Ok(())
    }
}