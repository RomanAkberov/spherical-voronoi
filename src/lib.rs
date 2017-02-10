extern crate cgmath;
extern crate ideal;

mod events;
mod beach_line;
mod voronoi;
mod diagram;
mod point;

pub use diagram::*;
pub use point::Position;
pub use voronoi::build;
