use std::hash::Hash;
use std::io::Read;

use wgpu::Buffer;

use super::atlas::{
    Atlas, 
    AtlasController, 
    AtlasControllerInitializer, 
    types::SqSize, 
};
use super::super::types::Texture;
use super::super::instance::{
    InstanceRaw, 
    InstanceGen, 
    buffer::InstanceArray, 
};

pub mod instance;
pub mod error;
pub mod atlas_insert;
pub mod shared;

/// アトラスの要素ごとのパラメータ
pub struct AtlasElemParam {
    /// テクスチャそのものの大きさの逆数
    pub texture_size: nalgebra::Vector2<f32>, 
    /// アトラス内部での座標および大きさ
    pub in_atras: (
        nalgebra::Point2<f32>, 
        nalgebra::Vector2<f32>, 
    ), 
}

pub struct AtlasRenderingModule<K, I> where
    K: Eq + Hash + Send + Sync + Sized + 'static, 
    I: AtlasController<
        4, 
        u8, 
        K, 
        AtlasElemParam, 
    > + Send + Sync, 
{
    atlas: Atlas<4, u8, K, AtlasElemParam, I>, 
    atlas_modified: bool, 
    texture: Texture, 
}
impl<K, I> AtlasRenderingModule<K, I> where
    K: Eq + Hash + Send + Sync + Sized + 'static, 
    I: AtlasController<
        4, 
        u8, 
        K, 
        AtlasElemParam,  
    > + Send + Sync, 
{
    pub fn new<'a, Q, P, Ii>(
        gfx_ctx: &crate::ctx::gfx::GfxCtx, 
        imaged: &super::super::ImagedShared, 
        size: SqSize, 
        inserter_initializer: Ii, 
        image: Option<impl Iterator<Item = (
            &'a Q, &'a P, 
        )>>, 
    ) -> Result<(
        Self, 
        Option<impl Iterator<Item = (&'a Q, usize)>>
    ), error::RdrInitError<
        K, I, Ii
    >> where
        Q: Eq + Hash + ?Sized + ToOwned<Owned = K> + 'a, 
        K: std::borrow::Borrow<Q>, 
        P: ?Sized + AsRef<std::path::Path> + 'a, 
        Ii: AtlasControllerInitializer<
            4, 
            u8, 
            K, 
            AtlasElemParam, 
            Initialized = I, 
        >, 
    {
        let mut atlas = Atlas::new(
            size, 
            inserter_initializer, 
            0
        ).map_err(|
            e
        | error::RdrInitError::AtlasInitError(e))?;

        let mut buf = Vec::new();
        let mut output = if image.is_some() { 
            Some(Vec::new()) 
        } else { None };
        if let Some(image) = image {
            for (name, path) in image {
                let fp = std::fs::File::open(path)
                    .map_err(
                        |e| error::RdrInitError::IOError(e)
                    )?;
                let mut br = std::io::BufReader::new(fp);
                br.read_to_end(&mut buf)
                    .map_err(
                        |e| error::RdrInitError::IOError(e)
                    )?;
                let image = image::load_from_memory(&buf)
                    .map_err(
                        |e| error::RdrInitError::OtherError(e.into())
                    )?
                    .to_rgba8();
                let id = Self::insert_atlas(
                    &mut atlas, 
                    name, 
                    image, 
                )?;
                let Some(
                    output
                ) = output.as_mut() else { continue };
                output.push((name, id));
            }
        }

        let texture = Texture::from_image(
            gfx_ctx, 
            &imaged.diffuse, 
            image::ImageBuffer::from_raw(
                atlas.size().w().get(), 
                atlas.size().h().get(), 
                atlas.raw()
            ).unwrap()
        );

        Ok((
            Self {
                atlas,
                atlas_modified: false,
                texture,
            }, 
            output.map(|e| e.into_iter())
        ))

    }

    pub fn update(
        &mut self, 
        gfx_ctx: &crate::ctx::gfx::GfxCtx, 
        imaged: &super::super::ImagedShared, 
    ) {
        if self.atlas_modified {
            self.texture.update_image(
                gfx_ctx, 
                &imaged.diffuse, 
                image::ImageBuffer::from_raw(
                    self.atlas.size().w().get(), 
                    self.atlas.size().h().get(), 
                    self.atlas.raw(), 
                ).unwrap(), 
            );
            self.atlas_modified = false;
        }
    }
}

