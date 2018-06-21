use cgmath::prelude::*;
use event::CellEvent;
use beach_line::{BeachLine, ArcId};
use super::Point;

#[derive(Default)]
struct Voronoi {
    cell_index: usize,
    cell_events: Vec<CellEvent>,
    beach: BeachLine,
}

impl Voronoi {
    fn begin(&mut self, points: &[Point]) {
        self.cell_events.extend(points.iter().map(CellEvent::new));
    }

    fn relax(&mut self, relaxer: &mut Relaxer) {
        for (cell_event, point) in self.cell_events.iter_mut().zip(relaxer.points.iter_mut()) {
            *cell_event = CellEvent::new(point);
            *point = Point::zero();
        }
        self.cell_index = 0;
        self.beach.clear();
    }

    fn build<V: Visitor>(&mut self, visitor: &mut V) {
        self.cell_events.sort_by(|s0, s1| s0.theta.partial_cmp(&s1.theta).unwrap());
        loop {
            let has_cells = self.cell_index < self.cell_events.len();
            let has_vertices = self.beach.has_vertices();
            if has_cells {
                if has_vertices && self.beach.top_theta() < self.cell_events[self.cell_index].theta {
                    self.handle_vertex_event(visitor);
                } else {
                    self.handle_cell_event();
                }
            } else if has_vertices {
                self.handle_vertex_event(visitor);
            } else {
                break;
            }
        }
    }

    fn handle_cell_event(&mut self) {
        let event = self.cell_events[self.cell_index];
        let arc = self.beach.insert(self.cell_index, &event);
        let neighbors = self.beach[arc].neighbors();
        self.attach_vertex(neighbors.prev);
        self.attach_vertex(neighbors.next);
        self.cell_index += 1;
    }

    fn handle_vertex_event<V: Visitor>(&mut self, visitor: &mut V) {
        let arc = self.beach.heap_pop();
        let neighbors = self.beach[arc].neighbors();
        if neighbors.prev != neighbors.next {
            visitor.visit(self.beach[arc].vertex, [
                self.beach[neighbors.prev].cell_index,
                self.beach[arc].cell_index,
                self.beach[neighbors.next].cell_index,
            ]);
            self.beach.remove(arc);
            self.attach_vertex(neighbors.prev);
            self.attach_vertex(neighbors.next);
        }
    }

    fn attach_vertex(&mut self, arc: ArcId) {
        let neighbors = self.beach[arc].neighbors();
        if neighbors.prev == neighbors.next {
            return;
        }
        let point = self.beach[arc].focus;
        let to_prev = self.beach[neighbors.prev].focus - point;
        let to_next = self.beach[neighbors.next].focus - point;
        let vertex = to_prev.cross(to_next).normalize();
        let theta = vertex.z.acos() + vertex.dot(point).acos();
        if theta < self.beach[neighbors.prev].theta && theta < self.beach[neighbors.next].theta {
            self.beach[arc].vertex = vertex;
            self.beach[arc].theta = theta;
            self.beach.heap_update(arc);
        }
    }
}

#[derive(Default)]
struct Relaxer {
    points: Vec<Point>,
}

impl Relaxer {
    fn new(num_points: usize) -> Self {
        Self { points: vec![Point::zero(); num_points] }
    }
}

impl Visitor for Relaxer {
    fn visit(&mut self, point: Point, cells: [usize; 3]) {
        self.points[cells[0]] += point;
        self.points[cells[1]] += point;
        self.points[cells[2]] += point;
    }
}

pub trait Visitor {
    fn visit(&mut self, point: Point, cells: [usize; 3]);
}

pub fn build<V: Visitor>(visitor: &mut V, points: &[Point], num_relaxations: usize) {
    let mut voronoi = Voronoi::default();
    let mut relaxer = Relaxer::new(points.len());
    voronoi.begin(points);
    for _ in 0 .. num_relaxations {
        voronoi.build(&mut relaxer);
        voronoi.relax(&mut relaxer);
    }
    voronoi.build(visitor);
}
