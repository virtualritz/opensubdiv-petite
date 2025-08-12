//! Error types for the opensubdiv-petite crate.

use thiserror::Error;

/// Main error type for opensubdiv-petite operations.
#[derive(Debug, Error)]
pub enum Error {
    /// Failed to create a topology refiner.
    #[error("Failed to create topology refiner")]
    CreateTopologyRefinerFailed,

    /// Failed to create a stencil table.
    #[error("Failed to create stencil table")]
    StencilTableCreation,

    /// Failed to create a patch table.
    #[error("Failed to create patch table")]
    PatchTableCreation,

    /// Stencil evaluation failed.
    #[error("Stencil evaluation failed")]
    EvalStencilsFailed,

    /// Invalid topology descriptor.
    #[error("Invalid topology descriptor: {0}")]
    InvalidTopology(String),

    /// Invalid patch configuration.
    #[error("Invalid patch configuration: {0}")]
    InvalidPatch(String),

    /// Index out of bounds.
    #[error("Index {index} out of bounds (max: {max})")]
    IndexOutOfBounds { index: usize, max: usize },

    /// Invalid buffer size.
    #[error("Invalid buffer size: expected {expected}, got {actual}")]
    InvalidBufferSize { expected: usize, actual: usize },

    /// FFI error from OpenSubdiv C++ library.
    #[error("OpenSubdiv FFI error: {0}")]
    Ffi(String),

    /// Null pointer encountered where non-null was expected.
    #[error("Unexpected null pointer")]
    NullPointer,

    /// Feature not available or not compiled in.
    #[error("Feature not available: {0}")]
    FeatureNotAvailable(String),

    /// GPU backend error.
    #[cfg(any(feature = "cuda", feature = "opencl", feature = "metal"))]
    #[error("GPU backend error: {0}")]
    GpuBackend(String),

    /// Truck integration error.
    #[cfg(feature = "truck")]
    #[error("Truck integration error: {0}")]
    TruckIntegration(#[from] crate::truck_integration::TruckIntegrationError),

    /// IO error for file operations.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Format error for export operations.
    #[error("Format error: {0}")]
    Format(#[from] std::fmt::Error),
}

/// Result type alias using our Error type.
pub type Result<T> = std::result::Result<T, Error>;

/// Convert from a null pointer to an error.
impl Error {
    /// Create an error from a null pointer check.
    pub fn from_null_ptr<T>(ptr: *const T, context: &str) -> Self {
        if ptr.is_null() {
            Error::NullPointer
        } else {
            Error::Ffi(format!("Unexpected error in {}", context))
        }
    }

    /// Check if a pointer is null and return an error if it is.
    pub fn check_null_ptr<T>(ptr: *const T, context: &str) -> Result<()> {
        if ptr.is_null() {
            Err(Error::Ffi(format!("Null pointer in {}", context)))
        } else {
            Ok(())
        }
    }
}