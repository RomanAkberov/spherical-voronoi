use std::f64::consts::PI;
use std::cmp::Ordering;
use cgmath::prelude::*;
use cgmath::Point3;
use point::{Point, SinCosCache};
use events::{Events, EventKind, Circle};
use beach::{Beach, Arc};
use diagram::{Diagram, Vertex, Edge, Face};

pub trait Position: From<Point> {
    fn point(&self) -> &Point;
    
    fn position(&self) -> Point3<f64> {
        self.point().position
    }
}

impl Position for Point {
    fn point(&self) -> &Point {
        self
    }
}

struct SphericalVoronoi<V, E, F>
    where V: Position, E: Default, F: Position {
    events: Events,
    beach: Beach,
    diagram: Diagram<V, E, F>,
    scan_theta: SinCosCache,
}

fn in_range(phi: f64, phi_start: f64, phi_end: f64) -> Ordering {
    if phi_start <= phi_end {
        if phi < phi_start {
            Ordering::Less
        } else if phi > phi_end {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    } else {
        if phi < phi_end || phi > phi_start {
            Ordering::Equal
        } else if phi >= phi_end {
            Ordering::Greater
        } else {
            Ordering::Less
        }
    }
}
        
impl<V, E, F> SphericalVoronoi<V, E, F> 
    where V: Position, E: Default, F: Position {
    fn new(points: &[Point]) -> Result<Self, Error> {
        if points.len() < 2 {
            return Err(Error::FewPoints);
        }
        let mut diagram = Diagram::default();
        let mut events = Events::default();
        for &point in points {
            let face = diagram.add_face(point.into());
            events.add_site(face, point);
        }
        Ok(SphericalVoronoi {
            events: events,
            beach: Beach::default(),
            diagram: diagram,
            scan_theta: 0.0.into(),
        })
    }
    
    fn build(mut self) -> Result<Diagram<V, E, F>, Error> {
        while let Some(event) = self.events.pop() {
            self.scan_theta = event.point.theta;
            //println!("Scan theta: {}", self.scan_theta.value);
            match event.kind {
                EventKind::Site(face) => self.site_event(face, event.point),
                EventKind::Circle(event) => self.circle_event(event),
            }
        }
        self.cleanup_vertices();
        self.finish_faces();
        Ok(self.diagram)
    }
  
    fn arc_point(&self, arc: Arc) -> &Point {
        self.diagram.face_data(self.beach.face(arc)).point()
    }
    
    fn create_vertex(&mut self, point: Point, faces: &[Face]) -> Vertex {
        //println!("Vertex: {:?}", point);
        let vertex = self.diagram.add_vertex(point.into());
        for &face in faces {
            self.diagram.add_vertex_face(vertex, face);
        }
        vertex
    }
    
    fn create_edge(&mut self, vertex0: Vertex, vertex1: Vertex) -> Edge {
        //println!("Edge [{:?}, {:?}]", self.vertex_point(vertex0), self.vertex_point(vertex1));
        self.diagram.add_edge(E::default(), vertex0, vertex1)
    }
    
            /*fn dump_beach(&self) {

        print!("[");
        if self.beach.root().is_some() {
            let mut arc = self.beach.first();
            for i in 0..self.beach.len() {
                if i > 0 {
                    print!(", ");
                }
                print!("{:?} {:?}", arc, self.arc_point(arc).phi());
                if i < self.beach.len() - 1 {
                    arc = self.beach.next(arc);
                }
            }
        }
        //println!("]");
        
    }*/

    fn site_event(&mut self, face: Face, point: Point) {
        //println!("Site: {:?} len: {}", point, self.beach.len());
        //self.dump_beach();
        if let Some(mut arc) = self.beach.root() {
            if self.beach.len() == 1 {
                self.beach.insert_after(Some(arc), face);
                let arc0 = self.beach.first();
                let arc1 = self.beach.last();
                let point = self.phi_to_point(point.phi, self.arc_point(arc0));
                let faces = [face, self.beach.face(arc0)];
                let vertex = self.create_vertex(point, &faces);
                self.beach.set_start(arc0, Some(vertex));
                self.beach.set_start(arc1, Some(vertex));
                return;
            }
            let mut use_tree = true;
            loop {
                let prev_arc = self.beach.prev(arc);
                let next_arc = self.beach.next(arc);
                let phi_start = self.arcs_intersection(prev_arc, arc);
                let phi_end = self.arcs_intersection(arc, next_arc);
                //println!("{:?}: {} < {} < {}", arc, phi_start, point.phi(), phi_end);
                match in_range(point.phi(), phi_start, phi_end) {
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
                        self.try_remove_circle(arc);
                        let arc2 = {
                            let face = self.beach.face(arc);
                            let a = if prev_arc == self.beach.last() {
                                None
                            } else {
                                Some(prev_arc)
                            };
                            self.beach.insert_after(a, face)
                        };
                        let new_arc = self.beach.insert_after(Some(arc2), face);
                        let point = self.phi_to_point(point.phi, self.arc_point(arc));
                        let faces = [face, self.beach.face(arc)];
                        let vertex = self.create_vertex(point, &faces);
                        self.beach.set_start(arc2, Some(vertex));
                        self.beach.set_start(new_arc, Some(vertex));
                        self.try_add_circle(prev_arc, arc2, new_arc, point.theta());
                        self.try_add_circle(new_arc, arc, next_arc, point.theta());
                        if self.try_remove_circle(prev_arc) {
                            let prev_prev = self.beach.prev(prev_arc);
                            self.try_add_circle(prev_prev, prev_arc, arc2, -2.0 * PI);
                        }
                        break;
                    }
                }
            }
        } else {
            self.beach.insert_after(None, face);
        }
        //self.dump_beach();
    }
    
    fn merge_arcs(&mut self, arc0: Arc, arc1: Arc, arc2: Arc, vertex: Option<Vertex>) {
        let (face0, face1, face2) = (self.beach.face(arc0), self.beach.face(arc1), self.beach.face(arc2));
        if face0 != face1 && face1 != face2 && face2 != face0 {
            let theta = self.scan_theta.value;
            if self.try_add_circle(arc0, arc1, arc2, theta) {
                if vertex.is_some() {
                    self.beach.set_start(arc1, vertex);
                }
            }
        }
    }
    
    fn edge_from_arc(&mut self, arc: Arc, vertex: Vertex) {
        if let Some(start) = self.beach.start(arc) {
            self.create_edge(start, vertex);
        }    
    }
    
    fn circle_event(&mut self, event: Circle) {
        if self.events.is_invalid(event) {
            return;
        }
        //println!("Circle");
        //self.dump_beach();
        let (arc0, arc1, arc2) = self.events.arcs(event);
        //println!("{:?}, {:?}, {:?}", arc0, arc1, arc2);
        assert_eq!(self.beach.circle(arc1), Some(event));
        self.beach.set_circle(arc1, None);
        self.try_remove_circle(arc0);
        self.try_remove_circle(arc2);
        let faces = [self.beach.face(arc0), self.beach.face(arc1), self.beach.face(arc2)];
        let point = self.events.center(event);
        let vertex = self.create_vertex(point, &faces);
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
        //self.dump_beach();
    }
    
    fn arcs_intersection(&mut self, arc0: Arc, arc1: Arc) -> f64 {
        let point0 = self.arc_point(arc0);
        let point1 = self.arc_point(arc1);       
        let theta0 = point0.theta;
        let phi0 = point0.phi;
        let theta1 = point1.theta;
        let phi1 = point1.phi;
        if theta0.value >= self.scan_theta.value {
            point0.phi.value
        } else if theta1.value >= self.scan_theta.value {
            point1.phi.value
        } else {
            let u1 = (self.scan_theta.cos - theta1.cos) * theta0.sin;
            let u2 = (self.scan_theta.cos - theta0.cos) * theta1.sin;
            let a1 = u1 * phi0.cos;
            let a2 = u2 * phi1.cos;
            let a = a1 - a2;
            let b1 = u1 * phi0.sin;
            let b2 = u2 * phi1.sin;
            let b = b1 - b2;
            let c = (theta0.cos - theta1.cos) * self.scan_theta.sin;
            let length = (a * a + b * b).sqrt();
            if a.abs() > length || c.abs() > length {
                unreachable!()
            } else {
                let gamma = a.atan2(b);
                let phi_int_plus_gamma1 = (c / length).asin();
                wrap(phi_int_plus_gamma1 - gamma)
            }
        }
    }
    
    fn phi_to_point(&self, phi: SinCosCache, point: &Point) -> Point {
        let mut phi = phi;
        phi.value = wrap(phi.value);
        if point.theta() >= self.scan_theta.value {
            Point::from_cache(self.scan_theta, phi) // could be any point on the line segment
        } else {
            let a = self.scan_theta.sin - point.theta.sin * (phi.value - point.phi()).cos();
            let b = point.theta.cos - self.scan_theta.cos;
            let theta = SinCosCache::from(b.atan2(a));
            Point::from_cache(theta, phi)
        }
    }
    
    fn try_add_circle(&mut self, arc0: Arc, arc1: Arc, arc2: Arc, theta: f64) -> bool {
        let p01 = self.arc_point(arc0).position - self.arc_point(arc1).position;
        let p21 = self.arc_point(arc2).position - self.arc_point(arc1).position;
        let cross = p01.cross(p21);
        let center = Point::from_cartesian(cross.x, cross.y, cross.z);
        let radius = center.distance(&self.arc_point(arc0)); 
        let point = Point::from_spherical(center.theta() + radius, center.phi());
        if point.theta() >= theta {
            let circle = self.events.add_circle((arc0, arc1, arc2), center, radius, point);
            self.beach.set_circle(arc1, Some(circle));
            true
        } else {
            false
        }
    }
    
    fn try_remove_circle(&mut self, arc: Arc) -> bool {
        if let Some(event) = self.beach.circle(arc) {
            self.events.set_invalid(event, true);
            self.beach.set_circle(arc, None);
            true
        } else {
            false
        }
    }
    
    fn cleanup_vertices(&mut self) {
        let mut bad_vertices = Vec::new();
        for vertex in self.diagram.vertices() {
            if self.diagram.vertex_faces(vertex).len() == 2 {
                let (edge0, edge1) = {
                    let edges = self.diagram.vertex_edges(vertex);
                    assert_eq!(edges.len(), 2);
                    (edges[0], edges[1])
                };
                let vertex0 = self.diagram.other_edge_vertex(edge0, vertex).unwrap();
                let vertex1 = self.diagram.other_edge_vertex(edge1, vertex).unwrap();
                self.create_edge(vertex0, vertex1);
                bad_vertices.push(vertex);
            }
        }
        self.diagram.remove_vertices(&bad_vertices);
    }
    
    fn vertex_point(&self, vertex: Vertex) -> &Point {
        self.diagram.vertex_data(vertex).point()
    }

    fn vertex_position(&self, vertex: Vertex) -> Point3<f64> {
        self.diagram.vertex_data(vertex).position()
    }
    
    fn face_position(&self, face: Face) -> Point3<f64> {
        self.diagram.face_data(face).position()
    }
    
    fn finish_faces(&mut self) {
        for edge in self.diagram.edges() {
            let mut common = Vec::new(); 
            let (vertex0, vertex1) = self.diagram.edge_vertices(edge);
            for face0 in self.diagram.vertex_faces(vertex0) {
                for face1 in self.diagram.vertex_faces(vertex1) {
                    if face0 == face1 {
                        common.push(*face0);
                    }
                }
            }
            assert_eq!(common.len(), 2);
            self.diagram.add_face_edge(common[0], edge);
            self.diagram.add_face_edge(common[1], edge);
            self.diagram.set_edge_faces(edge, common[0], common[1]);
        }
        for face in self.diagram.faces() {
            let n = self.face_position(face);
            let mut edge = self.diagram.face_edges(face)[0];
            let (v0, v1) = self.diagram.edge_vertices(edge);
            let (prev, v) = if are_clockwise(n, self.vertex_position(v0), self.vertex_position(v1)) {
                (v0, v1) 
            } else {
                (v1, v0)
            };
            self.diagram.add_face_vertex(face, prev);
            let mut vertex = v;
            for _ in 0..self.diagram.face_edges(face).len() - 1 {
                self.diagram.add_face_vertex(face, vertex);
                for &e in self.diagram.face_edges(face) {
                    if e != edge {
                        if let Some(v) = self.diagram.other_edge_vertex(e, vertex) {
                            vertex = v;
                            edge = e;
                            break;
                        }
                    }
                }    
            }
        }
    }
}

