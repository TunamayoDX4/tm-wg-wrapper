//! 文字列描画ライブラリ

use rusttype::{Font, Scale};
use super::super::ImagedShared;
use crate::ctx::gfx::GfxCtx;

pub mod render_core;
pub mod param;
pub mod atlas;
pub mod rendering;
pub mod draw_string;

pub mod prelude {
    use super::*;
    pub use param::{
        TypeParam, 
        TypeAlign, 
        TypeAlignH, 
        TypeAlignV, 
    };
}

use super::atlas::inserter::bl::{
    BLInserter, 
    BLInserterInitializer, 
    error as bl_error, 
};

pub struct TypeRenderer {
    font: Font<'static>, 
    scale: Scale, 
    atlas: atlas::TypeAtlas, 
    rdr: render_core::TextRender, 
}
impl TypeRenderer {
    pub fn new<GCd: Send + Sync>(
        gfx: &GfxCtx<GCd>, 
        font: Font<'static>, 
        scale: Scale, 
        atlas_size: super::atlas::types::SqSize, 
        imaged_shared: &ImagedShared, 
    ) -> Self { 
        let atlas = atlas::TypeAtlas::new(atlas_size);
        let rdr = render_core::TextRender::new(
            gfx, 
            imaged_shared, 
            image::ImageBuffer::from_raw(
                atlas.size().w().get(), 
                atlas.size().h().get(), 
                atlas.raw()
            ).unwrap(), 
        );
        Self {
            font, 
            scale, 
            atlas, 
            rdr,
        }
    }

    pub fn test_draw_char(
        &mut self, 
        char: char, 
        pos: nalgebra::Point2<f32>, 
        color: [f32; 4], 
    ) -> Result<
        (), bl_error::BLInsertError, 
    > {
        let (
            _hm, 
            uv_rect
        ) = match self.atlas.get_by_name(char) {
            Some((
                hm, 
                uv_rect, 
            )) => {
                (hm, uv_rect)
            },
            None => {
                let (
                    hm, 
                    uv_rect, 
                ) = self.atlas.insert_and_get(
                    char, 
                    self.scale, 
                    &self.font
                )?;

                (hm, uv_rect)
            },
        };
        let _vm = self.font.v_metrics(self.scale);

        match uv_rect {
            Some((uv, rect, _rect_strict)) => {
                let rv = rect.max - rect.min;
                self.rdr.push_instance(&render_core::TextInstance {
                    position: pos.into(),
                    size: [rv.x as f32, rv.y as f32],
                    rotation: 0.,
                    tex_coord: uv[0],
                    tex_size: [
                        uv[1][0] - uv[0][0], 
                        uv[1][1] - uv[0][1], 
                    ],
                    tex_rev: [false, false],
                    char_color: color,
                })
            },
            None => {},
        }

        Ok(())
    }

    pub fn draw_string(
        &mut self, 
        str: &str, 
        param: &param::TypeParam, 
    ) -> Result<
        nalgebra::Vector2<f32>, 
        bl_error::BLInsertError, 
    > {
        Ok(self.draw_string_inner(
            &self.font.v_metrics(self.scale), 
            str.split('\n'), 
            None, 
            param, 
            0., 
            0., 
            [
                param.rotation.cos(), 
                param.rotation.sin(), 
            ].into(), 
        )?.overall_size)
    }

    pub fn draw_line(
        &mut self, 
        str: &str, 
        pos: impl Into<nalgebra::Point2<f32>>, 
        color: [f32; 4], 
    ) -> Result<
        nalgebra::Vector2<f32>, 
        bl_error::BLInsertError, 
    > {
        let vm = self.font.v_metrics(self.scale);
        let pos: nalgebra::Point2<f32> = pos.into();
        
        // 列の最上辺
        let line_height = vm.ascent - vm.descent;

        let mut shift_x = pos.x;
        for char in str.chars() {
            let hm = match match self.atlas.get_by_name(
                char
            ) {
                Some(r) => r, 
                None => self.atlas.insert_and_get(
                    char, 
                    self.scale, 
                    &self.font
                )?, 
            } {
                (hm, Some((
                    uv, 
                    rect, 
                    _rect_strict, 
                ))) => {
                    // X軸上の原点
                    let origin_x = hm.left_side_bearing.abs();
                    let rv = rect.max - rect.min;
                    
                    // 水平方向の中心座標
                    let pos_x = rv.x as f32 / 2. + origin_x;
                    
                    // 文字のY軸上の大きさ
                    let bottom_line_y = (pos.y - line_height / 2.) + vm.descent;
                    let center_y = bottom_line_y + rv.y as f32 / 2. - rect.max.y as f32;

                    self.rdr.push_instance(&render_core::TextInstance {
                        position: [
                            pos_x + shift_x, 
                            center_y, 
                        ],
                        size: [
                            rv.x as f32, 
                            rv.y as f32, 
                        ],
                        rotation: 0.,
                        tex_coord: [
                            uv[0][0], 
                            uv[0][1], 
                        ],
                        tex_size: [
                            uv[1][0] - uv[0][0], 
                            uv[1][1] - uv[0][1], 
                        ],
                        tex_rev: [false, false],
                        char_color: color,
                    });

                    hm
                }, 
                (hm, None) => hm, 
            };

            let width = hm.advance_width;
            shift_x += width;
        }

        Ok([shift_x, line_height].into())
    }

    pub fn debug_save(
        &self, 
        path: impl AsRef<std::path::Path>, 
    ) -> Result<(), Box<dyn std::error::Error>> {
        let image = image::ImageBuffer::<
            image::Rgba<u8>, 
            &[u8], 
        >::from_raw(
            self.atlas.size().w().get(), 
            self.atlas.size().h().get(), 
            self.atlas.raw()
        ).ok_or("image create error!")?;
        let mut fp = std::fs::File::create(path)?;
        image.write_to(&mut fp, image::ImageFormat::Png)?;
        Ok(())
    }
}