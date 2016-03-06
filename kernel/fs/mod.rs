pub mod redoxfs;

pub use self::kscheme::KScheme;
pub use self::resource::{Resource, ResourceSeek};
pub use self::scheme::Scheme;
pub use self::url::{Url, OwnedUrl};
pub use self::vec_resource::VecResource;

/// Kernel schemes
pub mod kscheme;
/// Internal resource representation
pub mod resource;
/// Userspace scheme
pub mod scheme;
/// URL
pub mod url;
/// Default resource
pub mod vec_resource;