#[derive(PartialEq)]
pub enum Error {
    FewPoints,
    WrongEdgesNum,
    WrongCommonVerticesNum    
}

pub fn generate<V, E, F>(points: &[Point]) -> Result<Diagram<V, E, F>, Error>
    where V: Position, E: Default, F: Position {
    let voronoi = try!(SphericalVoronoi::new(points));
    voronoi.build()
}

pub fn generate_relaxed<V, E, F>(points: &[Point], relaxations: usize) -> Result<Diagram<V, E, F>, Error>
    where V: Position, E: Default, F: Position {
    let mut diagram = try!(generate(points));
    for _ in 0..relaxations {
        let new_points: Vec<_> = diagram.faces().
            map(|face| {
                let face_points: Vec<_> = diagram.
                    face_vertices(face).
                    iter().
                    map(|&vertex| {
                        let data: &V = diagram.vertex_data(vertex);
                        data.position()
                    }).
                    collect();
                let p = Point3::centroid(&face_points);
                Point::from_cartesian(p.x, p.y, p.z)
            }).
            collect();
        diagram = try!(generate(&new_points));
    }
    Ok(diagram)
}

fn are_clockwise(n: Point3<f64>, v1: Point3<f64>, v2: Point3<f64>) -> bool {
    (v1 - n).cross(v2 - n).dot(n.to_vec()) < 0.0
}

fn wrap(phi: f64) -> f64 {
    if phi > PI {
        phi - 2.0 * PI
    } else if phi < -PI {
        phi + 2.0 * PI
    } else {
        phi
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use point::Point;

    #[test]
    fn zero_points() {
        assert!(if let Err(Error::FewPoints) = generate::<Point, (), Point>(&vec![]) {
            true
        } else {
            false
        });
    }
    
    #[test]
    fn one_point() {
        assert!(if let Err(Error::FewPoints) = generate::<Point, (), Point>(&vec![Point::from_cartesian(1.0, 0.0, 0.0)]) {
            true
        } else {
            false
        });
    }
}
