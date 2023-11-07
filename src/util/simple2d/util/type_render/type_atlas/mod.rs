use std::{collections::BTreeMap, num::NonZeroU32};

use super::super::atlas::{
    types::{SqSize, SqPos}, 
    memory::AtlasMem, 
    elem::{AtlasElem, AtlasMemParam}, 
    AtlasController, 
    AtlasControllerInitializer, 
};
use super::super::rev_ref::RevRefContainer;

pub mod error;
use error::{
    TypeAtlasInsertError, 
    TypeAtlasRemoveError, 
};

pub struct TypeAtlasCtrlInitializer;
impl AtlasControllerInitializer<4, u8, char, (
    rusttype::HMetrics, 
    Option<(
        rusttype::Rect<i32>, 
        rusttype::Rect<f32>, 
    )>, 
)> for TypeAtlasCtrlInitializer {
    type Initialized = TypeAtlasControl;
    type InitError = ();

    fn initialize(
        self, 
        size: SqSize, 
        _memory: &mut AtlasMem<4, u8>, 
    ) -> Result<
        Self::Initialized, 
        Self::InitError
    > { Ok(TypeAtlasControl {
        unuse_flag: {
            let mut vec = Vec::with_capacity(size.serial());
            vec.resize(size.serial(), 0);
            vec
        },
        base_line: {
            let mut bl = BTreeMap::new();
            bl.insert(0, None);
            bl
        },
    })}
}

