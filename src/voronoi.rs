use std::f64::consts::PI;
use std::usize::{self};
use nalgebra::{Vector3, Cross};
use point::Point;
use events::{Events, EventKind, Circle};
use beach::{Beach, Arc};
use diagram::{Diagram, Vertex, Face};

struct SphericalVoronoi {
    events: Events,
    beach: Beach,
    diagram: Diagram,
    scan_theta: f64,
}

fn is_phi_between(phi: f64, phi_start: f64, phi_end: f64) -> bool {
    if phi_start <= phi_end {
        phi_start <= phi && phi <= phi_end
    } else {
        phi < phi_end || phi > phi_start
    }
}
        
impl SphericalVoronoi {
    fn new(directions: &[Vector3<f64>]) -> Result<Self, Error> {
        if directions.len() < 2 {
            return Err(Error::FewPoints);
        }
        let mut diagram = Diagram::new();
        let mut events = Events::new();
        for direction in directions {
            let face = diagram.new_face(Point::from_cartesian(*direction));
            events.add_site(face, diagram.face_point(face));
        }
        Ok(SphericalVoronoi {
            events: events,
            beach: Beach::new(),
            diagram: diagram,
            scan_theta: 0.0,
        })
    }
    
    fn build(mut self) -> Result<Diagram, Error> {
        while let Some(event) = self.events.pop() {
            self.scan_theta = event.point.theta;
            //println!("Scan theta: {}", self.scan_theta);
            match event.kind {
                EventKind::Site(face) => self.site_event(face, event.point),
                EventKind::Circle(event) => self.circle_event(event),
            }
        }
        self.diagram.cleanup_vertices();
        self.diagram.finish_faces();
        Ok(self.diagram)
    }
  
    fn arc_point(&self, arc: Arc) -> Point {
        self.diagram.face_point(self.beach.face(arc))  
    }
    
    fn site_event(&mut self, face: Face, point: Point) {
        //println!("Site: {:?}, len: {}", point, self.beach.len());
        if self.beach.len() == 0 {
            self.beach.add(0, face);
        } else if self.beach.len() == 1 {
            self.beach.add(1, face);
            let arc0 = self.beach.get(0);
            let arc1 = self.beach.get(1);
            //println!("{:?}", self.arc_point(arc0));
            let point = self.phi_to_point(point.phi, self.arc_point(arc0));
            let vertex = self.diagram.new_vertex(point, vec![face, self.beach.face(arc0)]);
            self.beach.set_start(arc0, Some(vertex));
            self.beach.set_start(arc1, Some(vertex));
        } else {
            let mut arc_index = 0;
            while arc_index < self.beach.len() {
                let arc = self.beach.get(arc_index);
                let prev_index = self.beach.prev_index(arc_index);
                let prev_arc = self.beach.get(prev_index);
                let next_index = self.beach.next_index(arc_index);
                let next_arc = self.beach.get(next_index);
                let phi_start = self.arc_phi(prev_arc, arc);
                let phi_end = self.arc_phi(arc, next_arc);
                if is_phi_between(point.phi, phi_start, phi_end) {
                    self.try_remove_circle(arc);
                    let arc2 = {
                        let face = self.beach.face(arc);
                        self.beach.add(arc_index, face)
                    };
                    arc_index += 1;
                    let new_arc = self.beach.add(arc_index, face);
                    let point = self.phi_to_point(point.phi, self.arc_point(arc));
                    let vertex = self.diagram.new_vertex(point, vec![face, self.beach.face(arc)]);
                    self.beach.set_start(arc2, Some(vertex));
                    self.beach.set_start(new_arc, Some(vertex));
                    let prev_index = self.beach.index(prev_arc).unwrap();
                    let arc_index2 = self.beach.next_index(prev_index);
                    self.try_add_circle(prev_arc, arc2, new_arc, point.theta);
                    self.try_add_circle(new_arc, arc, next_arc, point.theta);
                    if self.try_remove_circle(prev_arc) {
                        let prev_prev_index = self.beach.prev_index(prev_index);
                        let arc0 = self.beach.get(prev_prev_index);
                        let arc1 = self.beach.get(prev_index);
                        let arc2 = self.beach.get(arc_index2);
                        self.try_add_circle(arc0, arc1, arc2, -2.0 * PI);
                    }
                    break;
                }
                arc_index += 1;
            }
        }
    }
    
    fn merge_arcs(&mut self, arc0: Arc, arc1: Arc, arc2: Arc, vertex: Option<Vertex>) {
        let (face0, face1, face2) = (self.beach.face(arc0), self.beach.face(arc1), self.beach.face(arc2));
        if face0 != face1 && face1 != face2 && face2 != face0 {
            let theta = self.scan_theta;
            if self.try_add_circle(arc0, arc1, arc2, theta) {
                if vertex.is_some() {
                    self.beach.set_start(arc1, vertex);
                }
            }
        }
    }
    
    fn edge_from_arc(&mut self, arc: Arc, vertex: Vertex) {
        if let Some(start) = self.beach.start(arc) {
            self.diagram.new_edge(start, vertex);
        }    
    }
    
