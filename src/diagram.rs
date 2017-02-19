use ideal::{Id, IdVec};
use ideal::vec::IdsIter;
use ::Position;

pub struct VertexData {
    position: Position,
    edges: Vec<Edge>,
    cells: [Cell; 3],
}

pub struct EdgeData {
    vertices: (Vertex, Vertex),
    cells: (Cell, Cell),
}

pub struct CellData {
    vertices: Vec<Vertex>,
    edges: Vec<Edge>,
}

#[derive(Default)]
pub struct Diagram {
    vertices: IdVec<VertexData>,
    edges: IdVec<EdgeData>,
    cells: IdVec<CellData>,
}

impl Diagram {
    pub fn add_vertex(&mut self, position: Position, cells: [Cell; 3]) -> Vertex {
        let vertex = self.vertices.push(VertexData {
            position: position,
            edges: Vec::new(),
            cells: cells,
        });
        for &cell in &cells {
            self.cells[cell].vertices.push(vertex);
        }
        vertex
    }

    pub fn vertices(&self) -> IdsIter<VertexData> {
        self.vertices.ids()
    }

    pub fn vertex_position(&self, vertex: Vertex) -> Position {
        self.vertices[vertex].position
    }

    pub fn vertex_edges(&self, vertex: Vertex) -> &[Edge] {
        &self.vertices[vertex].edges
    }

    pub fn vertex_cells(&self, vertex: Vertex) -> &[Cell] {
        &self.vertices[vertex].cells
    }

    pub fn vertex_neighbors(&self, vertex: Vertex) -> Vec<Vertex> {
        self.vertex_edges(vertex)
            .iter()
            .map(|&edge| self.other_edge_vertex(edge, vertex))
            .collect()
    }

    pub fn add_edge(&mut self, vertex0: Vertex, vertex1: Vertex, cell0: Cell, cell1: Cell) -> Edge {
        let edge = self.edges.push(EdgeData {
            vertices: (vertex0, vertex1),
            cells: (cell0, cell1),
        });
        self.vertices[vertex0].edges.push(edge);
        self.vertices[vertex1].edges.push(edge);
        self.cells[cell0].edges.push(edge);
        self.cells[cell1].edges.push(edge);
        edge
    }

    pub fn edges(&self) -> IdsIter<EdgeData> {
        self.edges.ids()
    }

    pub fn edge_vertices(&self, edge: Edge) -> (Vertex, Vertex) {
        self.edges[edge].vertices
    }

    pub fn edge_cells(&self, edge: Edge) -> (Cell, Cell) {
        self.edges[edge].cells
    }

    pub fn set_edge_cells(&mut self, edge: Edge, cell0: Cell, cell1: Cell) {
        self.edges[edge].cells = (cell0, cell1);
        self.cells[cell0].edges.push(edge);
        self.cells[cell1].edges.push(edge);
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

    pub fn other_edge_cell(&self, edge: Edge, cell: Cell) -> Cell {
        let (cell0, cell1) = self.edge_cells(edge);
        if cell == cell0 {
            cell1
        } else if cell == cell1 {
            cell0
        } else {
            Cell::invalid()
        }
    }

    pub fn add_cell(&mut self) -> Cell {
        self.cells.push(CellData {
            vertices: Vec::new(),
            edges: Vec::new(),
        })
    }

    pub fn cells(&self) -> IdsIter<CellData> {
        self.cells.ids()
    }

    pub fn cell_vertices(&self, cell: Cell) -> &[Vertex] {
        &self.cells[cell].vertices
    }

    pub fn cell_edges(&self, cell: Cell) -> &[Edge] {
        &self.cells[cell].edges
    }

    pub fn cell_neighbors(&self, cell: Cell) -> Vec<Cell> {
        self.cell_edges(cell)
            .iter()
            .map(|&edge| self.other_edge_cell(edge, cell))
            .collect()
    }
}

pub type Vertex = Id<VertexData>;
pub type Edge = Id<EdgeData>;
pub type Cell = Id<CellData>;
