use std::collections::VecDeque;

use super::{
    types::InstanceGen, 
    img_obj::{
        ImgObjInstance, 
        ImgObjRender, 
    }
};

/// 単一エンティティホルダ
pub struct EntityHolder<T: InstanceGen<ImgObjInstance> + std::fmt::Debug> (Option<T>);
impl<T: InstanceGen<ImgObjInstance> + std::fmt::Debug> EntityHolder<T> {
    pub fn new(
        initializer: impl Into<T>, 
    ) -> Self { Self (
        Some(initializer.into())
    )}

    pub fn remove(&mut self) {
        self.0 = None
    }

    pub fn retain(
        &mut self, 
        mut f: impl FnMut(&mut T) -> bool, 
    ) {
        match &mut self.0 {
            t @ Some(_) => if !f(t.as_mut().unwrap()) {
                *t = None;
            }, 
            None => {}, 
        }
    }

    pub fn render_update(
        &self, 
        renderer: &mut ImgObjRender, 
    ) {
        renderer.update_instances(
            [&self.0].iter()
                .filter_map(|t|(*t).as_ref())
        )
    }

    pub fn get(&self) -> Option<&T> { self.0.as_ref() }
    pub fn get_mut(&mut self) -> Option<&mut T> { self.0.as_mut() }

    pub fn manip<R>(&self, f: impl FnOnce(&T) -> R) -> Option<R> { 
        self.0.as_ref().map(|t| f(t))
    }
    pub fn manip_mut<R>(&mut self, f: impl FnOnce(&mut T) -> R) -> Option<R> {
        self.0.as_mut().map(|t| f(t))
    }
}

/// エンティティ配列
pub struct EntityArray<T: InstanceGen<ImgObjInstance>> {
    entity: Vec<Option<T>>, 
    remove_queue: VecDeque<usize>, 
}
impl<T: InstanceGen<ImgObjInstance>>  EntityArray<T> {
    pub fn new(
        initializer: impl IntoIterator<Item = T>, 
    ) -> Self { Self {
        entity: initializer.into_iter().map(|i| Some(i)).collect(), 
        remove_queue: VecDeque::new(), 
    }}

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

    pub fn push(&mut self, entity: T) {
        while let Some(idx) = self.remove_queue.pop_front() {
            match self.entity.get_mut(idx) {
                e @ Some(None) => {
                    *e.unwrap() = Some(entity);
                    return;
                }, 
                None | Some(Some(_)) => {}, 
            }
        };
        self.entity.push(Some(entity))
    }

    pub fn render_update(
        &self, 
        renderer: &mut ImgObjRender, 
    ) {
        renderer.update_instances(
            self.entity.iter()
                .filter_map(|e| e.as_ref())
        )
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

    pub fn iter(&self) -> impl Iterator<Item = (usize, &T)> {
        self.entity.iter()
            .enumerate()
            .filter_map(|(idx, e)| e.as_ref().map(|e| (idx, e)))
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (usize, &mut T)> {
        self.entity.iter_mut()
            .enumerate()
            .filter_map(|(idx, e)| e.as_mut().map(|e| (idx, e)))
    }
}