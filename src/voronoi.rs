use std::collections::BinaryHeap;
use cgmath::InnerSpace;
use ideal::{Id, IdVec};
use point::Point;
use events::{Event, EventKind};
use beach::{Beach, Arc, Start};
use diagram::{Diagram, Vertex, Cell};

#[derive(Default)]
struct Builder {
    events: BinaryHeap<Event>,
    beach: Beach,
    diagram: Diagram,
    starts: IdVec<Start>,
}

impl Builder {
    fn new(points: &[Point]) -> Self {
        let mut builder = Builder::default();
        for &point in points {
            let cell = builder.diagram.add_cell(point);
            builder.events.push(Event {
                theta: point.theta.value,
                kind: EventKind::Site(cell),
            });
        }
        builder
    }
    
    pub fn build(mut self, relaxations: usize) -> Diagram {
        self.build_iter();
        for _ in 1..relaxations {
            self.reset();
            self.build_iter();
        }
        self.finish();
        self.diagram
    }   

    fn build_iter(&mut self) {
        while let Some(event) = self.events.pop() {
            //println!("{:?}", event);
            match event.kind {
                EventKind::Site(cell) => self.site_event(cell),
                EventKind::Circle(arc) => self.circle_event(arc, event.theta),
            }
        }
    }

    fn reset(&mut self) {
        self.diagram.reset();
        self.events.clear();
        self.starts.clear();
        self.beach.clear();
        for cell in self.diagram.cells() {
            self.events.push(Event {
                theta: self.diagram.cell_point(cell).theta.value,
                kind: EventKind::Site(cell)
            });
        }
    }

    fn arc_point(&self, arc: Arc) -> &Point {
        self.diagram.cell_point(self.beach.cell(arc))
    }

    fn create_temporary(&mut self, arc0: Arc, arc1: Arc) {
        let start = self.starts.push(Start { vertex: Id::invalid() });
        self.beach.set_start(arc0, start);
        self.beach.set_start(arc1, start);
    }

    fn site_event(&mut self, cell: Cell) {
        let arc = self.beach.insert(cell, &self.diagram);
        let (prev, next) = self.beach.neighbors(arc);
        if prev != arc {
            self.create_temporary(prev, arc);
            if prev != next {
                let theta = self.diagram.cell_point(cell).theta.value;
                self.attach_circle(prev, theta);
                self.attach_circle(next, theta);
            }
        }
    }

    fn circle_event(&mut self, arc: Arc, theta: f64) {
        if !self.beach.is_valid(arc) {
            return;
        }
        let (prev, next) = self.beach.neighbors(arc);
        self.beach.detach(arc);
        self.beach.detach(prev);
        self.beach.detach(next);
        let point = self.beach.center(arc);
        let vertex = self.diagram.add_vertex(point, [self.beach.cell(prev), self.beach.cell(arc), self.beach.cell(next)]);
        self.create_edge(prev, vertex);
        self.create_edge(arc, vertex);
        self.beach.remove(arc);
        if self.beach.prev(prev) == next {
            self.create_edge(next, vertex);
            self.beach.remove(prev);
            self.beach.remove(next);
        } else {
            if self.attach_circle(prev, theta) {
                let start = self.starts.push(Start { vertex: vertex });
                self.beach.set_start(prev, start);
            }
            self.attach_circle(next, theta);
        }
    }
    
    fn create_edge(&mut self, arc: Arc, end: Vertex) {
        let start = self.beach.start(arc);
        if start.is_valid() {
            let vertex = self.starts[start].vertex;
            if vertex.is_valid() {
                self.diagram.add_edge(vertex, end);
            } else {
                self.starts[start].vertex = end;
            }
        }
    }
    
    fn attach_circle(&mut self, arc: Arc, min: f64) -> bool {
        let (prev, next) = self.beach.neighbors(arc);
        let position = self.arc_point(arc).position;
        let from_prev = self.arc_point(prev).position - position;
        let from_next = self.arc_point(next).position - position;
        let center = from_prev.cross(from_next).normalize();
        let theta = center.z.acos() + center.dot(position).acos();
        if theta >= min {
            self.beach.attach(arc, center);
            self.events.push(Event {
                theta: theta,
                kind: EventKind::Circle(arc),
            });
            true
        } else {
            false
        }
    }
    
    fn finish(&mut self) {
        for edge in self.diagram.edges() {
            let mut common = Vec::new(); 
            let (vertex0, vertex1) = self.diagram.edge_vertices(edge);  
            for &cell0 in self.diagram.vertex_cells(vertex0) {
                for &cell1 in self.diagram.vertex_cells(vertex1) {
                    if cell0 == cell1 {
                        common.push(cell0);
                    }
                }
            }
            assert_eq!(common.len(), 2);
            self.diagram.set_edge_cells(edge, common[0], common[1]);
        }
        for vertex in self.diagram.vertices() {
            assert_eq!(self.diagram.vertex_cells(vertex).len(), 3);
            assert_eq!(self.diagram.vertex_edges(vertex).len(), 3);
        }
    }
}

pub fn build(points: &[Point], relaxations: usize) -> Diagram {
    Builder::new(points).build(relaxations)
}
