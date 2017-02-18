extern crate cgmath;
extern crate ideal;

mod event;
mod generator;
mod beach_line;
mod voronoi;
mod diagram;

pub use diagram::*;
pub type F = f64;
pub type Position = ::cgmath::Vector3<F>;

pub fn build<I: IntoIterator<Item=Position>>(positions: I, relaxations: usize) -> Diagram {
    if relaxations == 0 {
        voronoi::build::<generator::DiagramGenerator, I>(positions)
    } else {
        let mut centroids = voronoi::build::<generator::CentroidGenerator, I>(positions);
        for _ in 0..relaxations - 2 {
            centroids = voronoi::build::<generator::CentroidGenerator, generator::Centroids>(centroids);
        }
        voronoi::build::<generator::DiagramGenerator, generator::Centroids>(centroids)
    }
}
