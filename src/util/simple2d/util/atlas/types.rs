use std::num::NonZeroU32;

/// 長方形の座標
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SqPos ([u32; 2]);
impl From<[u32; 2]> for SqPos {
    fn from(value: [u32; 2]) -> Self { Self(value) }
}
impl From<SqPos> for [u32; 2] {
    fn from(value: SqPos) -> Self { value.0 }
}
impl SqPos {
    pub fn new(size: [u32; 2]) -> Self { Self(size) }
    pub fn raw(&self) -> &[u32; 2] { &self.0 }
    pub fn raw_mut(&mut self) -> &mut [u32; 2] { &mut self.0 }
    pub fn x(&self) -> &u32 { &self.0[0] }
    pub fn y(&self) -> &u32 { &self.0[1] }
    pub fn x_mut(&mut self) -> &mut u32 { &mut self.0[0] }
    pub fn y_mut(&mut self) -> &mut u32 { &mut self.0[1] }
    pub fn serial(&self, size: SqSize) -> usize {
        size.w().get() as usize * *self.y() as usize + *self.x() as usize
    }
    pub fn serial_checked(&self, size: SqSize) -> Option<usize> {
        if *self.x() < size.w().get() && *self.y() < size.h().get() {
            Some(self.serial(size))
        } else { None }
    }
}

/// 長方形の大きさ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SqSize ([NonZeroU32; 2]);
impl From<[NonZeroU32; 2]> for SqSize {
    fn from(value: [NonZeroU32; 2]) -> Self { Self(value) }
}
impl From<SqSize> for [u32; 2] {
    fn from(value: SqSize) -> Self { [
        value.w().get(), value.h().get()
    ]}
}
impl From<SqSize> for [NonZeroU32; 2] {
    fn from(value: SqSize) -> Self { value.0 }
}
impl SqSize {
    pub fn new_checked(size: [u32; 2]) -> Option<Self> {Some(Self([
        NonZeroU32::new(size[0])?, 
        NonZeroU32::new(size[1])?, 
    ]))}
    pub fn new(size: [NonZeroU32; 2]) -> Self {Self(size)}
    pub fn raw(&self) -> &[NonZeroU32; 2] { &self.0 }
    pub fn raw_mut(&mut self) -> &mut [NonZeroU32; 2] { &mut self.0 }
    pub fn w(&self) -> &NonZeroU32 { &self.0[0] }
    pub fn h(&self) -> &NonZeroU32 { &self.0[1] }
    pub fn w_mut(&mut self) -> &mut NonZeroU32 { &mut self.0[0] }
    pub fn h_mut(&mut self) -> &mut NonZeroU32 { &mut self.0[1] }
    pub fn serial(&self) -> usize {
        self.w().get() as usize * self.h().get() as usize
    }
}