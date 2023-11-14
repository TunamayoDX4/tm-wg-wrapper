use std::sync::Arc;
use parking_lot::{Mutex, RwLock, RwLockReadGuard};
use wgpu::{
    Device, 
    Surface, 
    Queue, 
    SurfaceConfiguration, 
    SurfaceTexture, 
    TextureView, 
};
use winit::window::Window;

/// WGPUのコンテキスト
pub struct WGPUCtx {
    pub surface: Surface, 
    pub device: Device, 
    pub queue: Queue, 
    pub config: SurfaceConfiguration, 
}

/// Winitのコンテキスト
pub struct WinitCtx {
    pub window: Arc<Window>, 
}

/// GfxCtxのデータ
pub struct GCData<D: Send + Sync> {
    data: RwLock<D>, 
    updater: Mutex<Box<dyn FnMut(
        &WinitCtx, 
        &WGPUCtx, 
        &mut D, 
    ) -> Result<(), Box<dyn std::error::Error>>>>, 
    reconfigureer: Mutex<Box<dyn FnMut(
        &WinitCtx, 
        &WGPUCtx, 
        &mut D, 
    ) -> Result<(), Box<dyn std::error::Error>>>>, 
}
impl<D: Send + Sync> GCData<D> {
    pub fn new(
        data: D, 
        updater: impl FnMut(
            &WinitCtx, 
            &WGPUCtx, 
            &mut D, 
        ) -> Result<(), Box<dyn std::error::Error>> + 'static, 
        reconfigureer: impl FnMut(
            &WinitCtx, 
            &WGPUCtx, 
            &mut D, 
        ) -> Result<(), Box<dyn std::error::Error>> + 'static, 
    ) -> Self { Self {
        data: RwLock::new(data),
        updater: Mutex::new(Box::new(updater)),
        reconfigureer: Mutex::new(Box::new(reconfigureer)), 
    }}
    pub fn get(&self) -> RwLockReadGuard<D> { self.data.read() }
    fn update(
        &self, 
        winit_ctx: &WinitCtx, 
        wgpu_ctx: &WGPUCtx, 
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.updater.lock()(
            winit_ctx, 
            wgpu_ctx, 
            &mut *self.data.write()
        )
    }
    fn reconfigure(
        &self, 
        winit_ctx: &WinitCtx, 
        wgpu_ctx: &WGPUCtx, 
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.reconfigureer.lock()(
            winit_ctx, 
            wgpu_ctx, 
            &mut *self.data.write()
        )
    }
}

/// GfxCtxの描画時に発生するエラー
pub enum GfxCtxRenderingError {
    SurfaceError(wgpu::SurfaceError), 
    RdrUpdateError(Box<dyn std::error::Error>), 
}

