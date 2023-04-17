//! テキスト用のレンダラ

/// テキストレンダラのフォントモジュールの共有構造体
#[derive(Clone)]
pub struct TextRendererGMArc(std::sync::Arc<
    parking_lot::Mutex<wgpu_glyph::GlyphBrush<()>>
>);
impl TextRendererGMArc {
    pub fn new(
        gfx: &crate::ctx::gfx::GfxCtx, 
        ttf_bytes: Vec<u8>, 
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let font = wgpu_glyph::ab_glyph::FontArc::try_from_vec(ttf_bytes)?;

        let glyph_brush = wgpu_glyph::GlyphBrushBuilder::using_font(font)
            .build(&gfx.device, gfx.config.format);

        Ok(Self(std::sync::Arc::new(
            parking_lot::Mutex::new(glyph_brush)
        )))
    }
}

/// テキスト用レンダラ
pub struct TextRenderer {
    staging_belt: wgpu::util::StagingBelt, 
    glyph: TextRendererGMArc, 
}
impl TextRenderer {
    pub fn new(
        glyph: TextRendererGMArc, 
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let staging_belt = wgpu::util::StagingBelt::new(1024);

        Ok(Self {
            staging_belt,
            glyph,
        })
    }

    pub fn set_glyph(&mut self, glyph: TextRendererGMArc) {
        self.glyph = glyph
    }

    pub fn get_glyph(&self) -> TextRendererGMArc {
        self.glyph.clone()
    }

    pub fn rendering <'a, S: 'a> (
        &mut self, 
        gfx: crate::ctx::gfx::GfxCtx, 
        encoder: &mut wgpu::CommandEncoder, 
        view: &wgpu::TextureView, 
        sections: impl IntoIterator<Item = S>, 
    ) where
        S: Into<std::borrow::Cow<'a, wgpu_glyph::Section<'a>>>, 
    { 
        self.staging_belt.recall();
        let mut glyph = self.glyph.0.lock();
        sections.into_iter()
            .for_each(|section| glyph.queue(section));

        glyph.draw_queued(
            &gfx.device, 
            &mut self.staging_belt, 
            encoder, 
            view, 
            gfx.config.width, 
            gfx.config.height
        ).expect("text draw queued");

        self.staging_belt.finish();
    }
}