//#[derive(Debug)]
pub struct TypeAtlasControl {
    unuse_flag: Vec<u8>, 
    base_line: BTreeMap<u32, Option<Vec<(u32, NonZeroU32)>>>, 
}
impl std::fmt::Debug for TypeAtlasControl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TypeAtlasControl")
            .field("unuse_flag\n", &self.unuse_flag)
            .field("base_line", &self.base_line)
            .finish()
    }
}
impl AtlasController<4, u8, char, (
    rusttype::HMetrics, 
    Option<(
        rusttype::Rect<i32>, 
        rusttype::Rect<f32>, 
    )>, 
)> for TypeAtlasControl {
    type InsertError = TypeAtlasInsertError;
    type RemoveError = TypeAtlasRemoveError;
    type ControllerElemData = ();

    fn insert<Q: Eq + std::hash::Hash + ?Sized + ToOwned<Owned = char>>(
        &mut self, 
        atlas: &mut AtlasMem<4, u8>, 
        elem: &mut RevRefContainer<
            char, 
            AtlasElem<
                (
                    rusttype::HMetrics, 
                    Option<(
                        rusttype::Rect<i32>, 
                        rusttype::Rect<f32>, 
                    )>
                ), 
                Self::ControllerElemData, 
            >
        >, 
        key: &Q, 
        size: Option<SqSize>, 
        ud: (
            rusttype::HMetrics, 
            Option<(
                rusttype::Rect<i32>, 
                rusttype::Rect<f32>, 
            )>, 
        ), 
    ) -> Result<
        (usize, Option<AtlasMemParam>), 
        Self::InsertError
    > where
        char: std::borrow::Borrow<Q>
    { match size {
        Some(size) => { if atlas.size.w() < size.w() 
            && atlas.size.h() < size.h() 
        {
            return Err(TypeAtlasInsertError::IsDataTooLarge)
        } else {
            let pos = self.seek_baseline(
                atlas, 
                size
            )?;

            let memp = AtlasMemParam {
                pos, 
                size, 
            };

            let id = elem.insert(
                key, 
                AtlasElem { 
                    memp: Some(memp), 
                    ud, 
                    insert_data: () 
                }
            ).map_err(|_| TypeAtlasInsertError::InsDuplicateKey)?;

            // フラグの更新
            for y in (0..size.h().get())
                .map(|y| y + pos.y())
            {
                let y_ser = y as usize * atlas.size.w().get() as usize;
                for x in 0..size.w().get() {
                    self.unuse_flag[
                        x as usize + *pos.x() as usize + y_ser
                    ] = u8::try_from(
                        x + 1
                    ).unwrap_or(u8::MAX);
                }
            }

            // ベースラインの更新
            self.base_line.entry(*pos.y() + size.h().get() + 1)
                .and_modify(|bl| match bl {
                    v @ None => *v = Some(Vec::from([
                        (0, NonZeroU32::new(1).unwrap()), 
                        (*pos.x(), *size.w()), 
                        (u32::MAX, NonZeroU32::new(u32::MAX).unwrap()), 
                    ])), 
                    Some(v) => {
                        let tail = v.pop().unwrap();
                        let mut i = v.len();
                        v.push((
                            *pos.x(), *size.w()
                        ));
                        v.push(tail);

                        // ノームソート
                        loop {
                            if v[i - 1] < v[i] { break }
                            v.swap(i - 1, i);
                            i -= 1;
                        }
                    }, 
                })
                .or_insert(Some(Vec::from([
                    (0, NonZeroU32::new(1).unwrap()), 
                    (*pos.x(), *size.w()), 
                    (u32::MAX, NonZeroU32::new(u32::MAX).unwrap()), 
                ])));

            Ok((id, Some(memp)))
        }}, 
        None => {
            Ok((
                elem.insert(
                    key, 
                    AtlasElem { 
                        memp: None, 
                        ud, 
                        insert_data: () 
                    }
                ).map_err(|_| TypeAtlasInsertError::InsDuplicateKey)?, 
                None
            ))
    }}}

    fn remove(
        &mut self, 
        atlas: &mut AtlasMem<4, u8>, 
        elem: &mut RevRefContainer<
            char, 
            AtlasElem<
                (
                    rusttype::HMetrics, 
                    Option<(
                        rusttype::Rect<i32>, 
                        rusttype::Rect<f32>, 
                    )>
                ), 
                Self::ControllerElemData, 
            >
        >, 
        id: usize, 
    ) -> Result<(
            (
                rusttype::HMetrics, 
                Option<(
                    rusttype::Rect<i32>, 
                    rusttype::Rect<f32>, 
                )>, 
            ), 
            char, 
            Option<AtlasMemParam>, 
        ), 
        Self::RemoveError
    > { match elem.remove(id) {
        Some((
            k, 
            AtlasElem { 
                memp: Some(amp), 
                ud: d, 
                insert_data: _, 
            }
        )) => {
            let size = &amp.size;
            let pos = &amp.pos;
            // フラグの更新
            for y in (0..size.h().get())
                .map(|y| y + pos.y())
            {
                let y_ser = y as usize * atlas.size.w().get() as usize;
                for x in 0..size.w().get() {
                    self.unuse_flag[
                        x as usize + *pos.x() as usize + y_ser
                    ] = 0;
                }
            }

            // ベースラインの更新
            let p = *pos.y() + size.h().get() + 1;
            if match self.base_line.get_mut(&p)
            {
                v @ Some(&mut Some(_)) => {
                    let v = v.unwrap();
                    let vu = v.as_mut().unwrap();
                    let mut i = 1;
                    let mut k = 0;
                    while vu[i].0 != u32::MAX {
                        if vu[i].0 == *pos.x() {
                            k += 1;
                        }
                        vu.swap(i, i + k);
                        i += 1;
                    }
                    // 要素の末尾を捨てる
                    vu.pop();

                    // 要素が空になってたら消す
                    vu.len() == 2
                }, 
                _ => unreachable!(), 
            } {
                self.base_line.remove(&p).unwrap();
            }

            Ok((d, k, Some(amp)))
        }, 
        Some((
            k, 
            AtlasElem { 
                memp: None, 
                ud: d, 
                insert_data: _ 
            }
        )) => Ok((d, k, None)), 
        None => Err(TypeAtlasRemoveError::EntryIsNotExist)
    } }
}

impl TypeAtlasControl {
    /*
    pub fn seek_bl(
        &self, 
        atlas: &AtlasMem<4, u8>, 
        object_size: SqSize, 
    ) -> Result<SqPos, TypeAtlasInsertError> {
        let default_ary = &[
            (0u32, NonZeroU32::new(1).unwrap()), 
            (1u32, *atlas.size.w()), 
            (u32::MAX, NonZeroU32::new(u32::MAX).unwrap()), 
        ];
        'blseek_y: for(
            bl_y, 
            x_bl_arr, 
        ) in self.base_line.iter()
            .map(|(
                bl_y, x_bl_arr, 
            )| (
                *bl_y, 
                x_bl_arr.as_ref()
                    .map(|v| v.as_slice())
                    .unwrap_or(default_ary)
            ))
        {
            if atlas.size.h().get() - bl_y < object_size.h().get() {
                return Err(TypeAtlasInsertError::InsNotEnoughSpace)
            }

            // x seek head
            let mut xskh = 0u32;

            // オブジェクトのX方向へのシーク
            for (bl_x, seek_width) in x_bl_arr.iter()
            {
                if atlas.size.w().get() - bl_x < object_size.w().get() { 
                    continue 'blseek_y 
                }

                match self.seek_obj(
                    object_size, 
                    [
                        *bl_x, 
                        bl_y
                    ].into(), 
                    xskh.checked_sub(object_size.w().get()).unwrap_or(0), 
                    seek_width.checked_add(*bl_x)
                        .unwrap_or(*atlas.size.w()), 
                ) {
                    Ok(x) => return Ok([
                        x, 
                        bl_y
                    ].into()), 
                    Err(e) => xskh = e, 
                }
            }
        }

        Err(TypeAtlasInsertError::InsNotEnoughSpace)
    }

    pub fn seek_obj(
        &self, 
        object_size: SqSize, 
        base_line: SqPos, 
        mut seek_head
        seek_area: (u32, NonZeroU32), 
        mut seek_head: u32, 
        seek_tail: NonZeroU32, 
    ) -> Result<u32, u32> {
        'base_seek: while seek_head < seek_tail.get() {
            
            let mut shift_x = 0;

            seek_head += shift_x;
        }

        Err(0)
    }
     */


