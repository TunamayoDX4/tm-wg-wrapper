use super::*;

impl<K: Eq + Hash, T, I: Sized> AtlasElemMem<K, T, I> {
    pub fn new() -> Self { Self {
        mem: Vec::new(),
        uu_mem: VecDeque::new(),
        table: HashMap::new(),
    } }

    pub fn insert<Q>(
        &mut self, 
        memp: Option<AtlasMemParam>, 
        ud: T, 
        key: &Q, 
        insert_data: I, 
    ) -> Result<
        usize, 
        (Option<AtlasMemParam>, T), 
    > where 
        Q: Eq + Hash + ?Sized + ToOwned<Owned = K>, 
        K: std::borrow::Borrow<Q>, 
    {
        if self.table.contains_key(key) { return Err((
            memp, ud
        )) }

        let idx = match self.uu_mem.pop_front() {
            Some(uu) => {
                self.mem[uu].replace(AtlasElem {
                    memp,
                    ud,
                    key: key.to_owned(),
                    insert_data, 
                });
                uu
            }, 
            None => {
                let id = self.mem.len();
                self.mem.push(Some(AtlasElem {
                    memp,
                    ud,
                    key: key.to_owned(),
                    insert_data, 
                }));
                id
            }
        };

        if self.table.insert(key.to_owned(), idx).is_some() {
            // 論理エラー
            unreachable!()
        }

        Ok(idx)
    }

    pub fn remove(
        &mut self, 
        idx: usize, 
    ) -> Option<(
        Option<AtlasMemParam>, 
        T, 
        K, 
        I, 
    )> {
        let elem = self.mem.get_mut(idx)
            .map(|m| m.take())
            .flatten()?;
        self.uu_mem.push_back(idx);
        if self.table.remove(&elem.key).is_none() { unreachable!() }
        Some((
            elem.memp, 
            elem.ud, 
            elem.key, 
            elem.insert_data, 
        ))
    }

    pub fn remove_by_name<Q>(
        &mut self, 
        key: &Q, 
    ) -> Option<(usize, (
        Option<AtlasMemParam>, 
        T, 
        K, 
        I, 
    ))> where
        Q: Eq + Hash, 
        K: std::borrow::Borrow<Q>, 
    {
        let idx = self.table.remove(key)?;
        let elem = self.mem.get_mut(idx)
            .map(|m| m.take())
            .flatten()
            .expect("logic error");
        self.uu_mem.push_back(idx);
        Some((idx, (
            elem.memp, 
            elem.ud, 
            elem.key, 
            elem.insert_data, 
        )))
    }

    pub fn get_id<Q>(
        &self, key: &Q, 
    ) -> Option<usize> where
        Q: Eq + Hash + ?Sized, 
        K: std::borrow::Borrow<Q>, 
    {
        self.table.get(key).copied()
    }

    pub fn get(
        &self, idx: usize, 
    ) -> Option<(Option<&AtlasMemParam>, &T, &K, &I)> {
        self.mem.get(idx)
            .map(|ae| ae.as_ref())
            .flatten()
            .map(|ae| (
                ae.memp.as_ref(), 
                &ae.ud, 
                &ae.key, 
                &ae.insert_data, 
            ))
    }

    pub fn get_mut(
        &mut self, idx: usize, 
    ) -> Option<(Option<&AtlasMemParam>, &mut T, &K, &mut I)> {
        self.mem.get_mut(idx)
            .map(|ae| ae.as_mut())
            .flatten()
            .map(|ae| (
                ae.memp.as_ref(), 
                &mut ae.ud, 
                &ae.key, 
                &mut ae.insert_data, 
            ))
    }

    pub fn get_by_name<Q>(
        &self, key: &Q, 
    ) -> Option<(Option<&AtlasMemParam>, &T, &K, &I)> where
        Q: Eq + Hash + ?Sized, 
        K: std::borrow::Borrow<Q>, 
    {
        self.get_id(key)
            .map(|id| self.get(id))
            .flatten()
    }

    pub fn get_by_name_mut<Q>(
        &mut self, key: &Q, 
    ) -> Option<(Option<&AtlasMemParam>, &mut T, &K, &mut I)> where
        Q: Eq + Hash + ?Sized, 
        K: std::borrow::Borrow<Q>, 
    {
        self.get_id(key)
            .map(|id| self.get_mut(id))
            .flatten()
    }

    pub fn iter(&self) -> impl Iterator<
        Item = (usize, (Option<&AtlasMemParam>, &T, &K, &I))
    > {
        self.mem.iter()
            .enumerate()
            .filter_map(|(
                idx, 
                ae
            )| ae.as_ref()
                .map(|ae| (idx, ae)))
            .map(|(id, ae)| (
                id, 
                (
                    ae.memp.as_ref(), 
                    &ae.ud, 
                    &ae.key, 
                    &ae.insert_data, 
                )
            ))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<
        Item = (usize, (Option<&AtlasMemParam>, &mut T, &K, &mut I))
    > {
        self.mem.iter_mut()
            .enumerate()
            .filter_map(|(
                idx, 
                ae
            )| ae.as_mut()
                .map(|ae| (idx, ae)))
            .map(|(id, ae)| (
                id, 
                (
                    ae.memp.as_ref(), 
                    &mut ae.ud, 
                    &ae.key, 
                    &mut ae.insert_data, 
                )
            ))
    }
}