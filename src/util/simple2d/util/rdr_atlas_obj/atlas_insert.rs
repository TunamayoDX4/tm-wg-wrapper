use super::*;
impl<K, I> AtlasRenderingModule<K, I> where
    K: Eq + Hash + Send + Sync + Sized + 'static, 
    I: AtlasController<
        4, 
        u8, 
        K, 
        AtlasElemParam, 
    > + Send + Sync, 
{
    pub(super) fn insert_atlas<Q, C, Ii>(
        atlas: &mut Atlas<
            4, 
            u8, 
            K, 
            AtlasElemParam, 
            I, 
        >, 
        key: &Q, 
        image: image::ImageBuffer<
            image::Rgba<u8>, C
        >
    ) -> Result<
        usize, 
        error::RdrInitError<K, I, Ii>, 
    > where
        Q: Eq + Hash + ?Sized + ToOwned<Owned = K> + std::fmt::Debug, 
        K: std::borrow::Borrow<Q>, 
        C: std::ops::Deref<Target = [u8]>, 
        Ii: AtlasControllerInitializer<
            4, 
            u8, 
            K, 
            AtlasElemParam, 
            Initialized = I, 
        >, 
    {
        let atlas_size = atlas.size();
        let (
            lazy_inserter, 
            iter, 
        ) = atlas.insert(
            key, 
            Some(SqSize::from([
                std::num::NonZeroU32::new(image.width())
                    .ok_or(error::RdrInitError::OtherError(
                        "image width is below zero.".into()
                    ))?, 
                std::num::NonZeroU32::new(image.height())
                    .ok_or(error::RdrInitError::OtherError(
                        "image height is below zero.".into()
                    ))?, 
            ]))
        ).map_err(|
            e
        | error::RdrInitError::AtlasInsertionError(e))?;

        let id = lazy_inserter.idx();

        let amp = lazy_inserter.part().0
            .as_ref()
            .unwrap()
            .clone();
        println!("{key:?}, {amp:?}");
        lazy_inserter.insert(super::AtlasElemParam {
            texture_size: std::array::from_fn(|
                i
            | (amp.size.raw()[i].get() as f32).recip()).into(),
            in_atras: (
                std::array::from_fn(|
                    i
                | amp.pos.raw()[i] as f32 / atlas_size.raw()[i].get() as f32).into(), 
                std::array::from_fn(|
                    i
                | amp.size.raw()[i].get() as f32 / atlas_size.raw()[i].get() as f32).into(), 
            ),
        });

        for (i, pix) in iter.unwrap() {
            (0..4)
                .map(|j| (j, i * 4 + j))
                .for_each(|(j, k)| pix[j] = image.as_raw()[k])
        }

        Ok(id)
    }
}