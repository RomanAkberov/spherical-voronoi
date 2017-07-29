use cgmath::prelude::*;
use voronoi;
use super::Point;

#[derive(Default)]
struct CentroidVisitor {
    points: Vec<Point>,
}

impl voronoi::Visitor for CentroidVisitor {
    fn vertex(&mut self, point: [f64; 3], cells: [usize; 3]) {
        let point = Point::from(point);
        self.points[cells[0]] += point;
        self.points[cells[1]] += point;
        self.points[cells[2]] += point;
    }
    
    fn edge(&mut self, _vertices: [usize; 2]) {

    }

    fn cell(&mut self) {
        self.points.push(Point::zero())
    }
}

pub fn build_relaxed<P: AsRef<[f64; 3]>, V: voronoi::Visitor>(points: &[P], visitor: &mut V, relaxations: usize) {
    if relaxations > 0 {
        let mut centroids = CentroidVisitor::default();
        voronoi::build(points, &mut centroids);
        build_relaxed(&centroids.points, visitor, relaxations - 1);
    } else {
        voronoi::build(points, visitor);
    }
}