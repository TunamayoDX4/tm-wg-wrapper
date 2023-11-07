use super::*;

/// アトラスを構成するメモリ
pub struct AtlasMem<const BL: usize, P> where
    P: Copy, 
{
    /// アトラスの大きさ
    pub size: types::SqSize, 

    /// アトラスの全体のテクスチャ
    pub tex: Vec<P>, 
}
impl<const BL: usize, P> AtlasMem<BL, P> where
    P: Copy, 
{
    pub fn new(
        size: types::SqSize, 
        pixel: P, 
    ) -> Self {
        let mut tex = Vec::with_capacity(size.serial() * BL);
        tex.resize(size.serial() * BL, pixel);

        Self {
            size,
            tex,
        }
    }

    pub fn raw(&self) -> &[P] { &self.tex }

    pub fn iter(&self) -> impl Iterator<Item = &[P]> {
        (0..self.size.serial())
            .map(|i| &self.tex[i * BL .. i * BL + BL])
    }
    pub fn iter_mut<'a, 'b>(&'a mut self) -> impl Iterator<Item = &mut [P]> + 'b where
        'a: 'b
    {
        (0..self.size.serial())
            .map(|
                i
            | unsafe { std::slice::from_raw_parts_mut(
                self.tex.as_mut_ptr().wrapping_add(i * BL), 
                BL
            ) })
    }

    pub fn get(&self, pos: types::SqPos) -> &[P] {
        let pos = pos.serial(self.size) * BL;
        &self.tex[pos..pos + BL]
    }

    pub fn get_obj(
        &self, 
        obj: &elem::AtlasMemParam
    ) -> impl Iterator<Item = (usize, &[P])> {
        let (w, h) = <[u32; 2]>::from(obj.size).into();
        let (sx, sy) = <[u32; 2]>::from(obj.pos).into();
        (0..h).flat_map(move |
            y
        | (0..w).map(move |x| {
                let i = y as usize * w as usize + x as usize;
                let p = (sx + x) as usize + (sy + y) as usize * self.size.w().get() as usize;
                (i, &self.tex[p * BL..p * BL + BL])
        }))
    }

    pub fn get_obj_mut<'a, 'b>(
        &'a mut self, 
        obj: &elem::AtlasMemParam
    ) -> impl Iterator<Item = (
        usize, &mut [P]
    )> + 'b where
        'a: 'b, 
    {
        let (w, h) = <[u32; 2]>::from(obj.size).into();
        let (sx, sy) = <[u32; 2]>::from(obj.pos).into();
        let size = self.size;
        (0..h).flat_map(move |
            y
        | (0..w).map(move |
            x
        | (
            y as usize * w as usize + x as usize, 
            (sx + x) as usize + (sy + y) as usize * size.w().get() as usize
        )))
        .map(|(i, p)| (
            i, 
            unsafe { std::slice::from_raw_parts_mut(
                self.tex.as_mut_ptr().wrapping_add(BL * p), 
                BL, 
            )}
        ))
    }
}