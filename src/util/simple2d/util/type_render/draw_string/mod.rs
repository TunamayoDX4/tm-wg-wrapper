use std::str::Split;

use rusttype::VMetrics;
use super::{
    type_atlas::error::TypeAtlasInsertError, 
    param::{
        TypeAlignH, 
        TypeAlignV, 
        TypeParam
    }, render_core::TextInstance, 
};

pub struct DrawStringInnerSuccess {
    pub overall_size: nalgebra::Vector2<f32>, 
    pub overall_shift: nalgebra::Vector2<f32>, 
}

impl super::TypeRenderer {
    pub(super) fn draw_string_inner(
        &mut self, 
        vm: &VMetrics, 
        mut splstr: Split<char>, 
        auto_returned: Option<&str>, 
        param: &TypeParam, 
        column_shift: f32, 
        row_size: f32, 
        rotation: nalgebra::Vector2<f32>, 
    ) -> Result<
        DrawStringInnerSuccess, 
        TypeAtlasInsertError, 
    > {
        let str = if let Some(
            auto_returned
        ) = auto_returned { auto_returned }
        else { match splstr.next() {
            Some(str) => str, 
            None => return Ok(DrawStringInnerSuccess { 
                overall_size: [
                    row_size, 
                    column_shift, 
                ].into(), 
                overall_shift: {
                    // ブロック全体の座標シフト
                    let shift = nalgebra::Vector2::from([
                        match param.align.horizon {
                            TypeAlignH::Left => 0.,
                            TypeAlignH::Center => -row_size / 2.,
                            TypeAlignH::Right => -row_size,
                        } * param.size_ratio.x, 
                        match param.align.vert {
                            TypeAlignV::Top => -column_shift,
                            TypeAlignV::Middle => -column_shift / 2.,
                            TypeAlignV::Bottom => 0.,
                        }, 
                    ]);
                    nalgebra::Vector2::from([
                        shift.x * rotation.x - shift.y * rotation.y, 
                        shift.x * rotation.y + shift.y * rotation.x, 
                    ])
                }
            })
        } };

        let (
            auto_returned, 
            size, 
        ) = self.calc_size(vm, str, param)?;
        
        //println!("{str}, {auto_returned:?}");

        // 再帰呼び出し
        let overall = self.draw_string_inner(
            vm, 
            splstr, 
            auto_returned.map(|(s, _)| s), 
            param, 
            column_shift + size.y * param.size_ratio.y, 
            row_size.max(size.x), 
            rotation, 
        )?;

        // 本描画
        self.draw_string_line(
            vm, 
            str, 
            param, 
            nalgebra::Vector2::from([
                0., 
                overall.overall_size.y - column_shift, 
            ]), 
            overall.overall_shift, 
            rotation, 
            auto_returned.map(|(_, i)| i), 
            overall.overall_size, 
            size.x, 
        );

        Ok(overall)
    }

    fn draw_string_line(
        &mut self, 
        vm: &VMetrics, 
        str: &str, 
        param: &TypeParam, 
        shift: nalgebra::Vector2<f32>, 
        overall_shift: nalgebra::Vector2<f32>, 
        rotation: nalgebra::Vector2<f32>, 
        clipped: Option<usize>, 
        overall_size: nalgebra::Vector2<f32>, 
        row_size: f32, 
    ) {
        let line_height = (vm.ascent - vm.descent) * param.size_ratio.y;
        let base_pos = param.position + shift;
        let bottom_line_y = (
            base_pos.y - line_height / 2.
        ) + vm.descent * param.size_ratio.y;
        let mut shift_x = (base_pos.x + match param.align.horizon {
            TypeAlignH::Left => 0.,
            TypeAlignH::Center => (overall_size.x - row_size) / 2.,
            TypeAlignH::Right => overall_size.x - row_size,
        }) * param.size_ratio.x;
        let str = match clipped {
            None => str, 
            Some(clip) => &str[..clip], 
        };
        for char in str.chars() {
            let hm = match self.atlas.get_by_name(char).unwrap() {
                (
                    hm, 
                    Some((
                        uv, 
                        rect, 
                        _rect_strict, 
                    ))
                ) => {
                    let rv = rect.max - rect.min;
                    let origin_x = hm.left_side_bearing.abs();
                    let pos = [
                        shift_x + (rv.x as f32 / 2. + origin_x) * param.size_ratio.x, 
                        bottom_line_y + (
                            rv.y as f32 / 2. 
                            - rect.max.y as f32
                        ) * param.size_ratio.y, 
                    ];
                    let pos = [
                        pos[0] * rotation.x - pos[1] * rotation.y, 
                        pos[0] * rotation.y + pos[1] * rotation.x, 
                    ];

                    self.rdr.push_instance(&TextInstance {
                        position: [
                            pos[0] + overall_shift.x, 
                            pos[1] + overall_shift.y, 
                        ],
                        size: [
                            rv.x as f32 * param.size_ratio.x, 
                            rv.y as f32 * param.size_ratio.y,  
                        ],
                        rotation: param.rotation,
                        tex_coord: [
                            uv[0][0], 
                            uv[0][1], 
                        ],
                        tex_size: [
                            uv[1][0] - uv[0][0], 
                            uv[1][1] - uv[0][1], 
                        ],
                        tex_rev: [false, false],
                        char_color: param.color,
                    });

                    hm
                }, 
                (hm, None) => hm, 
            };
            shift_x += hm.advance_width * param.size_ratio.x;
        }
    }

    fn calc_size<'a>(
        &mut self, 
        vm: &VMetrics, 
        str: &'a str, 
        param: &TypeParam, 
    ) -> Result<
        (
            Option<(&'a str, usize)>, 
            nalgebra::Vector2<f32>, 
        ), 
        TypeAtlasInsertError, 
    > {
        let line_height = vm.ascent - vm.descent;
        let mut shift_x = 0.;
        for (idx, char) in str.char_indices() {
            let hm = match self.atlas.get_by_name(
                char
            ) {
                Some((hm, _)) => hm, 
                None => self.atlas.insert_and_get(
                    char, 
                    self.scale, 
                    &self.font
                )?.0
            };
            let shift_x_extended = shift_x + hm.advance_width;
            match (param.area, param.enable_autoreturn) {
                (
                    Some(
                        area
                    ), 
                    true, 
                ) => if area.x <= shift_x_extended {
                    return Ok((
                        Some((&str[idx..], idx)), 
                        [
                            shift_x, 
                            line_height, 
                        ].into(), 
                    ))
                }, 
                _ => {}, 
            };
            shift_x = shift_x_extended;
        }
        Ok((None, [shift_x, line_height].into()))
    }
}