pub struct AtlasRenderer<K, I> where
    K: Eq + Hash + Send + Sync + Sized + 'static, 
    I: AtlasController<
        4, 
        u8, 
        K, 
       AtlasElemParam, 
    > + Send + Sync, 
{
    module: AtlasRenderingModule<K, I>, 
    instances: InstanceArray<
        AtlasRenderingModule<K, I>, 
        instance::AtlasObjInstance, 
    >, 
    instance_buffer: Buffer, 
}
impl<K, I> AtlasRenderer<K, I> where
    K: Eq + Hash + Send + Sync + Sized + 'static, 
    I: AtlasController<
        4, 
        u8, 
        K, 
        AtlasElemParam, 
    > + Send + Sync, 
{
    pub fn new<'a, Q, P, Ii>(
        gfx_ctx: &crate::ctx::gfx::GfxCtx, 
        imaged: &super::super::ImagedShared, 
        size: SqSize, 
        inserter_initializer: Ii, 
        image: Option<impl Iterator<Item = (
            &'a Q, &'a P
        )>>, 
    ) -> Result<(
        Self, 
        Option<impl Iterator<Item = (&'a Q, usize)>>, 
    ), 
        error::RdrInitError<K, I, Ii>, 
    > where
        Q: Eq + Hash + ?Sized + ToOwned<Owned = K> + 'a, 
        K: std::borrow::Borrow<Q>, 
        P: ?Sized + AsRef<std::path::Path> + 'a, 
        Ii: AtlasControllerInitializer<
            4, 
            u8, 
            K, 
            AtlasElemParam, 
            Initialized = I, 
        >, 
    {
        let (
            module, 
            image, 
        ) = AtlasRenderingModule::new(
            gfx_ctx, 
            imaged, 
            size, 
            inserter_initializer, 
            image, 
        )?;
        let mut instances = InstanceArray::new();
        let instance_buffer = instances.finish(
            gfx_ctx, 
            &module
        );

        Ok((Self {
            module,
            instances,
            instance_buffer,
        }, image))
    }

    pub fn get_atlas(&self) -> &Atlas<
        4, 
        u8, 
        K, 
        AtlasElemParam, 
        I, 
    > {
        &self.module.atlas
    }

    pub fn push_instance<
        'a, 
        T: InstanceGen<AtlasRenderingModule<K, I>, instance::AtlasObjInstance>, 
    >(
        &mut self, 
        instance: &T, 
    ) {
        instance.generate(&mut self.instances)
    }
}
impl<
    K: Eq + Hash + Send + Sync + Sized + 'static, 
    I: AtlasController<
        4, 
        u8, 
        K, 
        AtlasElemParam, 
    > + Send + Sync, 
> super::super::Simple2DRender for AtlasRenderer<K, I> {
    type Shared<'a> = (
        &'a super::super::SquareShared, 
        &'a super::super::ImagedShared, 
        &'a shared::AtlasObjRenderShared, 
    );

    fn rendering<'a>(
        &mut self, 
        gfx: &crate::ctx::gfx::GfxCtx, 
        encoder: &mut wgpu::CommandEncoder, 
        view: &wgpu::TextureView, 
        camera: &crate::prelude::simple2d::shared::S2DCamera, 
        shared: Self::Shared<'a>, 
    ) {
        self.instance_buffer = self.instances.finish(
            gfx, 
            &self.module
        );

        let mut render_pass = encoder.begin_render_pass(
            &wgpu::RenderPassDescriptor {
                label: Some("render pass"), 
                color_attachments: &[Some(wgpu::RenderPassColorAttachment { 
                    view, 
                    resolve_target: None, 
                    ops: wgpu::Operations { 
                        load: wgpu::LoadOp::Load, 
                        store: true 
                    } 
                })], 
                depth_stencil_attachment: None, 
            }
        );

        render_pass.set_pipeline(&shared.2.pipeline);
        render_pass.set_bind_group(
            0, 
            &self.module.texture.bind_group, 
            &[]
        );
        render_pass.set_bind_group(1, &camera.bg, &[]);
        render_pass.set_vertex_buffer(
            0, 
            shared.0.vertex.slice(..)
        );
        render_pass.set_vertex_buffer(
            1, 
            self.instance_buffer.slice(..)
        );
        render_pass.set_index_buffer(
            shared.0.index.slice(..), wgpu::IndexFormat::Uint16
        );
        render_pass.draw_indexed(
            0..super::super::raw::INDICES.len() as _, 
            0, 
            0..self.instances.len() as _
        );
    }
}