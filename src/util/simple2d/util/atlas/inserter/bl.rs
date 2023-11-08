//! Bottom-Left法に基づいたインサータ

use std::{
    collections::BTreeMap, 
    num::NonZeroU32, hash::Hash
};

pub mod error {
    /// 挿入処理に失敗
    #[derive(Debug, Clone)]
    pub enum BLInsertError {
        KeyDuplicate, 
        InsNotEnoughSpace, 
        InsDataIsTooLarge, 
    }

    /// 除去処理に失敗
    #[derive(Debug, Clone)]
    pub enum BLRemoveError {
        EntryNotExist, 
    }

    impl std::fmt::Display for BLInsertError {
        fn fmt(
            &self, 
            f: &mut std::fmt::Formatter<'_>
        ) -> std::fmt::Result { f.write_fmt(match self {
            BLInsertError::KeyDuplicate => format_args!(
                "Insert key duplicate"
            ), 
            BLInsertError::InsNotEnoughSpace => format_args!(
                "Insert space is not enough"
            ),
            BLInsertError::InsDataIsTooLarge => format_args!(
                "Insert data is too large"
            ),
        })}
    }
    impl std::error::Error for BLInsertError {}

    impl std::fmt::Display for BLRemoveError {
        fn fmt(
            &self, 
            f: &mut std::fmt::Formatter<'_>
        ) -> std::fmt::Result { f.write_fmt(match self {
            BLRemoveError::EntryNotExist => format_args!(
                "Entry is not exist"
            ), 
        })}
    }
    impl std::error::Error for BLRemoveError {}
}

/// # BL法に基づいた挿入処理モジュールのイニシャライザ
pub struct BLInserterInitializer;
impl<
    const BL: usize, 
    P: Copy, 
    K: Eq + Hash, 
    T, 
> super::AtlasControllerInitializer<
    BL, 
    P, 
    K, 
    T, 
> for BLInserterInitializer {
    type Initialized = BLInserter;
    type InitError = ();

    fn initialize(
        self, 
        size: super::SqSize, 
        _memory: &mut super::AtlasMem<BL, P>, 
    ) -> Result<Self::Initialized, Self::InitError> { Ok(BLInserter {
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

/// # BL法に基づいた挿入処理モジュール
pub struct BLInserter {

    /// データが未使用であることを確認するためのフラグ
    unuse_flag: Vec<u8>, 

    /// オブジェクトの配置可能箇所のチートシート
    base_line: BTreeMap<u32, Option<Vec<(u32, NonZeroU32)>>>, 
}
impl<
    const BL: usize, 
    P: Copy, 
    K: Eq + Hash, 
    T, 
> super::AtlasController<BL, P, K, T> for BLInserter {
    type InsertError = error::BLInsertError;
    type RemoveError = error::BLRemoveError;
    type ControllerElemData = ();

    fn insert<
        'a, 
        'b, 
        Q: Eq + Hash + ?Sized + ToOwned<Owned = K>, 
    >(
        &mut self, 
        atlas: &mut super::AtlasMem<BL, P>, 
        elem: &'a mut super::RevRefContainer<
            K, 
            super::AtlasElem<T, Self::ControllerElemData>
        >, 
        key: &'b Q, 
        size: Option<super::SqSize>, 
    ) -> Result<
        super::super::super::rev_ref::LazyInserter<
            'a, 
            'b, 
            K, 
            super::AtlasElem<T, Self::ControllerElemData>, 
            (Option<super::AtlasMemParam>, Self::ControllerElemData), 
            T, 
            Q, 
        >, 
        Self::InsertError
    > where
        K: std::borrow::Borrow<Q>, 
    {match size {
        Some(size) => { if atlas.size.w() < size.w() 
            && atlas.size.h() < size.h() 
        {
            return Err(error::BLInsertError::InsDataIsTooLarge)
        } else {
            let pos = self.seek_baseline(
                atlas, 
                size
            )?;

            let memp = super::AtlasMemParam {
                pos, 
                size, 
            };
            let li = elem.insert_lazy(
                key, 
                (
                    Some(memp), 
                    ()
                )
            ).map_err(|_| error::BLInsertError::KeyDuplicate)?;

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

            Ok(li)
        }}, 
        None => {
            let li = elem.insert_lazy(
                key, 
                (None, ())
            ).map_err(|_| error::BLInsertError::KeyDuplicate)?;
            Ok(li)
    }}}

    fn remove(
        &mut self, 
        atlas: &mut super::AtlasMem<BL, P>, 
        elem: &mut super::RevRefContainer<
            K, 
            super::AtlasElem<T, Self::ControllerElemData>
        >, 
        id: usize, 
    ) -> Result<
        (T, K, Option<super::AtlasMemParam>), 
        Self::RemoveError
    > {match elem.remove(id) {
        Some((
            k, 
            super::AtlasElem { 
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
            super::AtlasElem { 
                memp: None, 
                ud: d, 
                insert_data: _ 
            }
        )) => Ok((d, k, None)), 
        None => Err(error::BLRemoveError::EntryNotExist)
    }}
}
impl BLInserter {
    /// オブジェクトが置ける場所ごとの走査
    fn seek_baseline<
        const BL: usize, 
        P: Copy, 
    >(
        &self, 
        atlas: &super::AtlasMem<BL, P>, 
        obj_size: super::SqSize, 
    ) -> Result<super::SqPos, error::BLInsertError> {
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
                return Err(error::BLInsertError::InsNotEnoughSpace)
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

        Err(error::BLInsertError::InsNotEnoughSpace)
    }

    /// オブジェクトが置ける場所を詳細に走査
    fn seek_object<
        const BL: usize, 
        P: Copy, 
    >(
        &self, 
        atlas: &super::AtlasMem<BL, P>, 
        obj_size: super::SqSize, 
        baseline: super::SqPos, 
        baseline_x_tail: NonZeroU32, 
    ) -> Result<super::SqPos, u32> {
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