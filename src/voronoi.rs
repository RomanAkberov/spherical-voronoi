use std::collections::BinaryHeap;
use cgmath::{Vector3, InnerSpace};
use ideal::IdVec;
use event::{SiteEvent, CircleEvent};
use beach_line::{BeachLine, Arc};
use diagram::{Diagram, Vertex, Cell};

struct Center {
    sum: Vector3<f64>,
    count: f64,
}

#[derive(Default)]
struct Builder {
    circle_events: BinaryHeap<CircleEvent>,
    site_events: Vec<SiteEvent>,
    beach: BeachLine,
    diagram: Diagram,
    starts: IdVec<Vertex>,
    centers: Vec<Center>,
    is_final: bool,
}

impl Builder {
    fn new(positions: &[Vector3<f64>]) -> Self {
        let mut builder = Builder::default();
        for &position in positions {
            builder.site_events.push(SiteEvent::from(position));
        }
        builder
    }
    
    fn build(mut self, relaxations: usize) -> Diagram {
        self.sweep(relaxations == 0);
        for i in 1..relaxations {
            self.reset();
            self.sweep(i == relaxations - 1);
        }
        self.diagram
    }   

    fn sweep(&mut self, is_final: bool) {
        self.is_final = is_final;
        self.site_events.sort();
        loop {
            match (self.diagram.cells().len() >= self.site_events.len(), self.circle_events.is_empty()) {
                (true, true) => break,
                (true, false) => self.circle_event(),
                (false, true) => self.site_event(),
                (false, false) => if self.site_events[self.diagram.cells().len()].theta.value < self.circle_events.peek().unwrap().theta {
                    self.site_event()
                } else {
                    self.circle_event()
                }
            }
        }
    }

    fn reset(&mut self) {
        self.site_events.clear();
        self.circle_events.clear();
        self.starts.clear();
        self.beach.clear();
        for center in &self.centers {
            self.site_events.push(SiteEvent::from(center.sum / center.count));
        }
        self.centers.clear();
        self.diagram.clear();
    }

    fn site_event(&mut self) {
        let theta = self.site_events[self.diagram.cells().len()].theta.value;
        let cell = self.diagram.add_cell();
        self.centers.push(Center {
            sum: Vector3::new(0.0, 0.0, 0.0),
            count: 0.0,
        });
        let arc = self.beach.insert(cell, &self.site_events);
        let (prev, next) = self.beach.neighbors(arc);
        if prev != arc {
            self.create_temporary(prev, arc);
            if prev != next {
                self.attach_circle(prev, theta);
                self.attach_circle(next, theta);
            }
        }
    }

    fn circle_event(&mut self) {
        let CircleEvent { arc, theta } = self.circle_events.pop().unwrap();
        if let Some(center) = self.beach.center(arc) {
            let (prev, next) = self.beach.neighbors(arc);
            self.beach.detach_circle(arc);
            self.beach.detach_circle(prev);
            self.beach.detach_circle(next);
            let vertex = self.create_vertex(center, prev, arc, next);
            self.beach.remove(arc);
            if self.beach.prev(prev) == next {
                self.create_edge(next, vertex);
                self.beach.remove(prev);
                self.beach.remove(next);
            } else {
                if self.attach_circle(prev, theta) {
                    if self.is_final {
                        let start = self.starts.push(vertex);
                        self.beach.set_start(prev, start);
                    }
                }
                self.attach_circle(next, theta);
            }
        }
    }

    fn create_temporary(&mut self, arc0: Arc, arc1: Arc) {
        if self.is_final {
            let start = self.starts.push(Vertex::invalid());
            self.beach.set_start(arc0, start);
            self.beach.set_start(arc1, start);
        }
    }

    fn create_edge(&mut self, arc: Arc, end: Vertex) {
        if self.is_final {
            let start = self.beach.start(arc);
            if start.is_valid() {
                let vertex = self.starts[start];
                if vertex.is_valid() {
                    let (cell0, cell1) = self.common_cells(vertex, end);
                    self.diagram.add_edge(vertex, end, cell0, cell1);
                } else {
                    self.starts[start] = end;
                }
            }
        }
    }
    
    fn attach_circle(&mut self, arc: Arc, min: f64) -> bool {
        let (prev, next) = self.beach.neighbors(arc);
        let position = self.arc_position(arc);
        let from_prev = self.arc_position(prev) - position;
        let from_next = self.arc_position(next) - position;
        let center = from_prev.cross(from_next).normalize();
        let theta = center.z.acos() + center.dot(position).acos();
        if theta >= min {
            self.beach.attach_circle(arc, center);
            self.circle_events.push(CircleEvent {
                theta: theta,
                arc: arc,
            });
            true
        } else {
            false
        }
    }
    
    fn arc_position(&self, arc: Arc) -> Vector3<f64> {
        self.site_events[self.beach.cell(arc).index()].position
    }

    fn common_cells(&self, vertex0: Vertex, vertex1: Vertex) -> (Cell, Cell) {
        let mut cells = (Cell::invalid(), Cell::invalid());
        for &cell0 in self.diagram.vertex_cells(vertex0) {
            for &cell1 in self.diagram.vertex_cells(vertex1) {
                if cell0 == cell1 {
                    if cells.0.is_invalid() {
                        cells.0 = cell0; 
                    } else {
                        cells.1 = cell0;
                    }
                }
            }
        }
        cells
    }

    fn create_vertex(&mut self, center: Vector3<f64>, prev: Arc, arc: Arc, next: Arc) -> Vertex {
        if self.is_final {
            let vertex = self.diagram.add_vertex(center, [self.beach.cell(prev), self.beach.cell(arc), self.beach.cell(next)]);
            self.create_edge(prev, vertex);
            self.create_edge(arc, vertex);
            vertex
        } else {
            self.add_to_center(center, prev);
            self.add_to_center(center, arc);
            self.add_to_center(center, next);
            Vertex::invalid()
        }
    }

    fn add_to_center(&mut self, position: Vector3<f64>, arc: Arc) {
        let center = &mut self.centers[self.beach.cell(arc).index()];
        center.count += 1.0;
        center.sum += position;
    }
}

pub fn build(positions: &[Vector3<f64>], relaxations: usize) -> Diagram {
    Builder::new(positions).build(relaxations)
}
