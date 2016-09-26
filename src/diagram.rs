use nalgebra::{Vector3, Cross, Dot};
use id::{Id, IdVec, Ids, IterMut, IdVecIter};
use point::Point;

fn remap_id<T>(id: Id<T>, missing_ids: &[Id<T>]) -> Id<T> {
    let position = match missing_ids.binary_search(&id) {
        Ok(position) => position,
        Err(position) => position,
    };
    Id::new(id.index() - position)
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

fn is_bad_vertex(vertex_data: &VertexData) -> bool {
    vertex_data.faces.len() <= 2
}

fn is_bad_edge(edge_data: &EdgeData, vertices: &IdVec<VertexData>) -> bool {
    let (vertex0, vertex1) = edge_data.vertices;
    is_bad_vertex(&vertices[vertex0]) || is_bad_vertex(&vertices[vertex1])
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
        for vertex in self.vertices.ids() {
            if self.vertices.faces(vertex).len() == 2 {
                let (vertex0, vertex1) = {
                    let edges = &self.vertices.edges(vertex);
                    //println!("{:?} {:?} {:?}", vertex, edges, &self.vertices[vertex].faces);
                    assert_eq!(edges.len(), 2);
                    (self.edges.other_vertex(edges[0], vertex), self.edges.other_vertex(edges[1], vertex))
                };
                self.new_edge(vertex0.unwrap(), vertex1.unwrap());
            }
        }
        let bad_vertices: Vec<_> = self.vertices.ids().
            filter(|vertex| is_bad_vertex(&self.vertices.vertices[*vertex])).
            collect();
        let bad_edges: Vec<_> = self.edges.ids().
            filter(|edge| is_bad_edge(&self.edges.edges[*edge], &self.vertices.vertices)).
            collect();
        {
            let vertices = &self.vertices.vertices;
            self.edges.retain(|edge| !is_bad_edge(edge, vertices));
        }
        self.vertices.retain(|vertex| !is_bad_vertex(vertex));
        for vertex in self.vertices.ids() {
            for edge in self.vertices.vertices[vertex].edges.iter_mut() {
                *edge = remap_id(*edge, &bad_edges);
            }
        }
        for edge in self.edges.ids() {
            let (vertex0, vertex1) = self.edges.vertices(edge);
            self.edges.edges[edge].vertices = (remap_id(vertex0, &bad_vertices), remap_id(vertex1, &bad_vertices));
        }
        for face in self.faces.ids() {
            for vertex in self.faces.faces[face].vertices.iter_mut() {
                *vertex = remap_id(*vertex, &bad_vertices);
            }
            for edge in self.faces.faces[face].edges.iter_mut() {
                *edge = remap_id(*edge, &bad_edges);
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
        for vertex in self.vertices.ids() {
            for face in self.vertices.faces(vertex).iter() {
                self.faces.faces[*face].vertices.push(vertex);
            }
        }
        for face in self.faces.ids() {
            let n = self.faces.position(face);
            let vertices = &self.vertices.vertices;
            self.faces.faces[face].vertices.sort_by(|v1, v2| (vertices[*v1].point.position - n).cross(&(vertices[*v2].point.position - n)).dot(&n).partial_cmp(&0.0).unwrap());
        }
    }
}
