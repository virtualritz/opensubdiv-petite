use opensubdiv_sys as sys;
use std::convert::TryInto;

use super::TopologyRefiner;

pub trait PrimvarBufferSrc {
    const LEN_ELEMENTS: u32;
    fn as_f32(&self) -> &[f32];
}

pub trait PrimvarBufferDst {
    const LEN_ELEMENTS: u32;
    fn as_f32_mut(&mut self) -> &mut [f32];
}

pub struct PrimvarRefiner(sys::far::PrimvarRefinerPtr);

impl PrimvarRefiner {
    pub fn new(tr: &TopologyRefiner) -> PrimvarRefiner {
        unsafe {
            let ptr = sys::far::PrimvarRefiner_create(tr.0);
            if ptr.is_null() {
                panic!("PrimvarRefiner_create() returned null");
            }
            PrimvarRefiner(ptr)
        }
    }

    pub fn interpolate<B1: PrimvarBufferSrc, B2: PrimvarBufferDst>(
        &self,
        level: u32,
        src: &B1,
        dst: &mut B2,
    ) {
        unsafe {
            sys::far::PrimvarRefiner_Interpolate(
                self.0,
                B1::LEN_ELEMENTS.try_into().unwrap(),
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

impl Drop for PrimvarRefiner {
    fn drop(&mut self) {
        unsafe { sys::far::PrimvarRefiner_destroy(self.0) };
    }
}
