use super::*;

/// アトラス・オブジェクトのメモリ用パラメータ
#[derive(Debug, Clone, Copy)]
pub struct AtlasMemParam {
    pub pos: types::SqPos, 
    pub size: types::SqSize, 
}

/// アトラスに挿入されているオブジェクト
pub struct AtlasElem<T, I> where
    I: Sized, 
{
    pub memp: Option<AtlasMemParam>, 
    pub ud: T, 
    pub insert_data: I, 
}