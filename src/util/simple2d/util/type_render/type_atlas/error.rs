#[derive(Debug, Clone)]
pub enum TypeAtlasInsertError {
    InsDuplicateKey, 
    InsNotEnoughSpace, 
    IsDataTooLarge, 
}
impl std::fmt::Display for TypeAtlasInsertError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { match self {
        Self::InsDuplicateKey => f.write_str("insert key duplicate"), 
        Self::InsNotEnoughSpace => f.write_str("not enough space on insert"), 
        Self::IsDataTooLarge => f.write_str("data too large"), 
    }}
}
impl std::error::Error for TypeAtlasInsertError {}

#[derive(Debug, Clone)]
pub enum TypeAtlasRemoveError {
    EntryIsNotExist, 
}