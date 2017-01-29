use std::cmp::Ordering;
use angle::Angle;
use point::Point;
use events::{Events, EventKind, Circle};
use beach::{Beach, Arc, ArcStart};
use diagram::{Diagram, Vertex, Face};

struct Builder {
    events: Events,
    beach: Beach,
    diagram: Diagram,
    scan_theta: Angle,
    temporary: Vec<Vertex>,
}
        
impl Builder {
    fn new(points: &[Point]) -> Result<Self, Error> {
        if points.len() < 2 {
            return Err(Error::FewPoints);
        }
        let mut builder = Builder {
            events: Events::default(),
            beach: Beach::default(),
            diagram: Diagram::default(),
            scan_theta: Angle::from(0.0),
            temporary: Vec::new(),
        };
        for &point in points {
            let face = builder.diagram.add_face(point);
            builder.events.add_site(face, point);
        }
        Ok(builder)
    }
    
    pub fn build(mut self, relaxations: usize) -> Result<Diagram, Error> {
        self.build_iter();
        for _ in 1..relaxations {
            self.reset();
            self.build_iter();
        }
        self.finish();
        Ok(self.diagram)
    }   

    fn build_iter(&mut self) {
        while let Some(event) = self.events.pop() {
            //println!("{:#?}", event);
            let point = event.point;
            self.scan_theta = *point.theta();
            match event.kind {
                EventKind::Site(face) => self.handle_site_event(face, point),
                EventKind::Circle(event) => self.handle_circle_event(event),
            }
        }
    }

    fn reset(&mut self) {
        self.diagram.reset_faces();
        self.diagram.clear_vertices();
        self.diagram.clear_edges();
        for face in self.diagram.faces() {
            self.events.add_site(face, *self.diagram.face_point(face));
        }
        self.temporary.clear();
        self.events.clear();
        self.beach.clear();
        self.scan_theta = Angle::from(0.0);
    }

    fn arc_point(&self, arc: Arc) -> &Point {
        &self.diagram.face_point(self.beach.face(arc))
    }

    fn create_temporary(&mut self, arc0: Arc, arc1: Arc) {
        let index = self.temporary.len();
        self.temporary.push(Vertex::invalid());
        self.beach.set_start(arc0, ArcStart::Temporary(index));
        self.beach.set_start(arc1, ArcStart::Temporary(index));
    }

