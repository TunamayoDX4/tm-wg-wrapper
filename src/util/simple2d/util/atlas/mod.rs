use std::hash::Hash;

pub mod types;
pub mod memory;
pub mod elem;

/// # テクスチャ・アトラス作成用ユーティリティ
pub struct Atlas<const BL: usize, P, K, T, C> where
    P: Copy, 
    K: Eq + Hash, 
    C: AtlasController<BL, P, K, T>, 
{
    memory: memory::AtlasMem<BL, P>, 
    elem: super::rev_ref::RevRefContainer<
        K, 
        elem::AtlasElem<T, C::ControllerElemData>, 
    >, 
    inserter: C, 
}
impl<const BL: usize, P, K, T, C> Atlas<BL, P, K, T, C> where
    P: Copy, 
    K: Eq + Hash, 
    C: AtlasController<BL, P, K, T>, 
{
    pub fn new<Ci: AtlasControllerInitializer<
        BL, 
        P, 
        K, 
        T, 
        Initialized = C, 
    >>(
        size: types::SqSize, 
        inserter_initializer: Ci, 
        pixel: P, 
    ) -> Result<Self, Ci::InitError> { 
        let mut memory = memory::AtlasMem::new(
            size, 
            pixel, 
        );
        //let elem = elem::AtlasElemMem::new();
        let elem = super::rev_ref::RevRefContainer::new();
        let inserter = inserter_initializer.initialize(
            size, 
            &mut memory, 
        )?;
        Ok(Self {
            memory,
            elem,
            inserter,
        })
    }

    pub fn raw(&self) -> &[P] { self.memory.raw() }
    pub fn size(&self) -> types::SqSize { self.memory.size }

    pub fn iter(&self) -> impl Iterator<Item = (
        u32, 
        &T, 
        &K, 
        Option<impl Iterator<Item = (usize, &[P])>>, 
    )> {
        self.elem.iter()
            .map(|(
                id, 
            key, 
            &elem::AtlasElem {
                memp: aep, 
                ref ud, 
                insert_data: _, 
            }, 
        )| (
            id as u32, 
            ud, 
            key, 
            aep.map(|aep| self.memory.get_obj(&aep))
        ))
    }

    pub fn get_id<Q>(
        &self, 
        key: &Q, 
    ) -> Option<u32> where
        Q: Eq + Hash + ?Sized, 
        K: std::borrow::Borrow<Q>, 
    {
        self.elem.get_id(key).map(|idx| idx as u32)
    }

    pub fn get_amp(
        &self, 
        id: u32, 
    ) -> Option<&elem::AtlasMemParam> {
        self.elem
            .get(id as usize)
            .map(|(
                _k, 
                ae
            )| ae.memp.as_ref()).flatten()
    }

    pub fn get_amp_by_name<Q>(
        &self, 
        key: &Q, 
    ) -> Option<&elem::AtlasMemParam> where
        Q: Eq + Hash + ?Sized, 
        K: std::borrow::Borrow<Q>, 
    {
        self.elem
            .get(self.get_id(key)? as usize)
            .map(|(
                k, 
                ae, 
            )| ae.memp.as_ref()).flatten()
    }

    pub fn get(
        &self, 
        id: usize, 
    ) -> Option<(
        Option<&elem::AtlasMemParam>, 
        &T, 
        &K, 
        Option<impl Iterator<Item = (usize, &[P])>>, 
    )> {
        self.elem.get(id)
            .map(|(
                key, 
                &elem::AtlasElem {
                    memp: ref aem, 
                    ref ud, 
                    insert_data: _, 
                }
            )| (
                aem.as_ref(), 
                ud, 
                key, 
                aem.as_ref().map(|ae| self.memory.get_obj(ae))
            ))
    }

    pub fn get_by_name<Q>(
        &self, 
        key: &Q, 
    ) -> Option<(
        Option<&elem::AtlasMemParam>, 
        &T, 
        &K, 
        Option<impl Iterator<Item = (usize, &[P])>>, 
    )> where
        Q: Eq + Hash + ?Sized, 
        K: std::borrow::Borrow<Q>, 
    {
        self.get(self.get_id(key)? as usize)
    }

    pub fn get_mut(
        &mut self, 
        id: usize, 
    ) -> Option<(
        Option<&elem::AtlasMemParam>, 
        &mut T, 
        &K, 
        Option<impl Iterator<Item = (usize, &mut [P])>>, 
    )> {
        self.elem.get_mut(id)
            .map(|(
                key, 
                &mut elem::AtlasElem {
                    memp: ref aem,
                    ref mut ud,
                    insert_data: _,
                }, 
            )| (
                aem.as_ref(), 
                ud, 
                key, 
                aem.as_ref().map(|ae| self.memory.get_obj_mut(ae))
            ))
    }

    pub fn get_by_name_mut<Q>(
        &mut self, 
        key: &Q, 
    ) -> Option<(
        Option<&elem::AtlasMemParam>, 
        &mut T, 
        &K, 
        Option<impl Iterator<Item = (usize, &mut [P])>>, 
    )> where
        Q: Eq + Hash + ?Sized, 
        K: std::borrow::Borrow<Q>, 
    {
        self.get_mut(self.get_id(key)? as usize)
    }

    pub fn insert<Q>(
        &mut self, 
        key: &Q, 
        ud: T, 
        size: Option<types::SqSize>, 
    ) -> Result<(
        usize, Option<impl Iterator<Item = (usize, &mut [P])>>
    ), C::InsertError> where
        Q: Eq + Hash + ?Sized + ToOwned<Owned = K>, 
        K: std::borrow::Borrow<Q>, 
    {
        let (id, memp) = self.inserter.insert(
            &mut self.memory, 
            &mut self.elem, 
            key, 
            size, 
            ud
        )?;
        Ok((
            id, 
            memp.map(|memp| self.memory.get_obj_mut(&memp))
        ))
    }

    pub fn remove(
        &mut self, 
        id: usize, 
    ) -> Result<(
        T, K, Option<impl Iterator<Item = (usize, &[P])>>
    ), C::RemoveError> {
        let (t, k, memp) = self.inserter.remove(
            &mut self.memory, 
            &mut self.elem, 
            id
        )?;

        Ok((t, k, memp.map(|memp| self.memory.get_obj(&memp))))
    }

    pub fn remove_by_name<Q: Eq + Hash + ?Sized>(
        &mut self, 
        key: &Q, 
    ) -> Option<Result<
        (T, K, Option<impl Iterator<Item = (usize, &[P])>>), 
        C::RemoveError, 
    >> where
        K: std::borrow::Borrow<Q>, 
    {
        Some(self.remove(self.elem.get_id(key)?))
    }
}

