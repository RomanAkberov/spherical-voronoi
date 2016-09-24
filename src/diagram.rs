use nalgebra::{Vector3, Cross, Dot};
use id::{Id, Pool, Ids, IterMut};
use point::Point;

pub struct VertexData {
    point: Point,
    edges: Vec<Edge>,
    faces: Vec<Face>,
}
pub type Vertex = Id<VertexData>;

#[derive(Default)]
pub struct Vertices {
    vertices: Pool<VertexData>,
} 

impl Vertices {
    fn add(&mut self, data: VertexData) -> Vertex {
        self.vertices.add(data)    
    }
    
    fn add_edge(&mut self, vertex: Vertex, edge: Edge) {
        self.vertices[vertex].edges.push(edge);        
    }   
        
    pub fn point(&self, vertex: Vertex) -> Point {
        self.vertices[vertex].point    
    }
    
    pub fn position(&self, vertex: Vertex) -> Vector3<f64> {
        self.point(vertex).position
    } 
    
    pub fn faces(&self, vertex: Vertex) -> &[Face] {
        &self.vertices[vertex].faces
    }
    
    pub fn edges(&self, vertex: Vertex) -> &[Edge] {
        &self.vertices[vertex].edges
    }
    
    pub fn remove(&mut self, vertex: Vertex) {
        self.vertices.remove(vertex);
    }
    
    pub fn ids(&self) -> Ids<VertexData> {
        self.vertices.ids()
    }
}

pub struct EdgeData {
    vertices: (Vertex, Vertex),
    faces: (Face, Face),
}
pub type Edge = Id<EdgeData>;

#[derive(Default)]
pub struct Edges {
    edges: Pool<EdgeData>,
}

impl Edges {
    fn add(&mut self, data: EdgeData) -> Edge {
        self.edges.add(data)
    }
    
    pub fn vertices(&self, edge: Edge) -> (Vertex, Vertex) {
        self.edges[edge].vertices
    }
    
    pub fn other_vertex(&self, edge: Edge, vertex: Vertex) -> Option<Vertex> {
        let (vertex0, vertex1) = self.vertices(edge);
        if vertex  == vertex0 {
            Some(vertex1)
        } else if vertex == vertex1 {
            Some(vertex0)
        } else {
            None
        }
    }
    
    pub fn remove(&mut self, edge: Edge) {
        self.edges.remove(edge);
    }
    
    pub fn iter_mut(&mut self) -> IterMut<EdgeData> {
        self.edges.iter_mut()
    }
}

pub struct FaceData {
    point: Point,
    edges: Vec<Edge>,
    vertices: Vec<Vertex>,
}
pub type Face = Id<FaceData>;

#[derive(Default)]
pub struct Faces {
    faces: Pool<FaceData>,
}

impl Faces {
    fn add(&mut self, data: FaceData) -> Face {
        self.faces.add(data)
    }
    
    pub fn point(&self, face: Face) -> Point {
        self.faces[face].point
    }
    
    pub fn position(&self, face: Face) -> Vector3<f64> {
        self.point(face).position
    }
    
    pub fn vertices(&self, face: Face) -> &[Vertex] {
        &self.faces[face].vertices
    }
    
    pub fn edges(&self, face: Face) -> &[Edge] {
        &self.faces[face].edges
    }
    
    pub fn add_edge(&mut self, face: Face, edge: Edge) {
        self.faces[face].edges.push(edge);
    }
    
    pub fn add_vertex(&mut self, face: Face, vertex: Vertex) {
        self.faces[face].vertices.push(vertex);
    }
    
    pub fn remove(&mut self, face: Face) {
        self.faces.remove(face);
    }
    
    pub fn iter_mut(&mut self) -> IterMut<FaceData> {
        self.faces.iter_mut()
    }
    
    pub fn ids(&self) -> Ids<FaceData> {
        self.faces.ids()
    }
}

