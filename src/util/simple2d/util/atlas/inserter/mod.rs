//! インサータのよく使いそうな実装

use super::{
    AtlasController, 
    AtlasControllerInitializer, 
    memory::AtlasMem, 
    elem::{
        AtlasElem, 
        AtlasMemParam, 
    }, 
    types::{
        SqPos, 
        SqSize, 
    }, 
    super::rev_ref::RevRefContainer, 
};
pub mod bl;
pub mod prelude {
    pub use super::bl::{
        BLInserter, 
        BLInserterInitializer, 
        error as bl_error, 
    };
}