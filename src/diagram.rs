use nalgebra::{Vector3, Cross, Dot};
use id::{Id, IdVec, IdVecIter};
use point::Point;
use fnv::FnvHashMap;

fn remap_id<T>(id: Id<T>, remap: &FnvHashMap<Id<T>, Id<T>>) -> Id<T> {
    let mut result = id;
    while let Some(&mapped) = remap.get(&result) {
        result = mapped;
    }
    result
}

pub struct VertexData {
    point: Point,
    edges: Vec<Edge>,
    faces: Vec<Face>,
}
pub type Vertex = Id<VertexData>;

#[derive(Default)]
pub struct Vertices {
    vertices: IdVec<VertexData>,
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
    
    pub fn retain<F: FnMut(&VertexData) -> bool>(&mut self, f: F) {
        self.vertices.retain(f);
    }
    
    pub fn ids(&self) -> IdVecIter<VertexData> {
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
    edges: IdVec<EdgeData>,
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
    
    pub fn set_faces(&mut self, edge: Edge, face1: Face, face2: Face) {
        self.edges[edge].faces = (face1, face2);
    }
    
    pub fn retain<F: FnMut(&EdgeData) -> bool>(&mut self, f: F) {
        self.edges.retain(f);
    }
    
    pub fn ids(&self) -> IdVecIter<EdgeData> {
        self.edges.ids()
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
    faces: IdVec<FaceData>,
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
    
    pub fn ids(&self) -> IdVecIter<FaceData> {
        self.faces.ids()
    }
    
    pub fn len(&self) -> usize {
        self.faces.len()
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
        let mut bad_vertices = Vec::new();
        let mut bad_edges = Vec::new();
        for vertex in self.vertices.ids() {
            if self.vertices.faces(vertex).len() == 2 {
                let (edge0, edge1) = {
                    let edges = &self.vertices.edges(vertex);
                    assert_eq!(edges.len(), 2);
                    (edges[0], edges[1])
                };
                let vertex0 = self.edges.other_vertex(edge0, vertex).unwrap();
                let vertex1 = self.edges.other_vertex(edge1, vertex).unwrap();
                self.new_edge(vertex0, vertex1);
                bad_vertices.push(vertex);
                bad_edges.push(edge0);
                bad_edges.push(edge1);
            }
        }
        bad_edges.sort();
        bad_edges.dedup();
        let mut vertices_remap = FnvHashMap::default();
        for vertex in bad_vertices {
            let new_vertex = remap_id(vertex, &vertices_remap);
            let prev_vertex = self.vertices.vertices.remove(new_vertex);
            vertices_remap.insert(prev_vertex, vertex);
        }
        let mut edges_remap = FnvHashMap::default();
        for edge in bad_edges {
            let new_edge = remap_id(edge, &edges_remap);
            let prev_edge = self.edges.edges.remove(new_edge);
            edges_remap.insert(prev_edge, edge);
        }
        for edge in self.edges.ids() {
            let (vertex0, vertex1) = self.edges.vertices(edge);
            self.edges.edges[edge].vertices = (remap_id(vertex0, &vertices_remap), remap_id(vertex1, &vertices_remap));
        }
        for face in self.faces.ids() {
            for vertex in self.faces.faces[face].vertices.iter_mut() {
                *vertex = remap_id(*vertex, &vertices_remap);
            }
            for edge in self.faces.faces[face].edges.iter_mut() {
                *edge = remap_id(*edge, &edges_remap);
            }
        }
    }
    
    pub fn finish_faces(&mut self) {
        for edge in self.edges.ids() {
            let mut common = Vec::new(); 
            let (vertex0, vertex1) = self.edges.vertices(edge);
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
            self.edges.set_faces(edge, common[0], common[1]);
        }
        for face in self.faces.ids() {
            let n = self.faces.position(face);
            let mut edge = self.faces.edges(face)[0];
            let (v0, v1) = self.edges.vertices(edge);
            let (prev, v) = if are_clockwise(n, self.vertices.position(v0), self.vertices.position(v1)) {
                (v0, v1) 
            } else {
                (v1, v0)
            };
            self.faces.add_vertex(face, prev);
            let mut vertex = v;
            for _ in 0..self.faces.edges(face).len() - 1 {
                self.faces.add_vertex(face, vertex);
                for &e in self.faces.edges(face) {
                    if e != edge {
                        if let Some(v) = self.edges.other_vertex(e, vertex) {
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
