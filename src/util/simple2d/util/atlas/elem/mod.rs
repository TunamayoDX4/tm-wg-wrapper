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
impl<T: Sized, I> From<(
    (
        Option<AtlasMemParam>, 
        I, 
    ), 
    T, 
)> for AtlasElem<T, I> {
    fn from(value: (
        (
            Option<AtlasMemParam>, 
            I, 
        ), 
        T, 
    )) -> Self { Self {
        memp: value.0.0,
        ud: value.1,
        insert_data: value.0.1,
    }}
}