    fn handle_site_event(&mut self, face: Face, point: Point) {
        if let Some(mut arc) = self.beach.root() {
            if self.beach.len() == 1 {
                self.beach.insert_after(Some(arc), face);
                let arc0 = self.beach.first();
                let arc1 = self.beach.last();
                self.create_temporary(arc0, arc1);
                return;
            }
            let mut use_tree = true;
            loop {
                let prev_arc = self.beach.prev(arc);
                let next_arc = self.beach.next(arc);
                let phi_start = self.arcs_intersection(prev_arc, arc);
                let phi_end = self.arcs_intersection(arc, next_arc);
                match point.phi().is_in_range(phi_start, phi_end) {
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
                            let face = self.beach.face(arc);
                            let a = if prev_arc == self.beach.last() {
                                None
                            } else {
                                Some(prev_arc)
                            };
                            self.beach.insert_after(a, face)
                        };
                        let new_arc = self.beach.insert_after(Some(twin), face);
                        self.create_temporary(twin, new_arc);
                        self.attach_circle(prev_arc, twin, new_arc, point.theta().value());
                        self.attach_circle(new_arc, arc, next_arc, point.theta().value());
                        if self.detach_circle(prev_arc) {
                            let prev_prev = self.beach.prev(prev_arc);
                            self.attach_circle(prev_prev, prev_arc, twin, ::std::f64::MIN);
                        }
                        break;
                    }
                }
            }
        } else {
            self.beach.insert_after(None, face);
        }
    }
    
    fn merge_arcs(&mut self, arc0: Arc, arc1: Arc, arc2: Arc, vertex: Option<Vertex>) {
        let (face0, face1, face2) = (self.beach.face(arc0), self.beach.face(arc1), self.beach.face(arc2));
        if face0 != face1 && face1 != face2 && face2 != face0 {
            let theta = self.scan_theta.value();
            if self.attach_circle(arc0, arc1, arc2, theta) {
                if let Some(vertex) = vertex {
                    self.beach.set_start(arc1, ArcStart::Vertex(vertex));
                }
            }
        }
    }
    
    fn edge_from_arc(&mut self, arc: Arc, end: Vertex) {
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
    
    fn handle_circle_event(&mut self, event: Circle) {
        if self.events.is_invalid(event) {
            return;
        }
        let (arc0, arc1, arc2) = self.events.arcs(event);
        assert_eq!(self.beach.circle(arc1), Some(event));
        self.beach.set_circle(arc1, None);
        self.detach_circle(arc0);
        self.detach_circle(arc2);
        let point = self.events.center(event);
        let vertex = self.diagram.add_vertex(point, &[self.beach.face(arc0), self.beach.face(arc1), self.beach.face(arc2)]);
        self.edge_from_arc(arc0, vertex);
        self.edge_from_arc(arc1, vertex);
        self.beach.remove(arc1);
        if self.beach.prev(arc0) == arc2 {
            self.edge_from_arc(arc2, vertex);
            self.beach.remove(arc0);
            self.beach.remove(arc2);
        } else {
            let prev = self.beach.prev(arc0);
            let next = self.beach.next(arc2);
            self.merge_arcs(prev, arc0, arc2, Some(vertex));
            self.merge_arcs(arc0, arc2, next, None);
        }
    }
    
    fn arcs_intersection(&self, arc0: Arc, arc1: Arc) -> f64 {
        let point0 = self.arc_point(arc0);  
        let theta0 = point0.theta();
        let phi0 = point0.phi();
        if theta0.value() >= self.scan_theta.value() {
            return phi0.value();
        }
        let point1 = self.arc_point(arc1);  
        let theta1 = point1.theta();
        let phi1 = point1.phi();    
        if theta1.value() >= self.scan_theta.value() {
            return phi1.value();
        } 
        let u1 = (self.scan_theta.cos() - theta1.cos()) * theta0.sin();
        let u2 = (self.scan_theta.cos() - theta0.cos()) * theta1.sin();
        let a1 = u1 * phi0.cos();
        let a2 = u2 * phi1.cos();
        let a = a1 - a2;
        let b1 = u1 * phi0.sin();
        let b2 = u2 * phi1.sin();
        let b = b1 - b2;
        let c = (theta0.cos() - theta1.cos()) * self.scan_theta.sin();
        let length = (a * a + b * b).sqrt();
        if a.abs() > length || c.abs() > length {
            unreachable!()
        } else {
            let gamma = a.atan2(b);
            let phi_int_plus_gamma1 = (c / length).asin();
            Angle::wrap(phi_int_plus_gamma1 - gamma)
        }
    }
    
    fn attach_circle(&mut self, arc0: Arc, arc1: Arc, arc2: Arc, min_theta: f64) -> bool {
        let p1 = self.arc_point(arc1).position();
        let p01 = self.arc_point(arc0).position() - p1;
        let p21 = self.arc_point(arc2).position() - p1;
        let cross = p01.cross(p21);
        let center = Point::from_cartesian(cross.x, cross.y, cross.z);
        let radius = center.distance(&self.arc_point(arc0)); 
        let theta = center.theta().value() + radius;
        if theta >= min_theta {
            let point = Point::from_angles(Angle::from(theta), center.phi().clone());
            let circle = self.events.add_circle((arc0, arc1, arc2), center, radius, point);
            self.beach.set_circle(arc1, Some(circle));
            true
        } else {
            false
        }
    }
    
    fn detach_circle(&mut self, arc: Arc) -> bool {
        if let Some(event) = self.beach.circle(arc) {
            self.events.set_invalid(event, true);
            self.beach.set_circle(arc, None);
            true
        } else {
            false
        }
    }
    
    fn finish(&mut self) {
        for edge in self.diagram.edges() {
            let mut common = Vec::new(); 
            let (vertex0, vertex1) = self.diagram.edge_vertices(edge);  
            for &face0 in self.diagram.vertex_faces(vertex0) {
                for &face1 in self.diagram.vertex_faces(vertex1) {
                    if face0 == face1 {
                        common.push(face0);
                    }
                }
            }
            assert_eq!(common.len(), 2);
            self.diagram.set_edge_faces(edge, common[0], common[1]);
        }
    }
}

#[derive(PartialEq)]
pub enum Error {
    FewPoints,
}

pub fn build(points: &[Point], relaxations: usize) -> Result<Diagram, Error> {
    Builder::new(points)?.build(relaxations)
}

