extern crate cgmath;

mod beach_line;
mod event;
mod voronoi;

pub type Point = ::cgmath::Vector3<f64>;

pub use voronoi::{build, Visitor};
