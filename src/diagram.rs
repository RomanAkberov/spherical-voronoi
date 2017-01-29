use ideal::{Id, IdVec};
use ideal::vec::IdsIter;

pub struct VertexData {
    point: Point,
    edges: Vec<Edge>,
    faces: Vec<Face>,
}

pub struct EdgeData {
    vertices: (Vertex, Vertex),
    faces: (Face, Face),
}

pub struct FaceData {
    point: Point,
    vertices: Vec<Vertex>,
    edges: Vec<Edge>,
}

#[derive(Default)]
pub struct Diagram {
    vertices: IdVec<VertexData>,
    edges: IdVec<EdgeData>,
    faces: IdVec<FaceData>,
}

impl Diagram {
    pub fn add_vertex(&mut self, point: Point, faces: &[Face]) -> Vertex {
        let vertex = self.vertices.push(VertexData {
            point: point,
            edges: Vec::new(),
            faces: Vec::from(faces),
        });
        for &face in faces {
            self.faces[face].vertices.push(vertex);
        }
        vertex
    }

    pub fn vertices(&self) -> IdsIter<VertexData> {
        self.vertices.ids()
    }

    pub fn vertex_point(&self, vertex: Vertex) -> &Point {
        &self.vertices[vertex].point
    }

    pub fn vertex_edges(&self, vertex: Vertex) -> &[Edge] {
        &self.vertices[vertex].edges
    }

    pub fn vertex_faces(&self, vertex: Vertex) -> &[Face] {
        &self.vertices[vertex].faces
    }

    pub fn vertex_neighbors(&self, vertex: Vertex) -> Vec<Vertex> {
        self.vertex_edges(vertex)
            .iter()
            .map(|&edge| self.other_edge_vertex(edge, vertex))
            .collect()
    }

    pub fn add_edge(&mut self, vertex0: Vertex, vertex1: Vertex) -> Edge {
        self.edges.push(EdgeData {
            vertices: (vertex0, vertex1),
            faces: (Face::invalid(), Face::invalid())
        })
    }

    pub fn edges(&self) -> IdsIter<EdgeData> {
        self.edges.ids()
    }

    pub fn edge_vertices(&self, edge: Edge) -> (Vertex, Vertex) {
        self.edges[edge].vertices
    }

    pub fn set_edge_faces(&mut self, edge: Edge, face0: Face, face1: Face) {
        self.edges[edge].faces = (face0, face1)
    }

    pub fn other_edge_vertex(&self, edge: Edge, vertex: Vertex) -> Vertex {
        let (vertex0, vertex1) = self.edge_vertices(edge);
        if vertex == vertex0 {
            vertex1
        } else if vertex == vertex1 {
            vertex0
        } else {
            Vertex::invalid()
        }
    }

    pub fn add_face(&mut self, point: Point) -> Face {
        self.faces.push(FaceData {
            point: point,
            vertices: Vec::new(),
            edges: Vec::new(),
        })
    }

    pub fn faces(&self) -> IdsIter<FaceData> {
        self.faces.ids()
    }

    pub fn face_point(&self, face: Face) -> &Point {
        &self.faces[face].point
    }

    pub fn face_vertices(&self, face: Face) -> &[Vertex] {
        &self.faces[face].vertices
    }

    pub fn face_edges(&self, face: Face) -> &[Edge] {
        &self.faces[face].edges
    }
}

pub type Vertex = Id<VertexData>;
pub type Edge = Id<EdgeData>;
pub type Face = Id<FaceData>;