#[derive(Default)]
pub struct Diagram {
    pub faces: Faces,
    pub edges: Edges,
    pub vertices: Vertices,
}

fn are_clockwise(n: Vector3<f64>, v1: Vector3<f64>, v2: Vector3<f64>) -> bool {
    (v1 - n).cross(&(v2 - n)).dot(&n) < 0.0
}

impl Diagram {
    pub fn new_edge(&mut self, vertex0: Vertex, vertex1: Vertex) -> Edge {
        let edge = self.edges.add(EdgeData {
            vertices: (vertex0, vertex1),
            faces: (Face::none(), Face::none()),
        });
        self.vertices.add_edge(vertex0, edge);
        self.vertices.add_edge(vertex1, edge);
        edge
    }
    
    pub fn new_vertex(&mut self, point: Point, faces: Vec<Face>) -> Vertex {
        self.vertices.add(VertexData {
            point: point,
            edges: Vec::new(),
            faces: faces
        })
    }
    
    pub fn new_face(&mut self, point: Point) -> Face {
        self.faces.add(FaceData {
            point: point,
            edges: Vec::new(),
            vertices: Vec::new(),
        })
    }
    
    pub fn center(&self, face: Face) -> Vector3<f64> {
        let mut center = Vector3::new(0.0, 0.0, 0.0);
        let mut count = 0.0f64;
        for vertex in self.faces.vertices(face) {
            center += self.vertices.point(*vertex).position;
            count += 1.0;
        }
        Vector3::new(center.x / count, center.y / count, center.z / count)
    }       
    
    pub fn cleanup_vertices(&mut self) {
        let mut new_edges = Vec::new();
        let mut bad_edges = Vec::new();
        let mut bad_vertices = Vec::new();
        for vertex in self.vertices.ids() {
            if self.vertices.faces(vertex).len() == 2 {
                let edges = self.vertices.edges(vertex);
                assert_eq!(edges.len(), 2);
                let vertex0 = self.edges.other_vertex(edges[0], vertex).unwrap();
                let vertex1 = self.edges.other_vertex(edges[1], vertex).unwrap();
                new_edges.push((vertex0, vertex1));
                bad_edges.push(edges[0]);
                bad_edges.push(edges[1]);
                bad_vertices.push(vertex);
            }
        }
        for (vertex0, vertex1) in new_edges {
            self.new_edge(vertex0, vertex1);
        }
        for edge in bad_edges {
            self.edges.remove(edge);
        }
        for vertex in bad_vertices {
            self.vertices.remove(vertex);
        }
    }
    
    pub fn finish_faces(&mut self) {
        for (&edge, data) in self.edges.iter_mut() {
            let mut common = Vec::new(); 
            let (vertex0, vertex1) = data.vertices;
            for face0 in self.vertices.faces(vertex0) {
                for face1 in self.vertices.faces(vertex1) {
                    if face0 == face1 {
                        common.push(*face0);
                    }
                }
            }
            assert_eq!(common.len(), 2);
            self.faces.add_edge(common[0], edge);
            self.faces.add_edge(common[1], edge);
            data.faces = (common[0], common[1]);
        }
        for (_, data) in self.faces.iter_mut() {
            let n = data.point.position;
            let mut e = data.edges[0];
            let (v1, v2) = self.edges.vertices(e);
            let p1 = self.vertices.position(v1);
            let p2 = self.vertices.position(v2);
            let (v_prev, v) = if are_clockwise(n, p1, p2) {
                (v1, v2)
            } else {
                (v2, v1)  
            };
            data.vertices.push(v_prev);
            let mut v = v;
            for _ in 1..data.edges.len() {
                data.vertices.push(v);
                for &ee in data.edges.iter() {
                    if ee != e {
                        if let Some(vv) = self.edges.other_vertex(ee, v) {
                            assert!(are_clockwise(n, self.vertices.position(v), self.vertices.position(vv)));
                            e = ee;
                            v = vv;
                            break;
                        }
                    }
                }
            }
        }
    }
}
