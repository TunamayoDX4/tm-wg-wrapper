use crate::prelude::simple2d::img_obj::{ImgObjRender, ImgObjInstance};

use super::{
    FontSet, 
    FontTypeRender, 
    TypeParam, 
    TypeDirection, CharModel, 
};

impl FontTypeRender {
    pub fn new(
        font_set: FontSet, 
    ) -> Self { Self {
        font_set
    }}

    /// 文字列全体のサイズの計算
    pub fn overall_size(
        &self, 
        type_param: &TypeParam, 
    ) -> (f32, [f32; 2]) {
        let s = type_param.s.trim();
        let mut length: f32 = 0.;
        let mut width: [f32; 2] = [0., 0.];
        let (l_p, w_p) = match type_param.direction {
            TypeDirection::Horizontal => (0, 1),
            TypeDirection::Vertical => (1, 0),
        };
        if s.is_empty() { 
            // 文字列が空ならデフォルト文字の縦幅分の縦幅を計算
            let fsd = &self.font_set.default;
            let width0 = fsd.tex_size[w_p] * fsd.base_line[w_p] + 0.5;
            let width1 = fsd.tex_size[w_p] * (1. - fsd.base_line[w_p] + 0.5);
            return (0., [width0, width1]) 
        }
        let s = s.chars();
        for c in s
            .map(|c| self.font_set.fonts.get(&c))
            .map(|c| c.unwrap_or(&self.font_set.default))
        {
            length += c.tex_size[l_p] * type_param.size_ratio[l_p];
            let tex_size = c.tex_size[w_p] * type_param.size_ratio[w_p];
            width[0] = f32::max(
                width[0], 
                tex_size * (c.base_line[w_p] + 0.5)
            );
            width[1] = f32::max(
                width[1], 
                tex_size * (1. - (c.base_line[w_p] + 0.5))
            );
        }
        (length, width)
    }

    /// 文字列のモデル描画関数
    fn rendering_type_model(
        &self, 
        type_param: &TypeParam, 
        renderer: &mut ImgObjRender, 
        (l, w): (f32, [f32; 2]), 
        size: [f32; 2], 
        shift: f32, 
    ) {
        let (
            mut start_pos_shift, 
            w_shift, 
            direction_shift, 
            array_choice, 
        ) = match type_param.direction {
            TypeDirection::Horizontal => (
                match type_param.align_h {
                    super::TypeAlignH::Left => 0.,
                    super::TypeAlignH::Center => -l / 2.,
                    super::TypeAlignH::Right => -l,
                }, 
                match type_param.align_v {
                    super::TypeAlignV::Top => -size[1] + (w[1] + shift),
                    super::TypeAlignV::Middle => -size[1] / 2. + (w[1] + shift),
                    super::TypeAlignV::Bottom => w[1] + shift,
                }, 
                1., 
                [0, 1], 
            ),
            TypeDirection::Vertical => (
                match type_param.align_v {
                    super::TypeAlignV::Top => 0.,
                    super::TypeAlignV::Middle => -l / 2.,
                    super::TypeAlignV::Bottom => -l,
                }, 
                match type_param.align_h {
                    super::TypeAlignH::Left => w[1] + shift,
                    super::TypeAlignH::Center => -size[1] / 2. + (w[1] + shift),
                    super::TypeAlignH::Right => -size[1] + (w[1] + shift),
                }, 
                -1., 
                [1, 0]
            ),
        };

        for font in type_param.s.trim()
            .chars()
            .map(|c| self.font_set.fonts.get(&c))
            .map(|font| font.unwrap_or(
                &self.font_set.default)
            )
        {
            let position = Self::calc_instance_position(
                type_param, 
                array_choice, 
                font, 
                &mut start_pos_shift, 
                direction_shift, 
                w_shift
            );

            let size = [
                font.tex_size[0] * type_param.size_ratio[0], 
                font.tex_size[1] * type_param.size_ratio[1]
            ];

            // インスタンスの生成
            renderer.push_instance(&ImgObjInstance {
                position,
                size,
                rotation: type_param.rotation,
                tex_coord: font.tex_coord,
                tex_size: font.tex_size,
                tex_rev: [false, false], 
            });
        }
    }

