use std::collections::BinaryHeap;
use std::rc::Rc;
use std::f64::consts::PI;
use std::cell::Cell;
use std::usize::{self};
use nalgebra::{Vector3, Cross};
use point::Point;
use event::{Event, CircleEvent};
use beach_arc::{BeachArc};
use diagram::{Diagram, Vertex, Edge, Face};
use ref_eq::RefEq;

struct SphericalVoronoi {
    events: BinaryHeap<Event>,
    beach: Vec<Rc<BeachArc>>,
    vertices: Vec<Vertex>,
    edges: Vec<Edge>,
    faces: Vec<Face>,
    scan_theta: f64,
}

fn is_phi_between(phi: f64, phi_start: f64, phi_end: f64) -> bool {
    if phi_start <= phi_end {
        phi_start <= phi && phi <= phi_end
    } else {
        phi < phi_end || phi > phi_start
    }
}

fn is_bad_vertex(vertex: &Vertex) -> bool {
    vertex.face_ids.len() <= 2
}

fn is_bad_edge(edge: &Edge, vertices: &[Vertex]) -> bool {
    let (id0, id1) = edge.vertex_ids;
    is_bad_vertex(&vertices[id0]) || is_bad_vertex(&vertices[id1])
}

fn remap_id(id: usize, missing_ids: &[usize]) -> usize {
    let position = match missing_ids.binary_search(&id) {
        Ok(position) => position,
        Err(position) => position,
    };
    id - position
}
        
impl SphericalVoronoi {
    fn new(directions: &[Vector3<f64>]) -> Self {
        let mut faces = Vec::new();
        for direction in directions {
            faces.push(Face::new(Point::from_cartesian(*direction)));
        }
        let mut events = BinaryHeap::with_capacity(faces.len());
        for (face_id, face) in faces.iter().enumerate() {
            events.push(Event::Site(face_id, face.point));
        }
        SphericalVoronoi {
            events: events,
            beach: Vec::new(),
            vertices: Vec::new(),
            edges: Vec::new(),
            faces: faces,
            scan_theta: 0.0,
        }
    }
    
    fn build(mut self) -> Diagram {
        while let Some(event) = self.events.pop() {
            self.scan_theta = event.point().theta;
            match event {
                Event::Site(face_id, point) => self.handle_site_event(face_id, point),
                Event::Circle(event) => self.handle_circle_event(&event),
            }
        }
        self.cleanup_vertices();
        self.finish_faces();
        Diagram {
            vertices: self.vertices,
            edges: self.edges,
            faces: self.faces,
        }
    }
    
    fn new_arc(&mut self, face_id: usize) {
        self.beach.push(Rc::new(BeachArc::new(face_id)));
    }
    
    fn arc_point(&self, arc: &BeachArc) -> &Point {
        &self.faces[arc.face_id].point    
    }
    
    fn new_circle_event(&self, arc0: &Rc<BeachArc>, arc1: &Rc<BeachArc>, arc2: &Rc<BeachArc>) -> Rc<CircleEvent> {
        let p01 = self.arc_point(arc0).position - self.arc_point(arc1).position;
        let p21 = self.arc_point(arc2).position - self.arc_point(arc1).position;
        let center = Point::from_cartesian(p01.cross(&p21));
        let radius = center.distance(self.arc_point(arc0)); 
        let point = Point::from_spherical(center.theta + radius, center.phi);
        Rc::new(CircleEvent {
            arcs: [arc0.clone(), arc1.clone(), arc2.clone()],
            center: center,
            radius: radius,
            point: point,
            is_invalid: Cell::new(false),
        })
    }
    
    fn arc_index(&self, arc: &Rc<BeachArc>) -> Option<usize> {
        self.beach.iter().position(|x| x.ref_eq(arc))
    }
    
    fn new_edge(&mut self, vertex_id0: usize, vertex_id1: usize) -> usize {
        let edge = Edge {
            vertex_ids: (vertex_id0, vertex_id1),
            face_ids: (usize::MAX, usize::MAX),
        };
        let edge_id = self.edges.len();
        self.edges.push(edge);
        self.vertices[vertex_id0].edge_ids.push(edge_id);
        self.vertices[vertex_id1].edge_ids.push(edge_id);
        edge_id
    }
    
    fn new_vertex(&mut self, point: Point, face_ids: Vec<usize>) -> usize {
        let vertex_id = self.vertices.len();
        self.vertices.push(Vertex {
            point: point,
            edge_ids: Vec::new(),
            face_ids: face_ids
        });
        vertex_id  
    }
    
