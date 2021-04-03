use crate::far::{TopologyDescriptor, TopologyRefinerPtr};
use crate::sdc;
use crate::sdc::options::Options as SdcOptions;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Options {
    scheme_type: sdc::Scheme,
    scheme_options: SdcOptions,
    validate_full_topology: bool,
}

impl Options {
    pub fn new(
        scheme_type: sdc::Scheme,
        scheme_options: SdcOptions,
    ) -> Options {
        Options {
            scheme_type,
            scheme_options,
            validate_full_topology: false,
        }
    }
}

impl Default for Options {
    fn default() -> Options {
        Options {
            scheme_type: sdc::Scheme::CatmullClark,
            scheme_options: SdcOptions::default(),
            validate_full_topology: false,
        }
    }
}

extern "C" {
    pub fn TopologyRefinerFactory_TopologyDescriptor_Create(
        descriptor: *const TopologyDescriptor,
        options: Options,
    ) -> TopologyRefinerPtr;
}