/// グラフィック機能をまとめるコンテキスト
pub struct GfxCtx<D: Send + Sync> {
    pub winit_ctx: WinitCtx, 
    pub wgpu_ctx: WGPUCtx, 
    pub data: GCData<D>, 
}
impl<D: Send + Sync> GfxCtx<D> {
    pub async fn new(
        window: &Arc<Window>, 
        dinit: impl FnOnce(
            &WinitCtx, 
            &WGPUCtx, 
        ) -> Result<D, Box<dyn std::error::Error>>, 
        dupdater: impl FnMut(
            &WinitCtx, 
            &WGPUCtx, 
            &mut D, 
        ) -> Result<(), Box<dyn std::error::Error>> + 'static, 
        dreconfigureer: impl FnMut(
            &WinitCtx, 
            &WGPUCtx, 
            &mut D, 
        ) -> Result<(), Box<dyn std::error::Error>> + 'static, 
    ) -> Result<
        Self, 
        Box<dyn std::error::Error>
    > {
        // winitのコンテキストの生成
        let winit_ctx = WinitCtx {
            window: window.clone(), 
        };

        // ウィンドウサイズの取得
        let size = winit_ctx.window.inner_size();

        // WGPUのインスタンスの初期化
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(), 
            dx12_shader_compiler: Default::default(), 
        });
        
        // サーフェスの初期化
        let surface = unsafe {
            instance.create_surface(&(*winit_ctx.window))
        }?;

        // アダプタ(GPUの仮想的なインスタンス)の取得
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptionsBase { 
                power_preference: wgpu::PowerPreference::default(), 
                force_fallback_adapter: false, 
                compatible_surface: Some(&surface), 
            }
        )
            .await
            .ok_or("Adapter was not detected")?;

        // デバイスの仮想オブジェクトおよびコマンドキューの取得
        let (device, queue) =adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(), 
                limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                }, 
                label: None, 
            }, 
            None
        ).await?;

        // サーフェスの機能の取得
        let surface_caps = surface.get_capabilities(&adapter);

        // サーフェスのテクスチャフォーマットの取得
        let surface_format = surface_caps.formats.iter()
            .copied()
            .filter(|f| f.describe().srgb)
            .next()
            .map_or(
                surface_caps.formats.get(0).copied(), 
                |f| Some(f)
            )
            .ok_or("surface has not format")?;

        // サーフェスの設定
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT, 
            format: surface_format, 
            width: size.width, 
            height: size.height, 
            present_mode: wgpu::PresentMode::Fifo, 
            /*present_mode: surface_caps.present_modes.get(0)
                .copied()
                .ok_or("surface has not present mode")?, */
            alpha_mode: surface_caps.alpha_modes.get(0)
                .copied()
                .ok_or("surface has not alpha mode")?, 
            view_formats: vec![]
        };

        // サーフェスへの設定の適用
        surface.configure(&device, &config);

        // WGPUコンテキストの生成
        let wgpu_ctx = WGPUCtx {
            surface,
            device,
            queue,
            config,
        };

        // データの初期化
        let data = dinit(
            &winit_ctx, 
            &wgpu_ctx, 
        )?;

        // 処理成功
        Ok(Self {
            wgpu_ctx, 
            winit_ctx, 
            data: GCData::new(
                data, 
                dupdater, 
                dreconfigureer, 
            ), 
        })
    }

    /// 再設定
    pub fn reconfigure(
        &mut self, 
        new_size: Option<winit::dpi::PhysicalSize<u32>>, 
    ) -> Result<(), Box<dyn std::error::Error>> {
        // ウィンドウの大きさを得る
        let recfg_size = if let Some(new_size) = new_size {
            new_size
        } else {
            self.winit_ctx.window.inner_size()
        };

        // ウィンドウ内部の面積がゼロでなければウィンドウサイズの調整処理をする
        if recfg_size.width != 0 && recfg_size.height != 0 {
            self.wgpu_ctx.config.width = recfg_size.width;
            self.wgpu_ctx.config.height = recfg_size.height;
        }

        // 設定処理の実行
        self.wgpu_ctx.surface.configure(
            &self.wgpu_ctx.device, 
            &self.wgpu_ctx.config
        );

        // データの再コンフィグ
        self.data.reconfigure(
            &self.winit_ctx, 
            &self.wgpu_ctx, 
        )
    }

    /// 描画準備
    pub fn rendering<'a, FrG: super::FrameGlobal<D>>(
        &self, 
        fglob_ref: &'a FrG, 
    ) -> Result<RenderingChain<'_, 'a, D, FrG>, GfxCtxRenderingError> {
        // 出力先の初期化
        let output = self.wgpu_ctx.surface.get_current_texture()
            .map_err(|e| GfxCtxRenderingError::SurfaceError(e))?;
        let view = output.texture.create_view(&Default::default());
        self.data.update(&self.winit_ctx, &self.wgpu_ctx)
            .map_err(|e| GfxCtxRenderingError::RdrUpdateError(e))?;
        
        // チェーンを返す
        Ok(RenderingChain { 
            gfx: self, 
            output, 
            view, 
            fglob_ref, 
        })
    }
}

/// レンダラ
pub trait Renderer<GCd: Send + Sync, FrG: super::frame::FrameGlobal<GCd>> {
    fn rendering(
        &mut self, 
        output: &SurfaceTexture, 
        view: &TextureView, 
        gfx: &GfxCtx<GCd>, 
        fglob: &FrG, 
    );
}

/// 描画先を保持して描画するためのチェーン
pub struct RenderingChain<
    'a, 
    'b, 
    GCd: Send + Sync, 
    FrG: super::FrameGlobal<GCd>, 
> {
    gfx: &'a GfxCtx<GCd>, 
    output: SurfaceTexture, 
    view: TextureView, 
    fglob_ref: &'b FrG, 
}
impl<'a, 'b, GCd, FrG> RenderingChain<'a, 'b, GCd, FrG> where
    GCd: Send + Sync, 
    FrG: super::FrameGlobal<GCd>, 
{

    /// 描画ループ
    pub fn rendering(
        self, 
        renderer: &mut impl Renderer<GCd, FrG>, 
    ) -> Self {
        renderer.rendering(
            &self.output, 
            &self.view, 
            self.gfx, 
            self.fglob_ref, 
        );
        self
    }

    /// 描画
    pub(super) fn present(self) {
        self.output.present();
    }
}