extern "C" {
    pub fn TopologyRefinerFactory_TopologyDescriptor_Create(
        descriptor: *const crate::OpenSubdiv_v3_4_4_Far_TopologyDescriptor,
        options: crate::OpenSubdiv_v3_4_4_Far_TopologyRefinerFactory_Options,
    ) -> *mut crate::OpenSubdiv_v3_4_4_Far_TopologyRefiner;
}