    fn circle_event(&mut self, event: Circle) {
        if self.events.is_invalid(event) {
            return;
        }
        //println!("Circle");
        let (arc0, arc1, arc2) = self.events.arcs(event);
        assert_eq!(self.beach.circle(arc1), Some(event));
        self.beach.set_circle(arc1, None);
        self.try_remove_circle(arc0);
        self.try_remove_circle(arc2);
        let faces = vec![self.beach.face(arc0), self.beach.face(arc1), self.beach.face(arc2)];
        let vertex = self.diagram.new_vertex(self.events.center(event), faces);
        self.edge_from_arc(arc0, vertex);
        self.edge_from_arc(arc1, vertex);
        let index = self.beach.index(arc1).unwrap();
        self.beach.remove(index);
        let index0 = self.beach.index(arc0).unwrap();
        let index2 = self.beach.index(arc2).unwrap();
        if self.beach.prev_index(index0) == index2 {
            self.edge_from_arc(arc2, vertex);
            self.beach.remove(index0);
            let index2 = self.beach.index(arc2).unwrap();
            self.beach.remove(index2);
        } else {
            let prev_arc = self.beach.get(self.beach.prev_index(index0));
            let next_arc = self.beach.get(self.beach.next_index(index2));
            self.merge_arcs(prev_arc, arc0, arc2, Some(vertex));
            self.merge_arcs(arc0, arc2, next_arc, None);
        }
    }
    
    fn arc_phi(&self, arc0: Arc, arc1: Arc) -> f64 {
        if arc0 == arc1 { 
            self.arc_point(arc0).phi - PI 
        } else {
            self.arc_intersection(arc0, arc1).expect("Arcs don't intersect").phi
        }
    }
    
    fn arc_intersection(&self, arc1: Arc, arc2: Arc) -> Option<Point> {
        let point1 = self.arc_point(arc1);
        let point2 = self.arc_point(arc2);
        let theta1 = point1.theta;
        let phi1 = point1.phi;
        let theta2 = point2.theta;
        let phi2 = point2.phi;
        match (theta1 >= self.scan_theta, theta2 >= self.scan_theta) {
            (true, true) => None,
            (true, false) => Some(self.phi_to_point(phi1, point1)),
            (false, true) => Some(self.phi_to_point(phi2, point2)),
            (false, false) => {
                let (sin_scan, cos_scan) = self.scan_theta.sin_cos();
                let (sin_theta1, cos_theta1) = theta1.sin_cos();
                let (sin_theta2, cos_theta2) = theta2.sin_cos();
                let (sin_phi1, cos_phi1) = phi1.sin_cos();
                let (sin_phi2, cos_phi2) = phi2.sin_cos();
                let a1 = (cos_scan - cos_theta2) * sin_theta1 * cos_phi1;
                let a2 = (cos_scan - cos_theta1) * sin_theta2 * cos_phi2;
                let a = a1 - a2;
                let b1 = (cos_scan - cos_theta2) * sin_theta1 * sin_phi1;
                let b2 = (cos_scan - cos_theta1) * sin_theta2 * sin_phi2;
                let b = b1 - b2;
                let c = (cos_theta1 - cos_theta2) * sin_scan;
                let length = (a * a + b * b).sqrt();
                if a.abs() > length || c.abs() > length {
                    None
                } else {
                    let gamma = a.atan2(b);
                    let phi_int_plus_gamma1 = (c / length).asin();
                    let phi = phi_int_plus_gamma1 - gamma;
                    Some(self.phi_to_point(phi, point1))
                }
            }
        }
    }
    
    fn phi_to_point(&self, phi: f64, point: Point) -> Point {
        let phi = if phi > PI {
            phi - 2.0 * PI
        } else if phi < -PI {
            phi + 2.0 * PI
        } else {
            phi
        };
        if point.theta >= self.scan_theta {
            Point::from_spherical(self.scan_theta, phi) // could be any point on the line segment
        } else {
            let a = self.scan_theta.sin() - point.theta.sin() * (phi - point.phi).cos();
            let b = point.theta.cos() - self.scan_theta.cos();
            let theta = b.atan2(a);
            Point::from_spherical(theta, phi)
        }
    }
    
    fn try_add_circle(&mut self, arc0: Arc, arc1: Arc, arc2: Arc, theta: f64) -> bool {
        let p01 = self.arc_point(arc0).position - self.arc_point(arc1).position;
        let p21 = self.arc_point(arc2).position - self.arc_point(arc1).position;
        let center = Point::from_cartesian(p01.cross(&p21));
        let radius = center.distance(&self.arc_point(arc0)); 
        let point = Point::from_spherical(center.theta + radius, center.phi);
        if point.theta >= theta {
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
}

#[derive(PartialEq)]
pub enum Error {
    FewPoints    
}

pub fn generate(directions: &[Vector3<f64>]) -> Result<Diagram, Error> {
    let voronoi = try!(SphericalVoronoi::new(directions));
    voronoi.build()
}

pub fn generate_relaxed(directions: &[Vector3<f64>], relaxations: usize) -> Result<Diagram, Error> {
    let mut diagram = try!(generate(directions));
    for _ in 0..relaxations {
        let new_directions: Vec<Vector3<f64>> = diagram.faces().map(|face| {
            let mut center = Vector3::new(0.0, 0.0, 0.0);
            let mut count = 0.0f64;
            for vertex in diagram.face_vertices(face) {
                center += diagram.vertex_point(*vertex).position;
                count += 1.0;
            }
            Vector3::new(center.x / count, center.y / count, center.z / count)
        }).collect();
        diagram = try!(generate(&new_directions));
    }
    Ok(diagram)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::Vector3;
    
    #[test]
    fn zero_points() {
        assert!(if let Err(Error::FewPoints) = generate(&vec![]) {
            true
        } else {
            false
        });
    }
    
    #[test]
    fn one_point() {
        assert!(if let Err(Error::FewPoints) = generate(&vec![Vector3::new(1.0, 0.0, 0.0)]) {
            true
        } else {
            false
        });
    }
}