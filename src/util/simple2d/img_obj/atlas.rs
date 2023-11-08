//! アトラスを用いたImgObjRender

use std::{sync::Arc, hash::Hash};
use parking_lot::RwLock;

use super::super::instance::{
    Instance, 
    InstanceModifier, 
};
use super::super::util::atlas::{
    Atlas, 
    inserter::bl::BLInserter, 
};

pub struct AtlasModifier<K: Eq + Hash + Send + Sync + 'static>(
    Arc<RwLock<Atlas<4, u8, K, ([[f32; 2]; 2], [f32; 2]), BLInserter>>>
);
impl<
    K: Eq + Hash + Send + Sync + 'static
> InstanceModifier<AtlasObjInstance> for AtlasModifier<K> {
    fn modify(
        &mut self, 
        instance: &mut AtlasObjInstance, 
    ) { 
        let lock = self.0.read();
        let (
            uv, 
            size, 
        ) = lock.get(instance.atlas_id)
            .unwrap()
            .1;

        let tc = &mut instance.imgobj.tex_coord;
        let ts = &mut instance.imgobj.tex_size;
        tc[0] = uv[0][0] + tc[0] / size[0];
        tc[1] = uv[0][1] + tc[1] / size[1];
        ts[0] /= size[0];
        ts[1] /= size[1];
    }
}

pub struct AtlasObjRenderer<K: Eq + Hash + Send + Sync + 'static> {
    atlas: Arc<RwLock<Atlas<4, u8, K, ([[f32; 2]; 2], [f32; 2]), BLInserter>>>, 
}

pub struct AtlasObjInstance {
    pub atlas_id: usize, 
    pub imgobj: super::ImgObjInstance, 
}
impl<
    K: Eq + Hash + Send + Sync + Sized
> Instance<AtlasModifier<K>> for AtlasObjInstance {
    type Raw = super::ImgObjInstanceRaw;
    type T = super::Texture;

    fn as_raw(
        self, 
        context: &mut AtlasModifier<K>, 
        value: &Self::T, 
    ) -> Self::Raw {
        todo!()
    }
}