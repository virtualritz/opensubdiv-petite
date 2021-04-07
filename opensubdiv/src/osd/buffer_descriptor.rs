//! Describes elements in interleaved data buffers.
//!
//! Example:
//! ```
//!      n
//! -----+----------------------------------------+-------------------------
//!      |               vertex  0                |
//! -----+----------------------------------------+-------------------------
//!      |  X  Y  Z  R  G  B  A Xu Yu Zu Xv Yv Zv |
//! -----+----------------------------------------+-------------------------
//!      <------------- stride = 13 -------------->
//!
//!    - XYZ      (offset = n+0,  length = 3, stride = 13)
//!    - RGBA     (offset = n+3,  length = 4, stride = 13)
//!    - uTangent (offset = n+7,  length = 3, stride = 13)
//!    - vTangent (offset = n+10, length = 3, stride = 13)
//! ```
use opensubdiv_sys as sys;
use std::convert::TryInto;

/// A struct which describes buffer elements in interleaved data buffers.
///
/// Almost all evaluator APIs take `BufferDescriptor`s along with
/// device-specific buffer objects.
///
/// The `offset` of `BufferDescriptor` can also be used to express a batching
/// offset if the data buffer is combined across multiple objects together.
///
/// * Note that each element has the same data type ([`f32`]).
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BufferDescriptor(pub(crate) sys::osd::BufferDescriptor);

impl BufferDescriptor {
    /// Create new buffer descriptor.
    ///
    /// Use [`default()`](BufferDescriptor::default()) to create an empty buffer
    /// descriptor.
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

    /// Returns `true` if the descriptor values are internally consistent.
    pub fn is_valid(&self) -> bool {
        (self.0.length != 0)
            && (self.0.length <= self.0.stride - (self.local_offset() as i32))
    }

    /// Returns `true` if this buffer descriptor is empty.
    pub fn is_empty(&self) -> bool {
        0 == self.0.length
    }
}

impl Default for BufferDescriptor {
    /// Create an empty buffer desciptior.
    fn default() -> Self {
        Self {
            offset: 0,
            length: 0,
            stride: 0,
        }
    }
}
