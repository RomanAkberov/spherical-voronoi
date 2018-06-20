extern crate cgmath;

mod beach_line;
mod event;
mod relaxed;
mod voronoi;

pub type Point = ::cgmath::Vector3<f64>;

pub use voronoi::{build, Visitor};
pub use relaxed::build_relaxed;
