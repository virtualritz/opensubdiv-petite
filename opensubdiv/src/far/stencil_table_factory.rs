use super::stencil_table::StencilTable;
use super::topology_refiner::TopologyRefiner;
use opensubdiv_sys as sys;

pub use sys::far::stencil_table_factory::{Options, OptionsBuilder};

/// Short function for creating an [OptionsBuilder]
pub fn options() -> OptionsBuilder {
    OptionsBuilder::new()
}
/// Instantiates a [StencilTable] from a [TopologyRefiner] that has been
/// refined uniformly or adaptively.
///
/// The factory only creates stencils for vertices that have already been
/// refined in the TopologyRefiner. Use [TopologyRefiner::refine_uniform()] or
/// [TopologyRefiner::refine_adaptive()] before constructing the stencils.
///
/// ## Parameters
/// * `refiner` - The [TopologyRefiner] containing the topology.
/// * `options` -  [Options] controlling the creation of the table.
pub fn create(
    refiner: &TopologyRefiner,
    options: sys::far::stencil_table_factory::Options,
) -> StencilTable {
    let ptr =
        unsafe { sys::far::StencilTableFactory_Create(refiner.ptr, options) };

    if ptr.is_null() {
        panic!("StencilTableFactory_Create() returned null");
    }

    StencilTable { ptr }
}
