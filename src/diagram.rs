use point::Point;

pub struct Vertex {
    pub point: Point,
    pub edge_ids: Vec<usize>,
    pub face_ids: Vec<usize>,
}

pub struct Edge {
    pub vertex_ids: (usize, usize),
    pub face_ids: (usize, usize),
}

impl Edge {
    pub fn other_vertex_id(&self, vertex_id: usize) -> Option<usize> {
        let (id0, id1) = self.vertex_ids;
        if vertex_id  == id0 {
            Some(id1)
        } else if vertex_id == id1 {
            Some(id0)
        } else {
            None
        }
    }
}

pub struct Face {
    pub point: Point,
    pub edge_ids: Vec<usize>,
    pub vertex_ids: Vec<usize>,
}

impl Face {
    pub fn new(point: Point) -> Self {
        Face {
            point: point,
            edge_ids: Vec::new(),
            vertex_ids: Vec::new(),
        }
    }
}

pub struct Diagram {
    pub faces: Vec<Face>,
    pub edges: Vec<Edge>,
    pub vertices: Vec<Vertex>,
}

