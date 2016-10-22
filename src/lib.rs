extern crate cgmath;
extern crate diagram;
extern crate id;

mod events;
mod beach;
mod red_black_tree;
mod voronoi;
mod point;

pub use point::Point;
pub use diagram::{Diagram, Kind, Vertex, Edge, Face, VertexIter, EdgeIter, FaceIter};
pub use voronoi::{Position, Error, build, build_relaxed};