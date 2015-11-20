use super::vdev;

// Storage pool allocator
pub struct Spa {
    name: String, // Pool name
    root_vdev: vdev::Vdev,
}
