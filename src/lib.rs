extern crate cgmath;
extern crate diagram;
extern crate ideal;

mod angle;
mod events;
mod beach;
mod red_black_tree;
mod voronoi;
mod point;

pub use point::Point;
pub use diagram::*;
pub use voronoi::{Position, Error, build, build_relaxed};