    /// インスタンスの座標の計算
    fn calc_instance_position(
        type_param: &TypeParam, 
        array_choice: [usize; 2], 
        font: &CharModel, 
        start_pos_shift: &mut f32, 
        direction_shift: f32, 
        w_shift: f32, 
    ) -> [f32; 2] {
        let mut position = [0., 0.];

        // 進行方向へのシフト
        position[array_choice[0]] = ((
            *start_pos_shift + (
                font.tex_size[array_choice[0]] 
                * type_param.size_ratio[array_choice[0]]
            ) / 2.
        ) * direction_shift) + type_param.position[array_choice[0]];

        // シフトの計算
        *start_pos_shift += font.tex_size[array_choice[0]] 
            * type_param.size_ratio[array_choice[0]];

        // 幅方向へのシフト
        position[array_choice[1]] = (
            (
                (
                    (
                        font.tex_size[array_choice[1]] 
                        * type_param.size_ratio[array_choice[1]]
                    ) / 2.
                ) * -font.base_line[array_choice[1]]
            ) + type_param.position[array_choice[1]]
        ) + w_shift;

        // 回転
        if f32::EPSILON < type_param.rotation.abs() {[
            position[0] * type_param.rotation.cos()
            - position[1] * type_param.rotation.sin(), 
            position[0] * type_param.rotation.sin()
            + position[1] * type_param.rotation.cos()
        ]} else {
            position
        }
    }

    /// 描画関数の内部実装
    /// 再帰を使って改行を効率的にレンダリングしてみます
    fn rendering_inner(
        &self, 
        type_param: &TypeParam, 
        renderer: &mut ImgObjRender, 
        size: [f32; 2], 
    ) -> ([f32; 2], f32) {
        // 改行での文字列の分割を試みる。
        match type_param.s.split_once('\n')
            .map(|(s0, s1)| (
                TypeParam {
                    s: s0,
                    position: type_param.position,
                    rotation: type_param.rotation,
                    size_ratio: type_param.size_ratio,
                    align_v: type_param.align_v,
                    align_h: type_param.align_h,
                    direction: type_param.direction,
                }, 
                TypeParam {
                    s: s1,
                    position: type_param.position,
                    rotation: type_param.rotation,
                    size_ratio: type_param.size_ratio,
                    align_v: type_param.align_v,
                    align_h: type_param.align_h,
                    direction: type_param.direction,
                }
            ))
        {
            None => {
                // type_paramの文字列全体の大きさ
                let oa_size = self.overall_size(&type_param);
                
                // type_paramの文字列全体の縦の大きさ
                let oa_size_v = oa_size.1[0] + oa_size.1[1];

                // type_paramの大きさと、渡された合計サイズの加算および最大幅の計算
                let size = [
                    oa_size.0.max(size[0]), 
                    oa_size_v + size[1]
                ];

                // テキストの描画
                self.rendering_type_model(
                    type_param, 
                    renderer, 
                    oa_size, 
                    size, 
                    0., 
                );

                (size, oa_size_v)
            }, 
            Some((s0, s1)) => {
                // 改行前の文章の文字列全体の大きさ
                let oa_size = self.overall_size(&s0);
                
                // 改行前の文章の文字列全体の縦の大きさ
                let oa_size_v = oa_size.1[0] + oa_size.1[1];

                // 改行前の文章の大きさと、渡された合計サイズの加算および最大幅の計算
                let size = [
                    oa_size.0.max(size[0]), 
                    oa_size_v + size[1]
                ];

                // 改行後の文章の描画
                let (size, shift_up) = self.rendering_inner(
                    &s1, 
                    renderer, 
                    size, 
                );
                
                // 改行前のテキストの描画
                self.rendering_type_model(
                    &s0, 
                    renderer, 
                    oa_size, 
                    size, 
                    shift_up, 
                );

                (size, shift_up + oa_size_v)
            }
        }
    }

    /// 描画
    pub fn rendering(
        &self, 
        type_param: &TypeParam, 
        renderer: &mut ImgObjRender, 
    ) -> (f32, f32) {
        let size = self.rendering_inner(
            type_param, 
            renderer, 
            [0., 0.]
        ).0;
        (size[0], size[1])
    }
}