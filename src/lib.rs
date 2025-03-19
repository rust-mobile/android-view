#![deny(unsafe_op_in_unsafe_fn)]

pub use jni;
pub use ndk;

mod context;
pub use context::*;
mod events;
pub use events::*;
mod graphics;
pub use graphics::*;
mod ime;
pub use ime::*;
mod surface;
pub use surface::*;
mod view;
pub use view::*;
