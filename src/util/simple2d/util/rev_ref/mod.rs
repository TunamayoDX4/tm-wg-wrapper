//! Reverse Referencible Container
//! 逆引き可能コンテナ
//! 
//! テーブルと可変長配列、エントリバッファを組み合わせただけの
//! シンプルな逆引き可能コンテナです。

use std::{
    hash::Hash, 
    collections::VecDeque, 
    borrow::Borrow, 
};

use hashbrown::HashMap;

/// # Reverse Referencible Container Element
/// ## 逆引き可能コンテナ要素
/// 
/// 内部的に隠蔽をすることが前提ですが、キーと要素を
/// 保管します。
/// 
/// KをRefCellなどに隠蔽し、内部操作を行うことは未定義挙動の原因となります。
/// (HashMap参照)
struct RevRefElement<K, T> where
    K: Eq + Hash, 
{
    elem: T, 
    key: K, 
}

/// # Reverse Referncible Container
/// ## 逆引き可能コンテナ
/// 
/// 基本的にインデックスを用いたアクセスを想定しますが、
/// キーによるテーブルでの高速逆引きが可能であることを
/// 想定したコンテナとなります。
/// 
/// KをRefCellなどに隠蔽し、内部操作を行うことは未定義挙動の原因となります。
/// (HashMap参照)
pub struct RevRefContainer<K, T> where
    K: Eq + Hash, 
{
    memory: Vec<Option<RevRefElement<K, T>>>, 
    remque: VecDeque<usize>, 
    table: HashMap<K, usize>, 
}
impl<K, T> RevRefContainer<K, T> where
    K: Eq + Hash, 
{
    /// コンテナの初期化
    pub fn new() -> Self { Self {
        memory: Vec::new(),
        remque: VecDeque::new(),
        table: HashMap::new(),
    } }

    /// コンテナのメモリ確保サイズ指定初期化
    pub fn with_capacity(
        capacity: usize, 
    ) -> Self { Self {
        memory: Vec::with_capacity(capacity),
        remque: VecDeque::new(),
        table: HashMap::with_capacity(capacity),
    }}

    /// コンテナ要素に対する参照の取得
    pub fn get(
        &self, 
        id: usize, 
    ) -> Option<(&K, &T)> {
        self.memory.get(id)
            .map(|e| e.as_ref())
            .flatten()
            .map(|e| (&e.key, &e.elem))
    }

    /// コンテナ要素に対する可変参照の取得
    pub fn get_mut(
        &mut self, 
        id: usize, 
    ) -> Option<(&K, &mut T)> {
        self.memory.get_mut(id)
            .map(|e| e.as_mut())
            .flatten()
            .map(|e| (&e.key, &mut e.elem))
    }

    /// 領域チェックを行わない参照の取得
    pub fn get_unchecked(
        &self, 
        id: usize, 
    ) -> Option<(&K, &T)> {
        self.memory[id].as_ref()
            .map(|e| (&e.key, &e.elem))
    }

    /// 領域チェックを行わない参照の取得
    pub fn get_mut_unchecked(
        &mut self, 
        id: usize, 
    ) -> Option<(&K, &mut T)> {
        self.memory[id].as_mut()
            .map(|e| (&e.key, &mut e.elem))
    }

    /// IDの参照取得
    pub fn get_id<Q: Eq + Hash + ?Sized> (
        &self, 
        key: &Q, 
    ) -> Option<usize> where
        K: Borrow<Q>, 
    {
        self.table.get(key).copied()
    }

    /// キーから内容の取得
    pub fn get_by_name<Q: Eq + Hash + ?Sized> (
        &self, 
        key: &Q, 
    ) -> Option<(usize, &T)> where
        K: Borrow<Q>, 
    {
        self.table.get(key)
            .map(|id| (
                *id, 
                self.memory[*id].as_ref()
                    .map(|e| &e.elem)
                    .unwrap()
            ))
    }

    /// キーから内容の取得
    pub fn get_by_name_mut<Q: Eq + Hash + ?Sized> (
        &mut self, 
        key: &Q, 
    ) -> Option<(usize, &mut T)> where
        K: Borrow<Q>, 
    {
        self.table.get(key)
            .map(|id| (
                *id, 
                self.memory[*id].as_mut()
                    .map(|e| &mut e.elem)
                    .unwrap()
            ))
    }

    /// 要素のイテレーション
    pub fn iter(&self) -> impl Iterator<Item = (usize, &K, &T)> {
        self.memory.iter()
            .enumerate()
            .filter_map(|(i, e)| 
                e.as_ref()
                    .map(|e| (i, &e.key, &e.elem))
            )
    }

    /// 要素の可変イテレーション
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (usize, &K, &mut T)> {
        self.memory.iter_mut()
            .enumerate()
            .filter_map(|(
                i, 
                e, 
            )| e.as_mut()
                .map(|e| (i, &e.key, &mut e.elem))
            )
    }

    /// 要素のムーブを伴うイテレーション
    pub fn into_iter(
        self, 
    ) -> impl Iterator<Item = (usize, K, T)> {
        self.memory.into_iter()
            .enumerate()
            .filter_map(|(
                i, 
                e, 
            )| e.map(
                |e| (i, e.key, e.elem)
            ))
    }

    /// 要素の挿入
    pub fn insert<Q: Eq + Hash + ?Sized + ToOwned<Owned = K>>(
        &mut self, 
        key: &Q, 
        elem: T, 
    ) -> Result<usize, T> where
        K: std::borrow::Borrow<Q>, 
    {
        if self.table.contains_key(key) { return Err(elem) }

        let idx = match self.remque.pop_front() {
            Some(uu) => {
                self.memory[uu].replace(RevRefElement { 
                    elem, 
                    key: key.to_owned(), 
                });
                uu
            }, 
            None => {
                let id = self.memory.len();
                self.memory.push(Some(RevRefElement { 
                    elem, 
                    key: key.to_owned(), 
                }));
                id
            }
        };

        if self.table.insert(key.to_owned(), idx).is_some() {
            // 論理エラー
            unreachable!("logic error in rev_ref memory")
        }

        Ok(idx)
    }

    /// 要素の遅延挿入
    pub fn insert_lazy<'a, 'b, Q, Tpt0, Tpt1, IT>(
        &'a mut self, 
        key: &'b Q, 
        elem_part: IT, 
    ) -> Result<
        LazyInserter<'a, 'b, K, T, Tpt0, Tpt1, Q>, 
        IT, 
    > where
        Q: Eq + Hash + ?Sized + ToOwned<Owned = K>, 
        K: Borrow<Q>, 
        T: From<(Tpt0, Tpt1)>, 
        IT: IntoPart<T, Tpt0, Tpt1>, 
        'a: 'b, 
    {
        if self.table.contains_key(key) { return Err(elem_part) }

        let idx = self.remque.pop_front()
            .unwrap_or(self.memory.len());

        if self.table.insert(key.to_owned(), idx).is_some() {
            // 論理エラー
            unreachable!("logic error in rev_ref memory")
        }

        Ok(LazyInserter {
            _dummy: std::marker::PhantomData,
            ref_to: self,
            key,
            idx,
            part: elem_part.part(),
        })
    }

    /// 要素の削除
    pub fn remove(
        &mut self, 
        idx: usize, 
    ) -> Option<(K, T)> {
        let elem = self.memory.get_mut(idx)
            .map(|m| m.take())
            .flatten()?;
        self.remque.push_back(idx);
        if self.table.remove(&elem.key).is_none() { unreachable!() }
        Some((
            elem.key, 
            elem.elem, 
        ))
    }

    /// キーによる要素の削除
    pub fn remove_by_name<Q: Eq + Hash + ?Sized>(
        &mut self, 
        key: &Q, 
    ) -> Option<(usize, K, T)> where
        K: Borrow<Q>, 
    {
        let idx = self.table.remove(key)?;
        let elem = self.memory.get_mut(idx)
            .map(|m| m.take())
            .flatten()
            .expect("logic error in rev_ref memory");
        self.remque.push_back(idx);
        Some((
            idx, 
            elem.key, 
            elem.elem, 
        ))
    }
}

