use opensubdiv_sys as sys;
use std::convert::TryInto;

use super::TopologyRefiner;

pub trait PrimvarBufferSrc {
    const NUM_ELEMENTS: i32;
    fn as_f32(&self) -> &[f32];
}

pub trait PrimvarBufferDst {
    const NUM_ELEMENTS: i32;
    fn as_f32_mut(&mut self) -> &mut [f32];
}

pub struct PrimvarRefiner {
    ptr: sys::far::PrimvarRefinerPtr,
}

impl PrimvarRefiner {
    pub fn new(tr: &TopologyRefiner) -> PrimvarRefiner {
        unsafe {
            let ptr = sys::far::PrimvarRefiner_create(tr.ptr);
            if ptr.is_null() {
                panic!("PrimvarRefiner_create() returned null");
            }
            PrimvarRefiner { ptr }
        }
    }

    pub fn destroy(&self) {
        unsafe { sys::far::PrimvarRefiner_destroy(self.ptr) };
    }

    pub fn interpolate<B1: PrimvarBufferSrc, B2: PrimvarBufferDst>(
        &self,
        level: u32,
        src: &B1,
        dst: &mut B2,
    ) {
        unsafe {
            sys::far::PrimvarRefiner_Interpolate(
                self.ptr,
                B1::NUM_ELEMENTS,
                level.try_into().unwrap(),
                src.as_f32().as_ptr(),
                dst.as_f32_mut().as_mut_ptr(),
            );
        }
    }

    /*
    pub fn InterpolateVarying(
        &self,
        num_elements: i32,
        level: i32,
        src: *const f32,
        dst: *mut f32,
    );
    pub fn InterpolateFaceUniform(
        &self,
        num_elements: i32,
        level: i32,
        src: *const f32,
        dst: *mut f32,
    );
    pub fn InterpolateFaceVarying(
        &self,
        num_elements: i32,
        level: i32,
        src: *const f32,
        dst: *mut f32,
    );
    */
}
