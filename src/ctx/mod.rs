use std::sync::Arc;
use winit::{
    window::Window, 
    platform::run_return::EventLoopExtRunReturn, 
    event::{
        Event, 
        WindowEvent, 
        KeyboardInput, 
        DeviceEvent
    }
};

pub mod gfx;
pub mod sfx;
pub mod frame;

/// 全体のコンテキスト
pub struct Context<I, F: frame::Frame<I, GCd>, GCd> where
    GCd: Send + Sync, 
{
    _dummy: std::marker::PhantomData<I>, 
    ev_loop: winit::event_loop::EventLoop<()>, 
    window: Arc<Window>, 
    gfx: gfx::GfxCtx<GCd>, 
    sfx: sfx::SfxCtx, 
    frame: F, 
}
impl<I, F: frame::Frame<I, GCd>, GCd> Context<I, F, GCd> where
    GCd: Send + Sync, 
{
    pub async fn new(
        frame_initializer: I, 
        gfx_ctx_data_init: impl FnOnce(
            &gfx::WinitCtx, 
            &gfx::WGPUCtx, 
        ) -> GCd, 
        dupdater: impl FnMut(
            &gfx::WinitCtx, 
            &gfx::WGPUCtx, 
            &mut GCd, 
        ) + 'static, 
    ) -> Result<
        Self, 
        Box<dyn std::error::Error>
    > {
        // ウィンドウのイベント受信・処理に使うイベントループの初期化
        let ev_loop = winit::event_loop::EventLoopBuilder::new().build();

        // ウィンドウの生成
        let window = F::window_builder()
            .build(&ev_loop)?;
        let window = std::sync::Arc::new(window);

        // グラフィクスの初期化
        let gfx = gfx::GfxCtx::new(
            &window, 
            gfx_ctx_data_init, 
            dupdater, 
        ).await?;

        // オーディオの初期化
        let sfx = sfx::SfxCtx::new(0.063)?;

        // フレームの初期化
        let frame = F::new(
            frame_initializer, 
            &window, 
            &gfx, 
            &sfx, 
        )?;

        Ok(Self {
            _dummy: std::marker::PhantomData, 
            ev_loop, 
            window, 
            gfx, 
            sfx, 
            frame, 
        })
    }

    /// 実行
    pub fn run(
        mut self, 
    ) -> (i32, Result<(), Box<dyn std::error::Error>>) {
        let mut ret = Ok(());
        (self.ev_loop.run_return(|
            event, 
            _, 
            ctrl
        | {
            match event {
                // ウィンドウ固有イベントの処理
                Event::WindowEvent { 
                    window_id, 
                    event 
                } if self.window.id() == window_id => match event {
                    WindowEvent::CloseRequested => ctrl.set_exit(), 
                    WindowEvent::KeyboardInput { 
                        input: KeyboardInput {
                            state,
                            virtual_keycode: Some(keycode),
                            ..
                        }, 
                        .. 
                    } => self.frame.input_key(keycode, state), 
                    WindowEvent::MouseInput { 
                        state, 
                        button, 
                        .. 
                    } => self.frame.input_mouse_button(button, state), 
                    WindowEvent::Resized(new_size) => {
                        self.gfx.reconfigure(Some(new_size));
                        self.frame.window_resizing(new_size);
                    }, 
                    _ => {}, 
                }, 
                Event::DeviceEvent { 
                    event, 
                    .. 
                } => match event {
                    DeviceEvent::MouseMotion { 
                        delta 
                    } => self.frame.input_mouse_motion(delta), 
                    DeviceEvent::MouseWheel { 
                        delta
                    } => self.frame.input_mouse_scroll(delta), 
                    _ => {}, 
                }, 
                // 描画の必要性が生じたときの処理
                Event::RedrawRequested(
                    window_id
                ) if self.window.id() == window_id => match self.gfx.rendering() {
                    Ok(
                        render_chain
                    ) => self.frame.rendering(render_chain).present(), 
                    Err(wgpu::SurfaceError::Lost) => self.gfx.reconfigure(None), 
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        eprintln!("out of memory error occured.");
                        ctrl.set_exit_with_code(-1)
                    }, 
                    Err(e) => eprintln!("{e:?}"), 
                }, 
                // すべてのイベントの処理を終えた時の処理
                Event::MainEventsCleared => {
                    if let Err(e) = self.frame.update(
                        &self.window, 
                        ctrl, 
                        &self.gfx, 
                        &self.sfx, 
                    ) {
                        ret = Err(e);
                        ctrl.set_exit();
                    }
                    // 描画要求の発令
                    self.window.request_redraw(); 
                }, 
                _ => {}, 
            }
        }), ret)
    }
}