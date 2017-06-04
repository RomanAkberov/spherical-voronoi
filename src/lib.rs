extern crate cgmath;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate ideal;
mod event;
mod beach_line;
mod builder;
mod relaxed;

pub type Position = cgmath::Vector3<f64>;
pub use builder::{Builder, Item};
pub use relaxed::build_relaxed;
