//! Far is the primary API layer for processing client-supplied mesh data into
//! subdivided surfaces.
//!
//! The Far interface may be used directly and also may be used to prepare mesh
//! data for further processing by Osd. The two main aspects of the subdivision
//! process are Topology Refinement and Primvar Refinement.
//!
//! ## Topology Refinement
//! Topology refinement is the process of splitting the mesh topology according
//! to the specified subdivison rules to generate new topological vertices,
//! edges, and faces. This process is purely topological and does not depend on
//! the speciific values of any primvar data (point positions, etc).
//! Topology refinement can be either uniform or adaptive, where extraordinary
//! features are automatically isolated (see feature adaptive subdivision).
//! The far topology structs present a public interface for the refinement
//! functionality provided in [vtr],
//!
//! The main classes in Far related to topology refinement are:
//!
//! * [`TopologyRefiner`](crate::far::topology_refiner::TopologyRefiner) -
//!   Encapsulates mesh refinement.
//! * [`TopologyLevel`](crate::far::topology_level::TopologyLevel) - Representis
//!   one level of refinement within a
//!   [`TopologyRefiner`](crate::far::topology_refiner::TopologyRefiner).
//! * [`TopologyRefinerFactory`](crate::far::topology_refiner_factory::
//!   TopologyRefinerFactory) - A factory specialized in terms of the
//!   application's mesh representation used to construct
//! [`TopologyRefiner`](crate::far::topology_refiner::TopologyRefiner)
//! instances.
//!
//! ## Primvar Refinement
//! Primvar refinement is the process of computing values for primvar data
//! (points, colors, normals, texture coordinates, etc) by applying weights
//! determined by the specified subdivision rules. There are many advantages
//! gained by distinguishing between topology refinement and primvar
//! interpolation including the ability to apply a single static topological
//! refinement to multiple primvar instances or to different animated primvar
//! time samples. Far supports methods to refine primvar data at the locations
//! of topological vertices and at arbitrary locations on the
//! subdivision limit surface. The main classes in Far related to primvar
//! refinement are:
//! * [`PrimvarRefiner`] –    A class implementing refinement of primvar data at
//!   the locations of topological vertices.
//! * [`PatchTable`] –        A representation of the refined surface topology
//!   that can be used for efficient evaluation of primvar data at arbitrary
//!   locations.
//! * [`StencilTable`] –      A representation of refinement weights suitable
//!   for efficient parallel processing of primvar refinement.
//! * [`LimitStencilTable`] – A representation of refinement weights suitable
//!   for efficient parallel processing of primvar refinement at arbitrary limit
//!   surface locations.
pub mod topology_descriptor;
pub use topology_descriptor::*;

pub mod topology_level;
pub use topology_level::*;

pub mod topology_refiner;
pub use topology_refiner::*;

pub mod topology_refiner_factory;
pub use topology_refiner_factory::*;

pub mod stencil_table;
pub use stencil_table::*;

pub mod stencil_table_factory;
pub use stencil_table_factory::*;

pub mod primvar_refiner;
pub use primvar_refiner::*;
