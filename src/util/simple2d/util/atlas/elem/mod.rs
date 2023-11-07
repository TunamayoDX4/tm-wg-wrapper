use super::*;

pub mod container;

/// アトラス・オブジェクトのメモリ用パラメータ
#[derive(Debug, Clone, Copy)]
pub struct AtlasMemParam {
    pub pos: types::SqPos, 
    pub size: types::SqSize, 
}

/// アトラスに挿入されているオブジェクト
pub struct AtlasElem<K, T, I> where
    K: Eq + Hash, 
    I: Sized, 
{
    pub memp: Option<AtlasMemParam>, 
    pub ud: T, 
    pub key: K, 
    pub insert_data: I, 
}

/// アトラスに挿入されているオブジェクトの詳細データを保持するメモリ
pub struct AtlasElemMem<K, T, I> where
    K: Eq + Hash, 
    I: Sized, 
{
    mem: Vec<Option<AtlasElem<K, T, I>>>, 
    uu_mem: VecDeque<usize>, 
    table: HashMap<K, usize>, 
}