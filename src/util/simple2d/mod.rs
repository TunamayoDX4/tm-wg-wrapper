use wgpu::util::DeviceExt;

pub mod raw_param;
pub mod types;

pub mod img_obj;
pub mod img_tile;

/// カメラ
pub struct Camera {
    pub camera: types::Camera, 
    raw: raw_param::CameraRaw, 
    buffer: wgpu::Buffer, 
    bg: wgpu::BindGroup, 
}
impl Camera {
    pub fn new(
        camera: types::Camera, 
        gfx: &crate::ctx::gfx::GfxCtx, 
    ) -> Self {

        // 生のカメラ
        let raw = camera.as_raw();

        // カメラ用のバッファ
        let buffer = gfx.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("camera buffer"),
                contents: bytemuck::cast_slice(&[raw]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        // カメラのバインドグループレイアウトの生成
        let bg_layout = gfx.device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor { 
                label: Some("camera bindgroup"), 
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0, 
                        visibility: wgpu::ShaderStages::VERTEX, 
                        ty: wgpu::BindingType::Buffer { 
                            ty: wgpu::BufferBindingType::Uniform, 
                            has_dynamic_offset: false, 
                            min_binding_size: None 
                        }, 
                        count: None
                    }
                ] 
            }
        );

        // カメラのバインドグループ
        let bg = gfx.device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some("camera bindgroup"),
                layout: &bg_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: buffer.as_entire_binding(),
                    }
                ],
            }
        );

        Self {
            camera,
            raw,
            buffer,
            bg,
        }

    }

    pub fn setting(
        &mut self, 
        gfx: &crate::ctx::gfx::GfxCtx, 
    ) -> &wgpu::BindGroup {
        self.raw = self.camera.as_raw();
        gfx.queue.write_buffer(
            &self.buffer, 
            0, 
            bytemuck::cast_slice(&[self.raw])
        );
        &self.bg
    }
}