/// # インサータの初期化用構造体
pub trait AtlasControllerInitializer<const BL: usize, P, K, T> where
    Self: Sized, 
    P: Copy, 
    K: Eq + Hash, 
{
    type Initialized: AtlasController<BL, P, K, T>;
    type InitError;
    fn initialize(
        self, 
        size: types::SqSize, 
        memory: &mut memory::AtlasMem<BL, P>, 
    ) -> Result<Self::Initialized, Self::InitError>;
}

/// # アトラスに要素を挿入するインサータのテンプレート
pub trait AtlasController<const BL: usize, P, K, T> where
    P: Copy, 
    K: Eq + Hash, 
{
    type InsertError;
    type RemoveError;
    type ControllerElemData: Sized;
    
    fn insert<Q: Eq + Hash + ?Sized + ToOwned<Owned = K>>(
        &mut self, 
        atlas: &mut memory::AtlasMem<BL, P>, 
        elem: &mut super::rev_ref::RevRefContainer<
            K, 
            elem::AtlasElem<T, Self::ControllerElemData>
        >, 
        key: &Q, 
        size: Option<types::SqSize>, 
        ud: T, 
    ) -> Result<(usize, Option<elem::AtlasMemParam>), Self::InsertError> where
        K: std::borrow::Borrow<Q>
    ;

    fn remove(
        &mut self, 
        atlas: &mut memory::AtlasMem<BL, P>, 
        elem: &mut super::rev_ref::RevRefContainer<
            K, 
            elem::AtlasElem<T, Self::ControllerElemData>
        >, 
        id: usize, 
    ) -> Result<(T, K, Option<elem::AtlasMemParam>), Self::RemoveError>;
}