    pub fn seek_baseline(
        &self, 
        atlas: &AtlasMem<4, u8>, 
        obj_size: SqSize, 
    ) -> Result<SqPos, TypeAtlasInsertError> {

        // デフォルトの走査配列
        // アトラスの幅分。
        let defarray = &[
            (0u32, NonZeroU32::new(1).unwrap()), 
            (1u32, *atlas.size.w()), 
            (u32::MAX, NonZeroU32::new(u32::MAX).unwrap()), 
        ];

        // Y方向のベースライン走査
        'blseek_y: for(
            bl_y, 
            eids, 
        ) in self.base_line.iter()
            .map(|(bl_y, eids)| (
                *bl_y, eids.as_ref().map(|v| v.as_slice())
            ))
            .map(|(bl_y, eids)| (
                bl_y, 
                eids.unwrap_or(defarray), 
            ))
        {
            if atlas.size.h().get() - bl_y < obj_size.h().get() {
                return Err(TypeAtlasInsertError::InsNotEnoughSpace)
            }

            let mut seeked_tail_x = 0;

            // X方向のベースライン走査
            for (bl_x_l, bl_x_r) in (1..eids.len() - 1)
                .map(|i| eids[i]) 
            {
                if atlas.size.w().get() - bl_x_l < obj_size.w().get() {
                    continue 'blseek_y
                }

                match self.seek_object(
                    atlas, 
                    obj_size, 
                    [
                        bl_x_l.max(seeked_tail_x), 
                        bl_y, 
                    ].into(), 
                    bl_x_r.checked_add(bl_x_l).unwrap_or(*atlas.size.w())
                ) {
                    Ok(pos) => return Ok(pos), 
                    Err(tail_x) => {
                        seeked_tail_x = tail_x
                    }
                }
            }
        }

        Err(TypeAtlasInsertError::InsNotEnoughSpace)
    }

    pub fn seek_object(
        &self, 
        atlas: &AtlasMem<4, u8>, 
        obj_size: SqSize, 
        baseline: SqPos, 
        baseline_x_tail: NonZeroU32, 
    ) -> Result<SqPos, u32> {
        let mut seek_x_head = baseline.x().checked_sub(
            obj_size.w().get() - 1
        ).unwrap_or(0);
        let seek_x_tail = u32::min(
            baseline_x_tail.get() + obj_size.w().get() - 1, 
            atlas.size.w().get()
        );

        // 走査
        'base_seek: while seek_x_head < baseline_x_tail.get() {
            // もうこのオブジェクトには置ける場所がない
            if seek_x_tail.checked_sub(
                seek_x_head
            ).unwrap_or(0) < obj_size.w().get() {
                // 次のオブジェクトの走査へ
                return Err(baseline_x_tail.get())
            }

            // Y方向のシーク
            for seek_y in (
                *baseline.y()
                ..baseline.y() + obj_size.h().get()
            ).rev() {
                // Yシーク開始時点でのシリアル座標
                let y_ser = atlas.size.w().get() as usize * seek_y as usize;
                for seek_x in (
                    seek_x_head
                    ..seek_x_head + obj_size.w().get()
                ).rev() {
                    if self.unuse_flag[y_ser + seek_x as usize] != 0 {
                        seek_x_head = seek_x + 1;
                        continue 'base_seek
                    }
                }
            }

            // 設置可能！
            return Ok([
                seek_x_head,
                *baseline.y(),
            ].into())
        }

        Err(baseline_x_tail.get())
    }


}