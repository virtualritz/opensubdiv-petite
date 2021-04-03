#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub struct BufferDescriptor {
    offset: i32,
    length: i32,
    stride: i32,
}

impl BufferDescriptor {
    pub fn new(offset: i32, length: i32, stride: i32) -> BufferDescriptor {
        BufferDescriptor {
            offset,
            length,
            stride,
        }
    }
    /// Returns the relative offset within a stride
    pub fn get_local_offset(&self) -> i32 {
        if self.stride > 0 {
            self.offset % self.stride
        } else {
            0
        }
    }

    /// True if the descriptor values are internally consistent
    pub fn is_valid(&self) -> bool {
        (self.length > 0)
            && (self.length <= self.stride - self.get_local_offset())
    }
}
