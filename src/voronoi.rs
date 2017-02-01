use std::cmp::Ordering;
use angle::Angle;
use point::Point;
use events::{Events, EventKind, Circle};
use beach::{Beach, Arc, ArcStart};
use diagram::{Diagram, Vertex, Cell};
use cgmath::InnerSpace;

#[derive(Default)]
struct Builder {
    events: Events,
    beach: Beach,
    diagram: Diagram,
    scan_theta: Angle,
    temporary: Vec<Vertex>,
}
        
impl Builder {
    fn new(points: &[Point]) -> Self {
        let mut builder = Builder::default();
        for &point in points {
            let cell = builder.diagram.add_cell(point);
            builder.events.add_site(cell, point.theta.value);
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
            //println!("{:#?}", event);
            self.scan_theta = Angle::from(event.theta);
            match event.kind {
                EventKind::Site(cell) => self.handle_site_event(cell),
                EventKind::Circle(circle) => self.handle_circle_event(circle),
            }
        }
    }

    fn reset(&mut self) {
        self.diagram.reset();
        for cell in self.diagram.cells() {
            self.events.add_site(cell, self.diagram.cell_point(cell).theta.value);
        }
        self.temporary.clear();
        self.events.clear();
        self.beach.clear();
        self.scan_theta = Angle::default();
    }

    fn arc_point(&self, arc: Arc) -> &Point {
        &self.diagram.cell_point(self.beach.cell(arc))
    }

    fn create_temporary(&mut self, arc0: Arc, arc1: Arc) {
        let index = self.temporary.len();
        self.temporary.push(Vertex::invalid());
        self.beach.set_start(arc0, ArcStart::Temporary(index));
        self.beach.set_start(arc1, ArcStart::Temporary(index));
    }

    fn handle_site_event(&mut self, cell: Cell) {
        if let Some(mut arc) = self.beach.root() {
            if self.beach.len() == 1 {
                self.beach.insert_after(Some(arc), cell);
                let arc0 = self.beach.first();
                let arc1 = self.beach.last();
                self.create_temporary(arc0, arc1);
                return;
            }
            let point = *self.diagram.cell_point(cell);
            let mut use_tree = true;
            loop {
                let prev = self.beach.prev(arc);
                let next = self.beach.next(arc);
                let start = self.arcs_intersection(prev, arc);
                let end = self.arcs_intersection(arc, next);
                match point.phi.is_in_range(start, end) {
                    Ordering::Less => {
                        if use_tree {
                            if let Some(left) = self.beach.left(arc) {
                                arc = left;
                            } else {
                                // the tree has failed us, do the linear search from now on.
                                arc = self.beach.last();
                                use_tree = false;
                            }
                        } else {
                            arc = self.beach.prev(arc);
                        }
                    },
                    Ordering::Greater => {
                        if use_tree {
                            if let Some(right) = self.beach.right(arc) {
                                arc = right;
                            } else {
                                // the tree has failed us, do the linear search from now on.
                                arc = self.beach.first();
                                use_tree = false;
                            }
                        } else {
                            arc = self.beach.next(arc);
                        }
                    },
                    Ordering::Equal => {
                        self.detach_circle(arc);
                        let twin = {
                            let cell = self.beach.cell(arc);
                            let a = if prev == self.beach.last() {
                                None
                            } else {
                                Some(prev)
                            };
                            self.beach.insert_after(a, cell)
                        };
                        let new_arc = self.beach.insert_after(Some(twin), cell);
                        self.create_temporary(twin, new_arc);
                        self.attach_circle(prev, twin, new_arc, point.theta.value);
                        self.attach_circle(new_arc, arc, next, point.theta.value);
                        if self.detach_circle(prev) {
                            let prev_prev = self.beach.prev(prev);
                            self.attach_circle(prev_prev, prev, twin, ::std::f64::MIN);
                        }
                        break;
                    }
                }
            }
        } else {
            self.beach.insert_after(None, cell);
        }
    }
    
    fn merge_arcs(&mut self, arc0: Arc, arc1: Arc, arc2: Arc, vertex: Option<Vertex>) {
        let theta = self.scan_theta.value;
        if self.attach_circle(arc0, arc1, arc2, theta) {
            if let Some(vertex) = vertex {
                self.beach.set_start(arc1, ArcStart::Vertex(vertex));
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
    
    fn handle_circle_event(&mut self, circle: Circle) {
        if self.events.is_invalid(circle) {
            return;
        }
        let arc = self.events.arc(circle);
        assert_eq!(self.beach.circle(arc), circle);
        let prev = self.beach.prev(arc);
        let next = self.beach.next(arc);
        self.detach_circle(prev);
        self.detach_circle(next);
        let point = self.events.center(circle);
        let vertex = self.diagram.add_vertex(point, &[self.beach.cell(prev), self.beach.cell(arc), self.beach.cell(next)]);
        self.create_edge(prev, vertex);
        self.create_edge(arc, vertex);
        self.beach.remove(arc);
        if self.beach.prev(prev) == next {
            self.create_edge(next, vertex);
            self.beach.remove(prev);
            self.beach.remove(next);
        } else {
            let prev_prev = self.beach.prev(prev);
            let next_next = self.beach.next(next);
            self.merge_arcs(prev_prev, prev, next, Some(vertex));
            self.merge_arcs(prev, next, next_next, None);
        }
    }
    
    fn arcs_intersection(&self, arc0: Arc, arc1: Arc) -> f64 {
        let point0 = self.arc_point(arc0);
        let point1 = self.arc_point(arc1);
        let u1 = (self.scan_theta.cos - point1.theta.cos) * point0.theta.sin;
        let u2 = (self.scan_theta.cos - point0.theta.cos) * point1.theta.sin;
        let a = u1 * point0.phi.cos - u2 * point1.phi.cos;
        let b = u1 * point0.phi.sin - u2 * point1.phi.sin;
        let c = (point0.theta.cos - point1.theta.cos) * self.scan_theta.sin;
        let length = (a * a + b * b).sqrt();
        let gamma = a.atan2(b);
        let phi_plus_gamma = (c / length).asin();
        Angle::wrap(phi_plus_gamma - gamma)
    }
    
    fn attach_circle(&mut self, arc0: Arc, arc1: Arc, arc2: Arc, min_theta: f64) -> bool {
        let p1 = self.arc_point(arc1).position;
        let p01 = self.arc_point(arc0).position - p1;
        let p21 = self.arc_point(arc2).position - p1;
        let center = p01.cross(p21).normalize();
        let radius = center.dot(p1).acos();
        let theta = center.z.acos() + radius;
        if theta >= min_theta {
            let circle = self.events.add_circle(arc1, center, theta);
            self.beach.set_circle(arc1, circle);
            true
        } else {
            false
        }
    }
    
    fn detach_circle(&mut self, arc: Arc) -> bool {
        let circle = self.beach.circle(arc);
        if circle.is_valid() {
            self.events.set_invalid(circle, true);
            self.beach.set_circle(arc, Circle::invalid());
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
