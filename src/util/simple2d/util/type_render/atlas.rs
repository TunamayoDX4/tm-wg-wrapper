use rusttype::{
    Font, 
    HMetrics, 
    Rect, 
    Scale, 
    Point, 
};
use super::super::super::{
    ImagedShared, 
    types::Texture, 
};
use crate::ctx::gfx::GfxCtx;
use super::super::atlas::{
    Atlas, 
    types::SqSize, 
};

pub struct TypeAtlas {
    atlas: super::atlas::Atlas<
        4, 
        u8, 
        char, 
        (HMetrics, Option<(Rect<i32>, Rect<f32>)>,), 
        super::BLInserter, 
    >, 
    buf: Vec<[u8; 4]>, 
    updated: bool, 
}
impl TypeAtlas {
    pub fn new(
        size: SqSize, 
    ) -> Self { Self {
        atlas: super::atlas::Atlas::new(
            size, 
            super::BLInserterInitializer, 
            255u8, 
        ).unwrap(), 
        buf: Vec::new(), 
        updated: true, 
    }}

    pub fn size(&self) -> SqSize { self.atlas.size() }
    pub fn raw(&self) -> &[u8] { self.atlas.raw() }

    pub fn get(
        &self, 
        id: usize, 
    ) -> Option<
        (&HMetrics, Option<([[f32; 2]; 2], &Rect<i32>, &Rect<f32>)>), 
    > { 
        self.atlas.get(id)
            .map(|(
                amp, 
                (hm, rect), 
                _, 
                _, 
            )| match (amp, rect.as_ref()) {
                (Some(amp), Some((rect, rect_strict))) => {
                    let wh = [
                        self.atlas.size().w().get() as f32, 
                        self.atlas.size().h().get() as f32, 
                    ];
                    let uv = [
                        [
                            *amp.pos.x() as f32 / wh[0], 
                            *amp.pos.y() as f32 / wh[1], 
                        ], [
                            (*amp.pos.x() + amp.size.w().get()) as f32 / wh[0], 
                            (*amp.pos.y() + amp.size.h().get()) as f32 / wh[1], 
                        ]
                    ];

                    (hm, Some((uv, rect, rect_strict)))
                }, 
                (None, None) => (hm, None), 
                _ => unreachable!(), 
            })
    }

    pub fn get_by_name(
        &self, 
        char: char, 
    ) -> Option<
        (&HMetrics, Option<([[f32; 2]; 2], &Rect<i32>, &Rect<f32>)>), 
    > { 
        self.atlas.get_by_name(&char)
            .map(|(
                amp, 
                (hm, rect), 
                _, 
                _, 
            )| match (amp, rect.as_ref()) {
                (Some(amp), Some((rect, rect_strict))) => {
                    let wh = [
                        self.atlas.size().w().get() as f32, 
                        self.atlas.size().h().get() as f32, 
                    ];
                    let uv = [
                        [
                            *amp.pos.x() as f32 / wh[0], 
                            *amp.pos.y() as f32 / wh[1], 
                        ], [
                            (*amp.pos.x() + amp.size.w().get()) as f32 / wh[0], 
                            (*amp.pos.y() + amp.size.h().get()) as f32 / wh[1], 
                        ]
                    ];

                    (hm, Some((uv, rect, rect_strict)))
                }, 
                (None, None) => (hm, None), 
                _ => unreachable!(), 
            })
    }

    pub fn insert_and_get(
        &mut self, 
        char: char, 
        scale: Scale, 
        font: &Font<'static>, 
    ) -> Result<
        (&HMetrics, Option<([[f32; 2]; 2], &Rect<i32>, &Rect<f32>)>), 
        super::bl_error::BLInsertError
    > {
        let glyph = font.glyph(char)
            .scaled(scale);
        let hm = glyph.h_metrics();
        let rect_strict = glyph.exact_bounding_box();
        let glyph = glyph.positioned(Point{
            x: 0., y: 0.
        });
        let rect_size = match glyph
            .pixel_bounding_box()
        {
            Some(rect) => {
                let recv = rect.max - rect.min;
                let len = recv.x as usize * recv.y as usize;
                self.buf.resize(len, [0u8; 4]);
                glyph.draw(|x, y, m| {
                    let p = x as usize + recv.x as usize * y as usize;
                    let m = (m * u8::MAX as f32).floor() as u8;
                    self.buf[p] = [0, 0, 0, m];
                });
                let size = SqSize::from([
                    std::num::NonZeroU32::new(recv.x as u32).unwrap(), 
                    std::num::NonZeroU32::new(recv.y as u32).unwrap(), 
                ]);
                Some((rect_strict.unwrap(), rect, size))
            }, 
            None => None, 
        };

        let idx = match self.atlas.insert(
            &char, 
            rect_size.map(|(_, _, s)| s)
        )? {
            (
                li, 
                Some(iter)
            ) => {
                for (idx, p) in iter {
                    self.buf[idx].iter()
                        .enumerate()
                        .for_each(|(i, px)| p[i] = *px);
                }
                let idx = li.idx();
                li.insert((
                    hm, 
                    rect_size.map(|(
                        rs, 
                        r, 
                        _s
                    )| (r, rs))
                ));
                idx
            }, 
            (li, None) => {
                let idx = li.idx();
                li.insert((
                    hm, 
                    None, 
                ));
                idx
            }, 
        };

        self.updated = true;
        Ok(self.get(idx).unwrap())
    }

    pub fn write_texture(
        &mut self, 
        texture: &mut Texture, 
        gfx: &GfxCtx, 
        shared: &ImagedShared, 
    ) {
        if self.updated {
            texture.update_image(
                gfx, 
                &shared.diffuse, 
                image::ImageBuffer::from_raw(
                    self.atlas.size().w().get(), 
                    self.atlas.size().h().get(), 
                    self.atlas.raw()
                ).unwrap()
            );
            self.updated = false;
        }
    }
}