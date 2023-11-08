//! シンプル2Dレンダラ用のユーティリティ

pub mod rev_ref;
pub mod atlas;
pub mod type_render;

pub mod prelude {
    pub use super::{
        atlas::{
            Atlas, 
            AtlasController, 
            AtlasControllerInitializer, 
            types::{SqPos, SqSize}, 
            inserter::prelude::*, 
        }, 
        rev_ref::RevRefContainer, 
        type_render::prelude::*, 
    };
}