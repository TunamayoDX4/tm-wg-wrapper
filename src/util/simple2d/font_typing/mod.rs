use hashbrown::HashMap;

/// フォントのセット
pub struct FontSet {
    /// 文字からアクセスできるフォント
    pub fonts: HashMap<char, CharModel>, 

    /// 文字データが存在しないときのためのデフォルト
    pub default: CharModel, 
}

pub mod font_type_render;
pub mod font_type_render_rendering;

/// フォント・文字列の描画構造体
pub struct FontTypeRender {
    renderer: super::img_obj::ImgObjRender, 
    font_set: FontSet, 
}

/// 文字のモデル
pub struct CharModel {
    /// テクスチャ上の座標
    pub tex_coord: [f32; 2], 

    /// テクスチャの大きさ
    pub tex_size: [f32; 2], 

    /// 整列するうえでのベースライン
    pub base_line: [f32; 2], 
}

/// 文字列表示のためのパラメータ
pub struct TypeParam<'a> {
    /// 表示する文字列
    pub s: &'a str, 

    /// 文字列の座標
    pub position: [f32; 2], 

    /// 文字列の回転角度
    pub rotation: f32, 

    /// 拡縮の比率
    pub size_ratio: [f32; 2], 

    /// 垂直方向のアラインメント
    pub align_v: TypeAlignV, 

    /// 水平方向のアラインメント
    pub align_h: TypeAlignH, 

    /// 文字列の方向
    pub direction: TypeDirection, 
}

/// 垂直方向のアラインメント
#[derive(Debug, Clone, Copy)]
pub enum TypeAlignV {
    Top, 
    Middle,  
    Bottom, 
}

/// 水平方向のアラインメント
#[derive(Debug, Clone, Copy)]
pub enum TypeAlignH {
    Left, 
    Center, 
    Right, 
}

/// 文字列の方向
#[derive(Debug, Clone, Copy)]
pub enum TypeDirection {
    /// 横書き
    Horizontal, 

    /// 縦書き
    Vertical, 
}