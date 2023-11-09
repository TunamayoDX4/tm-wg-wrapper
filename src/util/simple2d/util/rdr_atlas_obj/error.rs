use std::fmt::Debug;

use super::*;

#[derive(Debug)]
pub enum RdrInitError<K, I, Ii> where
    K: Eq + Hash + Send + Sync + Sized + 'static, 
    I: AtlasController<
        4, 
        u8, 
        K, 
        AtlasElemParam, 
    >, 
    Ii: AtlasControllerInitializer<
        4, 
        u8, 
        K, 
        AtlasElemParam, 
        Initialized = I, 
    >, 
{
    IOError(std::io::Error), 
    AtlasInitError(Ii::InitError), 
    AtlasInsertionError(I::InsertError), 
    OtherError(Box<dyn std::error::Error>), 
}
impl<K, I, Ii> std::fmt::Display for RdrInitError<K, I, Ii> where
    K: Eq + Hash + Send + Sync + Sized + 'static, 
    I: AtlasController<
        4, 
        u8, 
        K, 
        AtlasElemParam, 
    >, 
    Ii: AtlasControllerInitializer<
        4, 
        u8, 
        K, 
        AtlasElemParam, 
        Initialized = I, 
    >, 
{
    fn fmt(
        &self, 
        f: &mut std::fmt::Formatter<'_>, 
    ) -> std::fmt::Result { match self {
        RdrInitError::IOError(io) => f.write_fmt(format_args!(
            "io error: {io}"
        )),
        RdrInitError::AtlasInitError(
            aie
        ) => f.write_fmt(format_args!(
            "atlas initializing process error: {aie}"
        )),
        RdrInitError::AtlasInsertionError(
            e
        ) => f.write_fmt(format_args!(
            "atlas insertion error: {e}"
        )), 
        RdrInitError::OtherError(
            e
        ) => f.write_fmt(format_args!(
            "other error: {e}"
        ))
    }}
}
impl<K, I, Ii> std::error::Error for RdrInitError<K, I, Ii> where
    K: Eq + Hash + Send + Sync + Sized + Debug + 'static, 
    I: AtlasController<
        4, 
        u8, 
        K, 
        AtlasElemParam, 
    > + Debug, 
    Ii: AtlasControllerInitializer<
        4, 
        u8, 
        K, 
        AtlasElemParam, 
        Initialized = I, 
    > + Debug, 
{}