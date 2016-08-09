pub use self::kscheme::KScheme;
pub use self::resource::{Resource, ResourceSeek};
pub use self::scheme::Scheme;
pub use self::slice_resource::{SliceResource, SliceMutResource};
pub use self::vec_resource::VecResource;
pub use self::supervisor_resource::SupervisorResource;

/// Kernel schemes
pub mod kscheme;
/// Internal resource representation
pub mod resource;
/// Userspace scheme
pub mod scheme;
/// Static slice resource
pub mod slice_resource;
/// Vec resource
pub mod vec_resource;
/// Supervisor resource.
pub mod supervisor_resource;
