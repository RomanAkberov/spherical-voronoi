extern crate cgmath;
extern crate ideal;

mod angle;
mod events;
mod beach;
mod red_black_tree;
mod voronoi;
pub mod diagram;
pub mod point;

pub use voronoi::{Error, build};