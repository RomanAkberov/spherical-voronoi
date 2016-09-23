use id::{Id, Pool, IterMut, Ids};
use point::Point;
use nalgebra::{Vector3, Cross, Dot};

pub struct VertexData {
    point: Point,
    edges: Vec<Edge>,
    faces: Vec<Face>,
}
pub type Vertex = Id<VertexData>;
 
pub struct EdgeData {
    vertices: (Vertex, Vertex),
    faces: (Face, Face),
}
pub type Edge = Id<EdgeData>;

pub struct FaceData {
    point: Point,
    edges: Vec<Edge>,
    vertices: Vec<Vertex>,
}
pub type Face = Id<FaceData>;

pub struct Diagram {
    faces: Pool<FaceData>,
    edges: Pool<EdgeData>,
    vertices: Pool<VertexData>,
}

fn is_bad_vertex(vertex_data: &VertexData) -> bool {
    vertex_data.faces.len() <= 2
}

fn is_bad_edge(edge_data: &EdgeData, vertices: &Pool<VertexData>) -> bool {
    let (vertex0, vertex1) = edge_data.vertices;
    is_bad_vertex(&vertices[vertex0]) || is_bad_vertex(&vertices[vertex1])
}

impl Diagram {
    pub fn new() -> Self {
        Diagram {
            faces: Pool::new(),
            edges: Pool::new(),
            vertices: Pool::new(),
        }
    }
    
    pub fn new_edge(&mut self, vertex0: Vertex, vertex1: Vertex) -> Edge {
        let edge = self.edges.add(EdgeData {
            vertices: (vertex0, vertex1),
            faces: (Face::none(), Face::none()),
        });
        self.vertices[vertex0].edges.push(edge);
        self.vertices[vertex1].edges.push(edge);
        //println!("Edge: {:?} - {:?}", self.vertex_point(vertex0), self.vertex_point(vertex1));
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
    
    pub fn vertices(&self) -> Ids<VertexData> {
        self.vertices.ids()
    }
    
    pub fn vertex_point(&self, vertex: Vertex) -> Point {
        self.vertices[vertex].point    
    }
    
    pub fn vertex_position(&self, vertex: Vertex) -> Vector3<f64> {
        self.vertex_point(vertex).position
    } 
    
    pub fn vertex_faces(&self, vertex: Vertex) -> &[Face] {
        &self.vertices[vertex].faces
    }
    
    pub fn vertex_edges(&self, vertex: Vertex) -> &[Edge] {
        &self.vertices[vertex].edges
    }
    
    pub fn edges(&self) -> Ids<EdgeData> {
        self.edges.ids()    
    }
    
    pub fn edge_vertices(&self, edge: Edge) -> (Vertex, Vertex) {
        self.edges[edge].vertices
    }
       
    pub fn faces(&self) -> Ids<FaceData> {
        self.faces.ids()
    }
    
    pub fn face_point(&self, face: Face) -> Point {
        self.faces[face].point
    }
    
    pub fn face_position(&self, face: Face) -> Vector3<f64> {
        self.face_point(face).position
    }
    
    pub fn face_vertices(&self, face: Face) -> &[Vertex] {
        &self.faces[face].vertices
    }
    
    pub fn face_edges(&self, face: Face) -> &[Edge] {
        &self.faces[face].edges
    }
    
    pub fn other_edge_vertex(&self, edge: Edge, vertex: Vertex) -> Option<Vertex> {
        let (vertex0, vertex1) = self.edge_vertices(edge);
        if vertex  == vertex0 {
            Some(vertex1)
        } else if vertex == vertex1 {
            Some(vertex0)
        } else {
            None
        }
    }
    
    pub fn cleanup_vertices(&mut self) {
        let mut new_edges = Vec::new();
        for vertex in self.vertices() {
            if self.vertex_faces(vertex).len() == 2 {
                let edges = &self.vertices[vertex].edges;
                assert_eq!(edges.len(), 2);
                let vertex0 = self.other_edge_vertex(edges[0], vertex).unwrap();
                let vertex1 = self.other_edge_vertex(edges[1], vertex).unwrap();
                new_edges.push((vertex0, vertex1));
            }
        }
        for (vertex0, vertex1) in new_edges {
            self.new_edge(vertex0, vertex1);
        }
        let mut bad_vertices = Vec::new();
        for vertex in self.vertices() {
            if is_bad_vertex(&self.vertices[vertex]) {
                bad_vertices.push(vertex);
            }
        }
        for vertex in bad_vertices {
            self.vertices.remove(vertex);
        }
        let mut bad_edges = Vec::new();
        for edge in self.edges() {
            if is_bad_edge(&self.edges[edge], &self.vertices) {
                bad_edges.push(edge);           
            } 
        }
        for edge in bad_edges {
            self.edges.remove(edge);
        }
    }
    
    pub fn finish_faces(&mut self) {
        let mut edge_faces = Vec::new(); 
        for edge in self.edges() {
            let mut common = Vec::new(); 
            let (vertex0, vertex1) = self.edge_vertices(edge);
            for face0 in self.vertex_faces(vertex0) {
                for face1 in self.vertex_faces(vertex1) {
                    if face0 == face1 {
                        common.push(*face0);
                    }
                }
            }
            assert_eq!(common.len(), 2);
            edge_faces.push((edge, common[0], common[1]));
        }
        for (edge, face0, face1) in edge_faces {
            self.faces[face0].edges.push(edge);
            self.faces[face1].edges.push(edge);
            self.edges[edge].faces = (face0, face1);
        }
        for vertex in self.vertices.ids() {
            for face in &self.vertices[vertex].faces {
                self.faces[*face].vertices.push(vertex);
            }
        }
        for (_, data) in self.faces.iter_mut() {
            let n = data.point.position;
            let vertices = &self.vertices;
            data.vertices.sort_by(|v1, v2| (vertices[*v1].point.position - n).cross(&(vertices[*v2].point.position - n)).dot(&n).partial_cmp(&0.0).unwrap());
        }
    }
}