    fn handle_site_event(&mut self, face_id: usize, point: Point) {
        //println!("Site: {:?}", point);
        if self.beach.len() == 0 {
            self.new_arc(face_id);
        } else if self.beach.len() == 1 {
            self.new_arc(face_id);
            let arc0 = self.beach[0].clone();
            let arc1 = self.beach[1].clone();
            let point = self.phi_to_point(point.phi, *self.arc_point(&arc0));
            let vertex_id = self.new_vertex(point, vec![face_id, arc0.face_id]);
            arc0.start_id.set(vertex_id);
            arc1.start_id.set(vertex_id);
        } else {
            let mut arc_index = 0;
            while arc_index < self.beach.len() {
                let arc = self.beach[arc_index].clone();
                let prev_index = self.prev_arc_index(arc_index);
                let prev_arc = self.beach[prev_index].clone();
                let next_index = self.next_arc_index(arc_index);
                let next_arc = self.beach[next_index].clone();
                let phi_start = if prev_index == arc_index { 
                    self.arc_point(&arc).phi - PI 
                } else {
                    self.arc_intersection(&prev_arc, &arc).expect("Arcs don't intersect (1)").phi
                };
                let phi_end = if next_index == arc_index {
                    self.arc_point(&arc).phi + PI 
                } else {
                    self.arc_intersection(&arc, &next_arc).expect("Arcs don't intersect (2)").phi
                };
                if is_phi_between(point.phi, phi_start, phi_end) {
                    self.try_remove_circle_event(&arc);
                    let arc2 = Rc::new(BeachArc::new(arc.face_id));
                    self.beach.insert(arc_index, arc2.clone());
                    arc_index += 1;
                    let new_arc = Rc::new(BeachArc::new(face_id));
                    self.beach.insert(arc_index, new_arc.clone());
                    let point = self.phi_to_point(point.phi, *self.arc_point(&arc));
                    let vertex_id = self.new_vertex(point, vec![face_id, arc.face_id]);
                    arc2.start_id.set(vertex_id);
                    new_arc.start_id.set(vertex_id);
                    let prev_index = self.arc_index(&prev_arc).unwrap();
                    let arc_index2 = self.next_arc_index(prev_index);
                    let event1 = self.new_circle_event(&prev_arc, &arc2, &new_arc);
                    if event1.point.theta >= point.theta {
                        *arc2.event.borrow_mut() = Some(event1.clone());
                        self.events.push(Event::Circle(event1));
                    }
                    let event2 = self.new_circle_event(&new_arc, &arc, &next_arc);
                    if event2.point.theta >= point.theta {
                        *arc.event.borrow_mut() = Some(event2.clone());
                        self.events.push(Event::Circle(event2));
                    }
                    if self.try_remove_circle_event(&prev_arc) {
                        let prev_prev_index = self.prev_arc_index(prev_index);
                        let event = self.new_circle_event(&self.beach[prev_prev_index], &self.beach[prev_index], &self.beach[arc_index2]);
                        *prev_arc.event.borrow_mut() = Some(event.clone());
                        self.events.push(Event::Circle(event));
                    }
                    break;
                }
                arc_index += 1;
            }
        }
    }
    
    fn handle_circle_event(&mut self, event: &Rc<CircleEvent>) {
        if event.is_invalid.get() {
            return;
        }
        //println!("Circle");
        let arc0 = event.arcs[0].clone();
        let arc1 = event.arcs[1].clone();
        let arc2 = event.arcs[2].clone();
        assert!(arc1.event.borrow().as_ref().unwrap().ref_eq(event));
        *arc1.event.borrow_mut() = None;
        self.try_remove_circle_event(&arc0);
        self.try_remove_circle_event(&arc2);
        let vertex_id = self.new_vertex(event.center, vec![arc0.face_id, arc1.face_id, arc2.face_id]);
        let start_id0 = arc0.start_id.get();
        if start_id0 != usize::MAX {
            self.new_edge(start_id0, vertex_id);
        }
        let start_id1 = arc1.start_id.get();
        if start_id1 != usize::MAX {
            self.new_edge(start_id1, vertex_id);
        }
        let index = self.arc_index(&arc1).unwrap();
        self.beach.remove(index);
        let index0 = self.arc_index(&arc0).unwrap();
        let index2 = self.arc_index(&arc2).unwrap();
        if self.prev_arc_index(index0) == index2 {
            let start_id2 = arc2.start_id.get();
            if start_id2 != usize::MAX {
                self.new_edge(start_id2, vertex_id);
            }
            self.beach.remove(index0);
            let index2 = self.arc_index(&arc2).unwrap();
            self.beach.remove(index2);
        } else {
            let it0 = self.prev_arc_index(index0);
            let it1 = self.next_arc_index(index2);
            let arc_it0 = self.beach[it0].clone();
            let arc_it1 = self.beach[it1].clone();
            if arc_it0.face_id != arc0.face_id &&
                arc0.face_id != arc2.face_id &&
                arc_it0.face_id != arc2.face_id {
                let event = self.new_circle_event(&arc_it0, &arc0, &arc2);
                if event.point.theta >= self.scan_theta {
                    *arc0.event.borrow_mut() = Some(event.clone());
                    arc0.start_id.set(vertex_id);
                    self.events.push(Event::Circle(event));
                }
            }
            if arc0.face_id != arc2.face_id &&
                arc2.face_id != arc_it1.face_id &&
                arc_it1.face_id != arc0.face_id {
                let event = self.new_circle_event(&arc0, &arc2, &arc_it1);
                if event.point.theta >= self.scan_theta {
                    *arc2.event.borrow_mut() = Some(event.clone());
                    self.events.push(Event::Circle(event));
                }
            }
        }
    }
    
