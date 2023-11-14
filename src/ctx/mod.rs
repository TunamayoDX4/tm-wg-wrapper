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

use self::frame::FrameGlobal;

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
    fglob: F::FrG, 
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
        ) -> Result<GCd, Box<dyn std::error::Error>>, 
        dupdater: impl FnMut(
            &gfx::WinitCtx, 
            &gfx::WGPUCtx, 
            &mut GCd, 
        ) -> Result<(), Box<dyn std::error::Error>> + 'static, 
        dreconfigureer: impl FnMut(
            &gfx::WinitCtx, 
            &gfx::WGPUCtx, 
            &mut GCd, 
        ) -> Result<(), Box<dyn std::error::Error>> + 'static, 
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
            dreconfigureer
        ).await?;

        // オーディオの初期化
        let sfx = sfx::SfxCtx::new(0.063)?;

        // フレームの初期化
        let (fglob, frame) = F::new(
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
            fglob, 
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
                    WindowEvent::Resized(
                        new_size
                    ) => match self.gfx.reconfigure(Some(new_size)) {
                        Ok(_) => self.frame.window_resizing(new_size), 
                        Err(e) => {
                            ret = Err(e);
                            ctrl.set_exit();
                        }
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
                ) if self.window.id() == window_id => match self.gfx.rendering(
                    &self.fglob
                ) {
                    Ok(
                        render_chain
                    ) => self.frame.rendering(
                        render_chain
                    ).present(), 
                    Err(gfx::GfxCtxRenderingError::SurfaceError(
                        wgpu::SurfaceError::Lost
                    )) => match self.gfx.reconfigure(
                        None
                    ) {
                        Ok(_) => {}, 
                        Err(e) => {
                            ret = Err(e);
                            ctrl.set_exit();
                        }
                    }, 
                    Err(gfx::GfxCtxRenderingError::SurfaceError(
                        wgpu::SurfaceError::OutOfMemory
                    )) => {
                        eprintln!("out of memory error occured.");
                        ctrl.set_exit_with_code(-1)
                    }, 
                    Err(gfx::GfxCtxRenderingError::SurfaceError(
                        e
                    )) => eprintln!("{e:?}"), 
                    Err(gfx::GfxCtxRenderingError::RdrUpdateError(
                        e
                    )) => {
                        ret = Err(e);
                        ctrl.set_exit();
                    }
                }, 
                // すべてのイベントの処理を終えた時の処理
                Event::MainEventsCleared => {
                    match match self.fglob.update(
                        &self.gfx, 
                        &self.sfx
                    ) {
                        Ok(_) => self.frame.update(
                            ctrl, 
                            &self.fglob, 
                            &self.gfx, 
                            &self.sfx, 
                        ), 
                        e @ _ => e, 
                    } {
                        Ok(_) => self.window.request_redraw(), 
                        Err(e) => {
                            ret = Err(e);
                            ctrl.set_exit();
                        }
                    }
                    // 描画要求の発令
                    self.window.request_redraw(); 
                }, 
                _ => {}, 
            }
        }), ret)
    }
}