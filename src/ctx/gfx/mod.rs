use std::sync::Arc;
use wgpu::{
    Device, 
    Surface, 
    Queue, 
    SurfaceConfiguration, 
    SurfaceTexture, 
    TextureView, 
    SurfaceError, 
};
use winit::window::Window;

/// グラフィック機能をまとめるコンテキスト
pub struct GfxCtx {
    pub surface: Surface, 
    pub device: Device, 
    pub queue: Queue, 
    pub config: SurfaceConfiguration, 
    pub window: Arc<Window>, 
}
impl GfxCtx {
    pub async fn new(
        window: &Arc<Window>, 
    ) -> Result<
        Self, 
        Box<dyn std::error::Error>
    > {
        // ウィンドウを保持するポインタのコピー
        let window = window.clone();
        
        // ウィンドウサイズの取得
        let size = window.inner_size();

        // WGPUのインスタンスの初期化
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(), 
            dx12_shader_compiler: Default::default(), 
        });
        
        // サーフェスの初期化
        let surface = unsafe {
            instance.create_surface(&(*window))
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

        // 処理成功
        Ok(Self {
            surface,
            device,
            queue,
            config,
            window,
        })
    }

    /// 再設定
    pub fn reconfigure(
        &mut self, 
    ) {
        // ウィンドウの大きさを得る
        let inner_size = self.window.inner_size();

        // ウィンドウ内部の面積がゼロでなければウィンドウサイズの調整処理をする
        if inner_size.width != 0 && inner_size.height != 0 {
            self.config.width = inner_size.width;
            self.config.height = inner_size.height;
        }

        // 設定処理の実行
        self.surface.configure(&self.device, &self.config);
    }

    /// 描画準備
    pub fn rendering(
        &self, 
    ) -> Result<RenderingChain, SurfaceError> {
        // 出力先の初期化
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&Default::default());
        
        // チェーンを返す
        Ok(RenderingChain { ctx: self, output, view })
    }
}

/// レンダラ
pub trait Renderer {
    fn rendering(
        &mut self, 
        output: &SurfaceTexture, 
        view: &TextureView, 
        ctx: &GfxCtx, 
    );
}

/// 描画先を保持して描画するためのチェーン
pub struct RenderingChain<'a> {
    ctx: &'a GfxCtx, 
    output: SurfaceTexture, 
    view: TextureView, 
}
impl<'a> RenderingChain<'a> {

    /// 描画ループ
    pub fn rendering(
        self, 
        renderer: &mut impl Renderer, 
    ) -> Self {
        renderer.rendering(
            &self.output, 
            &self.view, 
            self.ctx
        );
        self
    }

    /// 描画
    pub(super) fn present(self) {
        self.output.present();
    }
}