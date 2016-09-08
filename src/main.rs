extern crate spherical_voronoi;
extern crate nalgebra;
extern crate rand;

use std::io::prelude::*;
use std::fs::File;
use spherical_voronoi::diagram::Diagram;
use nalgebra::Vector3;
use rand::distributions::{IndependentSample, Range};

fn write_diagram(diagram: &Diagram, path: &str) -> std::io::Result<()> {
    let mut file = try!(File::create(path));
    try!(writeln!(file, "{} {} {}", diagram.vertices.len(), diagram.edges.len(), diagram.faces.len()));
    for vertex in diagram.vertices.iter() {
        try!(writeln!(file, "{} {} {}", vertex.point.x(), vertex.point.y(), vertex.point.z()));
    }
    for edge in diagram.edges.iter() {
        try!(writeln!(file, "{} {}", edge.vertex_ids.0, edge.vertex_ids.1));
    }
    for face in diagram.faces.iter() {
        try!(writeln!(file, "{} {} {}", face.point.x(), face.point.y(), face.point.z()));
        assert_eq!(face.vertex_ids.len(), face.edge_ids.len());
        try!(writeln!(file, "{}", face.vertex_ids.len()));
        for vertex_id in face.vertex_ids.iter() {
            assert!(*vertex_id < diagram.vertices.len());
            try!(write!(file, "{} ", vertex_id));
        }
        try!(writeln!(file, ""));
        for edge_id in face.edge_ids.iter() {
            assert!(*edge_id < diagram.edges.len());
            try!(write!(file, "{} ", edge_id));
        }
        try!(writeln!(file, ""));
    }
    Ok(())
}

fn main() {
    let mut rng = rand::thread_rng();
    let mut points = Vec::new();
    let range = Range::new(-1f64, 1f64);
    for _ in 0..1000 {
        points.push(Vector3::new(range.ind_sample(&mut rng), range.ind_sample(&mut rng), range.ind_sample(&mut rng)));
    }
    if let Ok(diagram) = spherical_voronoi::voronoi::generate_relaxed(&points, 3) {
        write_diagram(&diagram, "test.txt").expect("Something went wrong");
    }
}