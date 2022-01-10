pub mod ralloc;
pub mod register_allocation;
pub mod register_allocators;
pub mod register_class;
pub mod register_interface;
pub mod register_location;
pub use self::ralloc::*;
pub use self::register_allocation::*;
pub use self::register_class::*;
pub use self::register_location::*;
pub use register_interface::*;

pub use register_allocators::*;
