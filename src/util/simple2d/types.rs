use std::io::Read;
use wgpu::BindGroup;

/// 視認可能な範囲
pub struct VisibleField {
    cam_pos: nalgebra::Point2<f32>, 
    cam_res: nalgebra::Vector2<f32>, 
}
impl VisibleField {
    pub fn new(
        camera: &Camera, 
    ) -> Self { Self {
        cam_pos: camera.position, 
        cam_res: camera.size, 
    }}

    pub fn in_visible<P, S>(
        &self, 
        pos: P, 
        size: S, 
    ) -> bool where
        nalgebra::Point2<f32>: From<P>, 
        nalgebra::Vector2<f32>: From<S>, 
    {
        let distance: nalgebra::Vector2<f32> = (nalgebra::Point2::from(pos) - self.cam_pos).abs();
        let diff: nalgebra::Vector2<f32> = self.cam_res * 0.5 + nalgebra::Vector2::from(size);
        distance <= diff
    }

    pub fn visible_area(&self) -> [nalgebra::Point2<f32>; 2] { [
        self.cam_pos - self.cam_res * 0.5, 
        self.cam_pos + self.cam_res * 0.5, 
    ]}

    pub fn visible_edge(&self) -> [nalgebra::Point2<f32>; 4] { [
        self.cam_pos + nalgebra::Vector2::new(-self.cam_res.x, -self.cam_res.y) * 0.5, 
        self.cam_pos + nalgebra::Vector2::new(self.cam_res.x, -self.cam_res.y) * 0.5, 
        self.cam_pos + nalgebra::Vector2::new(-self.cam_res.x, self.cam_res.y) * 0.5, 
        self.cam_pos + nalgebra::Vector2::new(self.cam_res.x, self.cam_res.y) * 0.5, 
    ]}
}

/// カメラ
pub struct Camera {
    pub position: nalgebra::Point2<f32>, 
    pub size: nalgebra::Vector2<f32>, 
    pub zoom: f32, 
    pub rotation: f32, 
}
impl Camera {
    pub fn as_raw(&self) -> super::raw_param::CameraRaw {
        let position = self.position.into();
        let size = std::array::from_fn(
            |i| (self.size[i] * 0.5).recip() * self.zoom
        );
        let rotation = [
            (-self.rotation).cos(), 
            (-self.rotation).sin(), 
        ];
        super::raw_param::CameraRaw { 
            position, 
            size, 
            rotation, 
            _dummy: Default::default() 
        }
    }
}

/// テクスチャ
pub struct Texture {
    pub bind_group: BindGroup, 
    pub texture_size: [f32; 2], 
}
impl Texture {
    pub fn new(
        gfx: &crate::ctx::gfx::GfxCtx, 
        bind_group_layout: &wgpu::BindGroupLayout, 
        path: impl AsRef<std::path::Path>, 
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // 画像の読み込み・インスタンス生成
        let diffuse_rgba8 = {
            let mut file = std::fs::File::open(path)?;
            let mut diffuse_bytes = Vec::new();
            file.read_to_end(&mut diffuse_bytes)?;
            image::load_from_memory(&diffuse_bytes)?
                .to_rgba8()
        };

        // 画像の大きさ情報の取得
        let dimensions = diffuse_rgba8.dimensions();

        // GPUで画像を扱うための大きさの情報を初期化
        let texture_size = wgpu::Extent3d {
            width: dimensions.0, 
            height: dimensions.1, 
            depth_or_array_layers: 1, 
        };

        // 拡散テクスチャのインスタンス生成
        let diffuse_texture = gfx.device.create_texture(
            &wgpu::TextureDescriptor {
                size: texture_size, 
                mip_level_count: 1, 
                sample_count: 1, 
                dimension: wgpu::TextureDimension::D2, 
                format: wgpu::TextureFormat::Rgba8UnormSrgb, 
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST, 
                label: Some("diffuse_texture"), 
                view_formats: &[], 
            }
        );

        // GPUのキューにテクスチャ情報を書き込む
        gfx.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &diffuse_texture, 
                mip_level: 0, 
                origin: wgpu::Origin3d::ZERO, 
                aspect: wgpu::TextureAspect::All, 
            }, 
            &diffuse_rgba8, 
            wgpu::ImageDataLayout { 
                offset: 0, 
                bytes_per_row: std::num::NonZeroU32::new(4 * dimensions.0), 
                rows_per_image: std::num::NonZeroU32::new(dimensions.1) 
            }, 
            texture_size
        );

        // ビューの作成
        let diffuse_texture_view = diffuse_texture.create_view(
            &wgpu::TextureViewDescriptor::default()
        );

        // サンプラの作成
        let diffuse_sampler = gfx.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge, 
            address_mode_v: wgpu::AddressMode::ClampToEdge, 
            address_mode_w: wgpu::AddressMode::ClampToEdge, 
            mag_filter: wgpu::FilterMode::Nearest, 
            min_filter: wgpu::FilterMode::Nearest, 
            mipmap_filter: wgpu::FilterMode::Nearest, 
            ..Default::default()
        });

        // バインドグループの作成
        let diffuse_bind_group = gfx.device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: bind_group_layout, 
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0, 
                        resource: wgpu::BindingResource::TextureView(&diffuse_texture_view), 
                    }, 
                    wgpu::BindGroupEntry {
                        binding: 1, 
                        resource: wgpu::BindingResource::Sampler(&diffuse_sampler), 
                    }, 
                ], 
                label: Some("diffuse bind group"), 
            }
        );

        Ok(Self { 
            bind_group: diffuse_bind_group, 
            texture_size: [dimensions.0 as f32, dimensions.1 as f32], 
        })
    }
}

/// インスタンスを生成しうる構造体
pub trait InstanceGen<I: Instance>: Send + Sync + 'static {
    fn generate(&self) -> I;
}

/// インスタンス
pub trait Instance: InstanceGen<Self> + Send + Sync + Sized + Copy {
    type Raw: super::raw_param::InstanceRaw;
    type Arv;
    fn as_raw(&self, v: &Self::Arv) -> Self::Raw;
}