    fn arc_intersection(&self, arc1: &BeachArc, arc2: &BeachArc) -> Option<Point> {
        let point1 = self.arc_point(arc1);
        let point2 = self.arc_point(arc2);
        let theta1 = point1.theta;
        let phi1 = point1.phi;
        let theta2 = point2.theta;
        let phi2 = point2.phi;
        match (theta1 >= self.scan_theta, theta2 >= self.scan_theta) {
            (true, true) => None,
            (true, false) => Some(self.phi_to_point(phi1, *point1)),
            (false, true) => Some(self.phi_to_point(phi2, *point2)),
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
                    Some(self.phi_to_point(phi, *point1))
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
    
    fn prev_arc_index(&self, index: usize) -> usize {
        (index + self.beach.len() - 1) % self.beach.len()
    }
    
    fn next_arc_index(&self, index: usize) -> usize {
        (index + 1) % self.beach.len()
    }
    
    fn try_remove_circle_event(&self, arc: &Rc<BeachArc>) -> bool {
        let is_some = arc.event.borrow().is_some();
        if is_some {
            {
                arc.event.borrow().as_ref().unwrap().is_invalid.set(true);
            }
            *arc.event.borrow_mut() = None;
            true
        } else {
            false
        }
    }
    
    fn cleanup_vertices(&mut self) {
        for id in 0..self.vertices.len() {
            if self.vertices[id].face_ids.len() == 2 {
                let (id0, id1) = {
                    let edge_ids = &self.vertices[id].edge_ids;
                    assert_eq!(edge_ids.len(), 2);
                    (self.edges[edge_ids[0]].other_vertex_id(id), self.edges[edge_ids[1]].other_vertex_id(id))
                };
                self.new_edge(id0.unwrap(), id1.unwrap());
            }
        }
        let removed_vertex_ids: Vec<usize> = self.vertices.iter().
            enumerate().
            filter_map(|(id, vertex)| if is_bad_vertex(vertex) { Some(id) } else { None }).
            collect();
        let removed_edge_ids: Vec<usize> = self.edges.iter().
            enumerate().
            filter_map(|(id, edge)| if is_bad_edge(edge, &self.vertices) { Some(id) } else { None }).
            collect();
        {
            let vertices = &self.vertices;
            self.edges.retain(|edge| !is_bad_edge(edge, vertices));
        }
        self.vertices.retain(|vertex| !is_bad_vertex(vertex));
        for vertex in self.vertices.iter_mut() {
            for edge_id in vertex.edge_ids.iter_mut() {
                *edge_id = remap_id(*edge_id, &removed_edge_ids);
            }
        }
        for edge in self.edges.iter_mut() {
            let (id0, id1) = edge.vertex_ids;
            edge.vertex_ids = (remap_id(id0, &removed_vertex_ids), remap_id(id1, &removed_vertex_ids));
        }
        for face in self.faces.iter_mut() {
            for vertex_id in face.vertex_ids.iter_mut() {
                *vertex_id = remap_id(*vertex_id, &removed_vertex_ids);
            }
            for edge_id in face.edge_ids.iter_mut() {
                *edge_id = remap_id(*edge_id, &removed_edge_ids);
            }
        }
    }
    
    fn finish_faces(&mut self) {
        for (edge_id, edge) in self.edges.iter_mut().enumerate() {
            let mut common = Vec::new(); 
            let (id0, id1) = edge.vertex_ids;
            for face0 in self.vertices[id0].face_ids.iter() {
                for face1 in self.vertices[id1].face_ids.iter() {
                    if face0 == face1 {
                        common.push(*face0);
                    }
                }
            }
            assert_eq!(common.len(), 2);
            for face in common.iter() {
                self.faces[*face].edge_ids.push(edge_id);
            }
            edge.face_ids = (common[0], common[1]);
        }
        for (vertex_id, vertex) in self.vertices.iter().enumerate() {
            for face_id in vertex.face_ids.iter() {
                self.faces[*face_id].vertex_ids.push(vertex_id);
            }
        }
    }
}

pub fn generate(directions: &[Vector3<f64>]) -> Diagram {
    SphericalVoronoi::new(directions).build()
}

pub fn generate_relaxed(directions: &[Vector3<f64>], relaxations: usize) -> Diagram {
    let mut diagram = generate(directions);
    for i in 0..relaxations {
        let new_directions: Vec<Vector3<f64>> = diagram.faces.iter().map(|face| {
            let mut center = Vector3::new(0.0, 0.0, 0.0);
            for vertex_id in face.vertex_ids.iter() {
                center += diagram.vertices[*vertex_id].point.position;
            }
            let len = face.vertex_ids.len() as f64;
            Vector3::new(center.x / len, center.y / len, center.z / len)
        }).collect();
    }
    diagram
}