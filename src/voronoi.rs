use std::collections::BinaryHeap;
use point::Point;
use events::{Event, EventKind};
use beach::{Beach, Arc, ArcStart};
use diagram::{Diagram, Vertex, Cell};
use cgmath::InnerSpace;

#[derive(Default)]
struct Builder {
    events: BinaryHeap<Event>,
    beach: Beach,
    diagram: Diagram,
    scan_theta: f64,
    temporary: Vec<Vertex>,
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
        self.temporary.clear();
        self.beach.clear();
        self.scan_theta = 0.0;
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
        let index = self.temporary.len();
        self.temporary.push(Vertex::invalid());
        self.beach.set_start(arc0, ArcStart::Temporary(index));
        self.beach.set_start(arc1, ArcStart::Temporary(index));
    }

    fn site_event(&mut self, cell: Cell) {
        let point = *self.diagram.cell_point(cell);
        let arc = self.beach.insert(cell, point, &self.diagram);
        let (prev, next) = self.beach.neighbors(arc);
        if prev != arc {
            self.create_temporary(prev, arc);
            if prev != next {
                self.attach_circle(prev, point.theta.value);
                self.attach_circle(next, point.theta.value);
            }
        }
    }

    fn circle_event(&mut self, arc: Arc, theta: f64) {
        if !self.beach.is_valid(arc) {
            return;
        }
        self.scan_theta = theta;
        let (prev, next) = self.beach.neighbors(arc);
        self.beach.detach(arc);
        self.beach.detach(prev);
        self.beach.detach(next);
        let point = self.beach.center(arc);
        let vertex = self.diagram.add_vertex(point, &[self.beach.cell(prev), self.beach.cell(arc), self.beach.cell(next)]);
        self.create_edge(prev, vertex);
        self.create_edge(arc, vertex);
        self.beach.remove(arc);
        if self.beach.prev(prev) == next {
            self.create_edge(next, vertex);
            self.beach.remove(prev);
            self.beach.remove(next);
        } else {
            self.merge_arcs(prev, Some(vertex));
            self.merge_arcs(next, None);
        }
    }
    
    fn merge_arcs(&mut self, arc: Arc, vertex: Option<Vertex>) {
        let theta = self.scan_theta;
        if self.attach_circle(arc, theta) {
            if let Some(vertex) = vertex {
                self.beach.set_start(arc, ArcStart::Vertex(vertex));
            }
        }
    }
    
    fn create_edge(&mut self, arc: Arc, end: Vertex) {
        match self.beach.start(arc) {
            ArcStart::Temporary(index) => {
                let start = self.temporary[index];
                if start.is_invalid() {
                    self.temporary[index] = end;
                } else {
                    self.diagram.add_edge(start, end);
                }
            },
            ArcStart::Vertex(start) => {
                self.diagram.add_edge(start, end);
            },
            ArcStart::None => {},
        };
    }
    
    fn attach_circle(&mut self, arc: Arc, min_theta: f64) -> bool {
        let (prev, next) = self.beach.neighbors(arc);
        let position = self.arc_point(arc).position;
        let from_prev = self.arc_point(prev).position - position;
        let from_next = self.arc_point(next).position - position;
        let center = from_prev.cross(from_next).normalize();
        let radius = center.dot(position).acos();
        let theta = center.z.acos() + radius;
        if theta >= min_theta {
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
    }
}

pub fn build(points: &[Point], relaxations: usize) -> Diagram {
    Builder::new(points).build(relaxations)
}