/// 遅延挿入用のインサータ
pub struct LazyInserter<'a, 'b, K, T, Tpt0, Tpt1, Q> where
    K: Eq + Hash, 
    T: From<(Tpt0, Tpt1)>, 
    Q: Eq + Hash + ?Sized + ToOwned<Owned = K>, 
    'a: 'b, 
{
    _dummy: std::marker::PhantomData<Tpt1>, 
    ref_to: &'a mut RevRefContainer<K, T>, 
    key: &'b Q, 
    idx: usize, 
    part: Tpt0, 
}
impl<'a, 'b, K, T, Tpt0, Tpt1, Q> LazyInserter<'a, 'b, K, T, Tpt0, Tpt1, Q> where
    K: Eq + Hash, 
    T: From<(Tpt0, Tpt1)>, 
    Q: Eq + Hash + ?Sized + ToOwned<Owned = K>, 
    'a: 'b, 
{
    pub fn part(&self) -> &Tpt0 { &self.part }
    pub fn key(&self) -> &Q { self.key }
    pub fn idx(&self) -> usize { self.idx }
    pub fn insert(
        self, 
        part: Tpt1, 
    ) {
        self.ref_to.memory[self.idx] = Some(RevRefElement { 
            elem: (self.part, part).into(), 
            key: self.key.to_owned(),  
        })
    }
}

/// パーツに出来るやつ
pub trait IntoPart<T, Tpt0, Tpt1> where
    T: From<(Tpt0, Tpt1)>, 
{
    fn part(self) -> Tpt0;
}
impl<T, Tpt0, Tpt1> IntoPart<T, Tpt0, Tpt1> for Tpt0 where
    T: From<(Tpt0, Tpt1)>, 
{
    fn part(self) -> Tpt0 { self }
}