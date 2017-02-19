use std::iter::Map;
use std::vec::IntoIter;
use cgmath::Zero;
use ideal::{Id, IdVec};
use diagram::{Diagram, Vertex, Cell};
use ::Position;

pub trait Generator: Default {
    type Result;

    fn result(self) -> Self::Result;
    fn vertex(&mut self, position: Position, cell0: Cell, cell1: Cell, cell2: Cell) -> Vertex;
    fn start(&mut self, index: usize, vertex: Vertex);
    fn temporary(&mut self, index: usize, prev: usize);
    fn edge(&mut self, index: usize, end: Vertex);
    fn cell(&mut self) -> Cell;
}

pub struct Centroid {
    sum: Position,
    count: f64,
}

impl Centroid {
    fn new() -> Self {
        Centroid {
            sum: Position::zero(),
            count: 1.0,
        }
    }

    pub fn position(self) -> Position {
        self.sum / self.count
    }
}

pub type Centroids = Map<IntoIter<Centroid>, fn(Centroid) -> Position>;

#[derive(Default)]
pub struct CentroidGenerator {
    centroids: Vec<Centroid>,
}

impl CentroidGenerator {
    fn add_to_centroid(&mut self, cell: Cell, position: Position) {
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

    fn vertex(&mut self, position: Position, cell0: Cell, cell1: Cell, cell2: Cell) -> Vertex {
        self.add_to_centroid(cell0, position);
        self.add_to_centroid(cell1, position);
        self.add_to_centroid(cell2, position);
        Vertex::invalid()
    }

    fn start(&mut self, _: usize, _: Vertex) {}
    fn temporary(&mut self, _: usize, _: usize) {}
    fn edge(&mut self, _: usize, _: Vertex) {}

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

    fn set_start(&mut self, index: usize, start: Id<Vertex>) {
        if index < self.arc_starts.len() {
            self.arc_starts[index] = start;
        } else {
            assert_eq!(self.arc_starts.len(), index);
            self.arc_starts.push(start);
        }
    }
}

impl Generator for DiagramGenerator {
    type Result = Diagram;

    fn result(self) -> Self::Result {
        self.diagram
    }

    fn vertex(&mut self, position: Position, cell0: Cell, cell1: Cell, cell2: Cell) -> Vertex {
        self.diagram.add_vertex(position, [cell0, cell1, cell2])
    }

    fn start(&mut self, index: usize, vertex: Vertex) {
        self.arc_starts[index] = self.start_vertices.push(vertex);
    }

    fn temporary(&mut self, index: usize, prev: usize) {
        if index != prev {
            let start = self.start_vertices.push(Vertex::invalid());
            if index < prev {
                self.set_start(index, start);
                self.set_start(prev, start);
            } else {
                self.set_start(prev, start);
                self.set_start(index, start);
            }
        } else {
            self.set_start(index, Id::invalid());
        }
    }

    fn edge(&mut self, index: usize, end: Vertex) {
        let start = self.arc_starts[index];
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
