use opensubdiv_sys as sys;
use std::convert::TryInto;

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BufferDescriptor(pub(crate) sys::osd::BufferDescriptor);

impl BufferDescriptor {
    pub fn new(offset: u32, length: u32, stride: u32) -> Self {
        Self(sys::osd::BufferDescriptor {
            offset: offset.try_into().unwrap(),
            length: length.try_into().unwrap(),
            stride: stride.try_into().unwrap(),
        })
    }

    /// Returns the relative offset within a stride.
    pub fn local_offset(&self) -> u32 {
        if self.0.stride != 0 {
            (self.0.offset % self.0.stride) as _
        } else {
            0
        }
    }

    /// True if the descriptor values are internally consistent.
    pub fn is_valid(&self) -> bool {
        (self.0.length != 0)
            && (self.0.length <= self.0.stride - (self.local_offset() as i32))
    }

    pub fn is_empty(&self) -> bool {
        0 == self.0.length
    }
}
