use std::cmp::Ordering;
use red_black_tree::{RedBlackTree, Node};
use diagram::{Diagram, Vertex, Cell};
use point::{Point, Position};
use angle::Angle;

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

    pub fn insert(&mut self, cell: Cell, point: Point, diagram: &Diagram) -> Arc {
        if let Some(root) = self.root() {
            if self.len() == 1 {
                return self.insert_after(Some(root), cell);
            }
            let mut arc = root;
            let mut use_tree = true;
            loop {
                let (prev, next) = self.neighbors(arc);
                let start = self.intersect(prev, arc, point.theta, diagram);
                let end = self.intersect(arc, next, point.theta, diagram);
                match point.phi.is_in_range(start, end) {
                    Ordering::Less => {
                        if use_tree {
                            if let Some(left) = self.left(arc) {
                                arc = left;
                            } else {
                                // the tree has failed us, do the linear search from now on.
                                arc = self.last();
                                use_tree = false;
                            }
                        } else {
                            arc = self.prev(arc);
                        }
                    },
                    Ordering::Greater => {
                        if use_tree {
                            if let Some(right) = self.right(arc) {
                                arc = right;
                            } else {
                                // the tree has failed us, do the linear search from now on.
                                arc = self.first();
                                use_tree = false;
                            }
                        } else {
                            arc = self.next(arc);
                        }
                    },
                    Ordering::Equal => {
                        self.detach(arc);
                        let twin = {
                            let cell = self.cell(arc);
                            let a = if prev == self.last() {
                                None
                            } else {
                                Some(prev)
                            };
                            self.insert_after(a, cell)
                        };
                        return self.insert_after(Some(twin), cell);
                    }
                }
            }
        } else {
            self.insert_after(None, cell)
        }
    }
        
    fn intersect(&self, arc0: Arc, arc1: Arc, theta: Angle, diagram: &Diagram) -> f64 {
        let point0 = diagram.cell_point(self.cell(arc0));
        let point1 = diagram.cell_point(self.cell(arc1));
        let u1 = (theta.cos - point1.theta.cos) * point0.theta.sin;
        let u2 = (theta.cos - point0.theta.cos) * point1.theta.sin;
        let a = u1 * point0.phi.cos - u2 * point1.phi.cos;
        let b = u1 * point0.phi.sin - u2 * point1.phi.sin;
        let c = (point0.theta.cos - point1.theta.cos) * theta.sin;
        let length = (a * a + b * b).sqrt();
        let gamma = a.atan2(b);
        let phi_plus_gamma = (c / length).asin();
        Angle::wrap(phi_plus_gamma - gamma)
    }

    pub fn insert_after(&mut self, arc: Option<Arc>, cell: Cell) -> Arc {
        self.arcs.insert_after(arc, ArcData {
            cell: cell,
            start: ArcStart::None,
            center: Position::new(0.0, 0.0, 0.0),
            is_valid: false,
        })
    }
    
    pub fn neighbors(&self, arc: Arc) -> (Arc, Arc) {
        (self.prev(arc), self.next(arc))
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
