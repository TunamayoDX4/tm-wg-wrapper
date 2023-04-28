use std::collections::VecDeque;

use super::{
    instance::{
        Instance, 
        InstanceGen, 
    }, 
};

pub struct EntityHolder<_T: Instance, T: InstanceGen<_T>> {
    _dummy: std::marker::PhantomData<_T>, 
    entity: Option<T>, 
}
impl<_T: Instance, T: InstanceGen<_T>> EntityHolder<_T, T> {
    pub fn new(
        initializer: impl Into<T>, 
    ) -> Self { Self {
        _dummy: Default::default(), 
        entity: Some(initializer.into()), 
    } }

    pub fn remove(&mut self) { self.entity = None }
    pub fn exist(&self) -> bool { self.entity.is_some() }

    pub fn retain(
        &mut self, 
        mut f: impl FnMut(&mut T) -> bool, 
    ) {
        if !self.entity.as_mut().map_or(
            false, 
            |e| f(e)
        ) { self.entity = None }
    }

    pub fn get(&self) -> Option<&T> { self.entity.as_ref() }
    pub fn get_mut(&mut self) -> Option<&mut T> { self.entity.as_mut() }
    pub fn manip<R>(&self, f: impl FnOnce(&T) -> R) -> Option<R> {
        self.entity.as_ref().map(|t| f(t))
    }
    pub fn manip_mut<R>(&mut self, f: impl FnOnce(&mut T) -> R) -> Option<R> {
        self.entity.as_mut().map(|t| f(t))
    }
}
impl<_T: Instance, T: InstanceGen<_T>> InstanceGen<_T> for EntityHolder<_T, T> {
    fn generate(&self, instances: &mut super::instance::buffer::InstanceArray<_T>) {
        self.entity.as_ref().map(|e| e.generate(instances));
    }
}

pub struct EntityArray<_T: Instance, T: InstanceGen<_T>> {
    _dummy: std::marker::PhantomData<_T>, 
    entity: Vec<Option<T>>, 
    remove_queue: VecDeque<usize>, 
}
impl<_T: Instance, T: InstanceGen<_T>> EntityArray<_T, T>{
    pub fn new(
        initializer: impl IntoIterator<Item = T>, 
    ) -> Self { 
        let entity = initializer.into_iter().map(|i| {
            Some(i)
        }).collect();
        Self {
            _dummy: Default::default(), 
            entity, 
            remove_queue: VecDeque::new(), 
        }
    }

    pub fn remove(&mut self, idx: usize) {
        if let Some(_) = self.entity.get_mut(idx)
            .map(|e| e.take())
            .flatten() 
        {
            self.remove_queue.push_back(idx);
        }
    }

    pub fn retain(
        &mut self, 
        mut f: impl FnMut(usize, &mut T) -> bool, 
    ) -> usize {
        let mut count = 0;
        self.entity.iter_mut()
            .enumerate()
            .filter_map(|(idx, e)| match e {
                e @ Some(_) => {
                    if !f(idx, e.as_mut().unwrap()) {
                        Some((e, idx))
                    } else {
                        None
                    }
                }, 
                _ => None, 
            })
            .for_each(|(e, idx)| {
                count += 1;
                *e = None;
                self.remove_queue.push_back(idx)
            });
        count
    }

    pub fn push(&mut self, entity: T) -> usize {
        while let Some(idx) = self.remove_queue.pop_front() {
            match self.entity.get_mut(idx) {
                e @ Some(None) => {
                    *e.unwrap() = Some(entity);
                    return idx;
                }, 
                None | Some(Some(_)) => {}, 
            }
        };
        let idx = self.entity.len();
        self.entity.push(Some(entity));
        idx
    }

    pub fn get(&self, idx: usize) -> Option<&T> { 
        self.entity.get(idx)
            .map(|t| t.as_ref())
            .flatten()
    }
    pub fn get_mut(&mut self, idx: usize) -> Option<&mut T> { 
        self.entity.get_mut(idx)
            .map(|t| t.as_mut())
            .flatten()
    }

    pub fn manip<R>(&self, idx: usize, f: impl FnOnce(&T) -> R) -> Option<R> {
        self.entity.get(idx)
            .map(|t| t.as_ref())
            .flatten()
            .map(|t| f(t))
    }
    pub fn manip_mut<R>(&mut self, idx: usize, f: impl FnOnce(&mut T) -> R) -> Option<R> {
        self.entity.get_mut(idx)
            .map(|t| t.as_mut())
            .flatten()
            .map(|t| f(t))
    }

    pub fn iter(&self) -> impl Iterator<Item = EntityRef<_T, T>> {
        self.entity.iter()
            .enumerate()
            .filter_map(|(idx, e)| 
                e.as_ref().map(|e| EntityRef {
                    _dummy: std::marker::PhantomData,
                    idx,
                    entity: e,
                })
            )
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = EntityRefMut<_T, T>> {
        self.entity.iter_mut()
            .enumerate()
            .filter_map(|(idx, e)| 
                e.as_mut().map(|e| EntityRefMut {
                    _dummy: std::marker::PhantomData,
                    idx,
                    entity: e,
                })
            )
    }
}
impl<_T: Instance, T: InstanceGen<_T>> InstanceGen<_T> for EntityArray<_T, T> {
    fn generate(&self, instances: &mut super::instance::buffer::InstanceArray<_T>) {
        self.entity.iter()
            .filter_map(|e| e.as_ref())
            .for_each(|e| e.generate(instances));
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EntityRef<'a, _T: Instance, T: InstanceGen<_T>> {
    _dummy: std::marker::PhantomData<_T>, 
    pub idx: usize, 
    pub entity: &'a T, 
}

pub struct EntityRefMut<'a, _T: Instance, T: InstanceGen<_T>> {
    _dummy: std::marker::PhantomData<_T>, 
    pub idx: usize, 
    pub entity: &'a mut T, 
}