use std::iter::Map;
use std::vec::IntoIter;
use cgmath::{Vector3, Zero};
use ideal::{Id, IdVec};
use diagram::{Diagram, Vertex, Cell};
use beach_line::Arc;

pub struct Centroid {
    sum: Vector3<f64>,
    count: f64,
}

impl Centroid {
    fn new() -> Self {
        Centroid { sum: Vector3::zero(), count: 1.0 }
    }

    pub fn position(self) -> Vector3<f64> {
        self.sum / self.count
    }
}

pub type Centroids = Map<IntoIter<Centroid>, fn(Centroid) -> Vector3<f64>>;

pub trait Generator: Default {
    type Result;

    fn result(self) -> Self::Result;
    fn vertex(&mut self, position: Vector3<f64>, cell0: Cell, cell1: Cell, cell2: Cell) -> Vertex;
    fn start(&mut self, arc: Arc, vertex: Vertex);
    fn temporary(&mut self, arc: Arc, prev: Arc);
    fn edge(&mut self, arc: Arc, end: Vertex);
    fn cell(&mut self) -> Cell;
}

#[derive(Default)]
pub struct CentroidGenerator {
    centroids: Vec<Centroid>,
}

impl CentroidGenerator {
    fn add_to_centroid(&mut self, cell: Cell, position: Vector3<f64>) {
        let centroid = &mut self.centroids[cell.index()];
        centroid.count += 1.0;
        centroid.sum += position; 
    }
}

impl Generator for CentroidGenerator {
    type Result = Centroids;

    fn result(self) -> Self::Result {
        self.centroids.into_iter().map(Centroid::position)
    }

    fn vertex(&mut self, position: Vector3<f64>, cell0: Cell, cell1: Cell, cell2: Cell) -> Vertex {
        self.add_to_centroid(cell0, position);
        self.add_to_centroid(cell1, position);
        self.add_to_centroid(cell2, position);
        Vertex::invalid()
    }

    fn start(&mut self, _: Arc, _: Vertex) {}

    fn temporary(&mut self, _: Arc, _: Arc) {}

    fn edge(&mut self, _: Arc, _: Vertex) {}

    fn cell(&mut self) -> Cell {
        self.centroids.push(Centroid::new());
        Cell::from(self.centroids.len() - 1)
    }
}

#[derive(Default)]
pub struct DiagramGenerator {
    diagram: Diagram,
    arc_starts: Vec<Id<Vertex>>,
    start_vertices: IdVec<Vertex>,
}

impl DiagramGenerator {
    fn common_cells(&self, vertex0: Vertex, vertex1: Vertex) -> (Cell, Cell) {
        let mut cells = (Cell::invalid(), Cell::invalid());
        for &cell in self.diagram.vertex_cells(vertex0) {
            for &other_cell in self.diagram.vertex_cells(vertex1) {
                if cell == other_cell {
                    if cells.0.is_invalid() {
                        cells.0 = cell; 
                    } else {
                        cells.1 = cell;
                    }
                }
            }
        }
        cells
    }
}

impl Generator for DiagramGenerator {
    type Result = Diagram;

    fn result(self) -> Self::Result {
        self.diagram
    }

    fn vertex(&mut self, position: Vector3<f64>, cell0: Cell, cell1: Cell, cell2: Cell) -> Vertex {
        self.diagram.add_vertex(position, [cell0, cell1, cell2])
    }

    fn start(&mut self, arc: Arc, vertex: Vertex) {
        let start = self.start_vertices.push(vertex);
        self.arc_starts[arc.index()] = start;
    }

    fn temporary(&mut self, arc: Arc, prev: Arc) {
        if arc != prev {
            let start = self.start_vertices.push(Vertex::invalid());
            // if `arc` and `prev` are the only arcs on the beach, then `prev` is not new.
            if arc > prev {
                self.arc_starts[prev.index()] = start;
            } else {
                self.arc_starts.push(start);
            }
            self.arc_starts.push(start);
        } else {
            self.arc_starts.push(Id::invalid());
        }
    }

    fn edge(&mut self, arc: Arc, end: Vertex) {
        let start = self.arc_starts[arc.index()];
        if start.is_valid() {
            let vertex = self.start_vertices[start];
            if vertex.is_valid() {
                let (cell0, cell1) = self.common_cells(vertex, end);
                self.diagram.add_edge(vertex, end, cell0, cell1);
            } else {
                self.start_vertices[start] = end;
            }
        }
    }

    fn cell(&mut self) -> Cell {
        self.diagram.add_cell()
    }
}
