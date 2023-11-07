#[derive(Debug, Clone, Copy, Default)]
pub enum TypeAlignV {
    #[default]
    Top, 
    Middle, 
    Bottom, 
}

#[derive(Debug, Clone, Copy, Default)]
pub enum TypeAlignH {
    #[default]
    Left, 
    Center, 
    Right, 
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TypeAlign {
    pub vert: TypeAlignV, 
    pub horizon: TypeAlignH, 
}

#[derive(Debug, Clone)]
pub struct TypeParam {
    pub color: [f32; 4], 
    pub position: nalgebra::Point2<f32>, 
    pub rotation: f32, 
    pub size_ratio: nalgebra::Vector2<f32>, 
    pub align: TypeAlign, 
    pub area: Option<nalgebra::Vector2<f32>>, 
    pub enable_autoreturn: bool, 
}