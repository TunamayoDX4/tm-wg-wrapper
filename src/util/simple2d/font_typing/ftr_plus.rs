type PMutex<T> = parking_lot::Mutex<T>;
use super::{
    text_render::{
        TextRender, 
        TextInstance, 
    }, 
    FontSet, 
    FontTypeRenderPlus, 
    TypeParamPlus, 
    TypeAlign, 
    TypeAlignH, 
    TypeAlignV, 
    TypeDirection, 
};

/// FontTypeRenderPlusの一時値
pub struct EphemeralFTRPParam {
    /// 行データのスタック
    pub line_stack: Vec<FTRPLineParam>, 

    /// フォーマットのスタック
    pub format_stack: Vec<FTRPFormatParam>, 
}
impl EphemeralFTRPParam {
    pub fn clear(&mut self) {
        self.line_stack.clear();
        self.format_stack.clear();
    }
}

/// 行ごとのデータ
pub struct FTRPLineParam {
    /// 行方向への移動距離
    pub column_shift: f32, 

    /// 列方向の幅
    pub row_length: f32, 

    /// インデントのカウント
    pub indent_count: u32, 
}

/// フォーマットのデータ
pub struct FTRPFormatParam {
    /// テキストの色
    pub text_color: [f32; 4], 

    /// テキストサイズの比率
    pub text_size_ratio: nalgebra::Vector2<f32>, 
}

/// タグ指定機能についての実装
pub mod tag {

    pub const TAG_HEAD_KEY: char = '$';
    pub const TAG_TAIL_KEY: char = '!';
    pub const TAG_BLOCK: [char; 2] = ['<', '>'];
    pub const TAG_K_P_DELIMITER: char = '=';
    pub const TAGS: &'static [(
        &'static str, (
            &'static [&'static str], 
            &'static [(&'static str, &'static [&'static str])]
        )
    )] = &[
        ("format", (
            &[
                "form", 
            ], 
            &[
                ("color", &[
                    "c", 
                    "color", 
                ]), 
                ("size_ratio", &[
                    "s", 
                    "sr", 
                    "size", 
                    "size_r", 
                    "size_ratio"
                ]), 
            ]
        )), 
    ];
    
    type PMutex<T> = parking_lot::Mutex<T>;
    type HSHashMap<K, T> = hashbrown::HashMap<K, T>;
    type HSHashSet<K> = hashbrown::HashSet<K>;
    pub static TAG_VAR: once_cell::sync::Lazy<
        PMutex<HSHashMap<
            &'static str, 
            (
                HSHashSet<&'static str>, 
                HSHashMap<
                    &'static str, 
                    &'static [&'static str], 
                >
            )
        >>
    > = once_cell::sync::Lazy::new(|| PMutex::new(
        TAGS.iter()
            .map(|(
                key, 
                (show_name, element)
            )| (*key, {
                let show_name = show_name.iter()
                    .map(|sn| *sn)
                    .collect();
                let element = element.iter()
                    .map(|(en, e)| (*en, *e))
                    .collect();

                (show_name, element)
            }))
            .collect()
    ));
}

impl FontTypeRenderPlus {
    pub fn new(
        renderer: TextRender, 
        font_set: FontSet, 
    ) -> Self { Self {
        renderer,
        font_set,
        ephemeral: PMutex::new(EphemeralFTRPParam {
            line_stack: Vec::new(),
            format_stack: Vec::new(),
        }),
    } }
}