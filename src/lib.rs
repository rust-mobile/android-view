#![deny(unsafe_op_in_unsafe_fn)]

pub use jni;
pub use ndk;

mod util;

mod binder;
pub use binder::*;
mod context;
pub use context::*;
mod events;
pub use events::*;
mod graphics;
pub use graphics::*;
mod ime;
pub use ime::*;
mod peer_result;
pub use peer_result::*;
mod surface;
pub use surface::*;
mod view;
pub use view::*;
