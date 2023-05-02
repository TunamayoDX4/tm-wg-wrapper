use crate::ctx::{
    frame::Frame, 
    gfx::{
        GfxCtx, 
        Renderer, 
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

/// フレーム付属の値
pub trait FrameParam: Sized + Send + Sync + 'static {
    type Rdr: Send + Sync + Renderer;
    fn update(
        &mut self, 
        render: &Self::Rdr, 
    ) -> Result<(), Box<dyn std::error::Error>>;
}

pub trait Scene: Sized + Send + Sync + 'static {
    type Rdr: Send + Sync + Renderer;
    type Fpr: Send + Sync + FrameParam<Rdr = Self::Rdr>;
    type PopV: Send + Sync;

    /// ウィンドウビルダの出力
    fn window_builder() -> WindowBuilder;

    /// 初期処理
    fn init_proc(
        window: &Window, 
        gfx: &GfxCtx, 
        sfx: &SfxCtx, 
    ) -> Result<Self::Fpr, Box<dyn std::error::Error>>;

    /// レンダラの初期化
    fn render_init(
        gfx: &GfxCtx, 
    ) -> Result<Self::Rdr, Box<dyn std::error::Error>>;

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

    /// 処理を要するか
    fn require_process(
        &self, 
        depth: usize, 
        is_top: bool, 
    ) -> bool;

    /// 実際の処理
    fn process(
        &mut self, 
        depth: usize, 
        is_top: bool, 
        renderer: &Self::Rdr, 
        frame_param: &mut Self::Fpr, 
        window: &Window, 
        gfx: &GfxCtx, 
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
    fn rendering(
        &self, 
        depth: usize, 
        is_top: bool, 
        renderer: &mut Self::Rdr, 
        frame_param: &Self::Fpr, 
    );

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
    fparam: S::Fpr, 
    renderer: S::Rdr, 
}
impl<S, Si, Sio> Frame<Si> for SceneFrame<S> where
    S: Scene, 
    Si: FnOnce(
        &mut S::Fpr, 
        &mut S::Rdr, 
    ) -> Sio,  
    Sio: IntoIterator<Item = S>, 
{

    fn window_builder() -> winit::window::WindowBuilder {
        S::window_builder()
    }

    fn new(
        initializer: Si, 
        window: &winit::window::Window, 
        gfx: &GfxCtx, 
        sfx: &SfxCtx, 
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut fparam = S::init_proc(window, gfx, sfx)?;
        let mut renderer = S::render_init(gfx)?;
        let scenes = stack::SceneStack::new(
            initializer(&mut fparam, &mut renderer)
        );

        Ok(Self {
            scenes,
            fparam,
            renderer,
        })
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

    fn rendering<'r>(
        &mut self, 
        render_chain: RenderingChain<'r>, 
    ) -> RenderingChain<'r> {
        self.scenes.rendering(
            &mut self.renderer, 
            &self.fparam, 
        );
        render_chain.rendering(&mut self.renderer)
    }

    fn update(
        &mut self, 
        window: &winit::window::Window, 
        ctrl: &mut winit::event_loop::ControlFlow, 
        gfx: &GfxCtx, 
        sfx: &SfxCtx, 
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.fparam.update(&self.renderer)?;
        match self.scenes.process(
            &self.renderer, 
            &mut self.fparam, 
            window, 
            gfx, 
            sfx, 
        )? {
            SceneFrameCtrlParam::Exit(code) => ctrl.set_exit_with_code(code),
            SceneFrameCtrlParam::Continue => {},
        };

        Ok(())
    }
}