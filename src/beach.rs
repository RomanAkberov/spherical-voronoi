use red_black_tree::{RedBlackTree, Node};
use diagram::{Vertex, Cell};
use point::Position;

#[derive(Copy, Clone)]
pub enum ArcStart {
    None,
    Vertex(Vertex),
    Temporary(usize),
}

pub struct ArcData {
    cell: Cell,
    start: ArcStart,
    center: Position,
    is_valid: bool,
}

pub type Arc = Node<ArcData>;

pub struct Beach {
    arcs: RedBlackTree<ArcData>,  
}

impl Beach {
    pub fn root(&self) -> Option<Arc> {
        self.arcs.root()
    }

    pub fn insert_after(&mut self, arc: Option<Arc>, cell: Cell) -> Arc {
        self.arcs.insert_after(arc, ArcData {
            cell: cell,
            start: ArcStart::None,
            center: Position::new(0.0, 0.0, 0.0),
            is_valid: false,
        })
    }
    
    pub fn remove(&mut self, arc: Arc) {
        self.arcs.remove(arc);
    }

    pub fn is_valid(&self, arc: Arc) -> bool {
        self.arcs[arc].is_valid
    }
    
    pub fn cell(&self, arc: Arc) -> Cell {
        self.arcs[arc].cell
    }
    
    pub fn start(&self, arc: Arc) -> ArcStart {
        self.arcs[arc].start    
    }
    
    pub fn set_start(&mut self, arc: Arc, start: ArcStart) {
        self.arcs[arc].start = start;
    }
    
    pub fn center(&self, arc: Arc) -> Position {
        self.arcs[arc].center
    }

    pub fn attach(&mut self, arc: Arc, center: Position) {
        let data = &mut self.arcs[arc];
        data.is_valid = true;
        data.center = center;
    }

    pub fn detach(&mut self, arc: Arc) {
        self.arcs[arc].is_valid = false;
    }

    pub fn len(&self) -> usize {
        self.arcs.len()
    }

    pub fn next(&self, arc: Arc) -> Arc {
        self.arcs.next(arc).unwrap_or_else(|| self.arcs.first(self.arcs.root().unwrap()))
    }

    pub fn prev(&self, arc: Arc) -> Arc {
        self.arcs.prev(arc).unwrap_or_else(|| self.arcs.last(self.arcs.root().unwrap()))
    }

    pub fn left(&self, arc: Arc) -> Option<Arc> {
        self.arcs.left(arc)
    }

    pub fn right(&self, arc: Arc) -> Option<Arc> {
        self.arcs.right(arc)
    }

    pub fn first(&self) -> Arc {
        self.arcs.first(self.root().unwrap())
    }

    pub fn last(&self) -> Arc {
        self.arcs.last(self.root().unwrap())
    }

    pub fn clear(&mut self) {
        self.arcs.clear();
    }
}

impl Default for Beach {
    fn default() -> Self {
        Beach {
            arcs: Default::default(),
        }
    }
}
