use std::f64::consts::{PI, FRAC_1_PI};
use red_black_tree::{RedBlackTree, Node};
use diagram::{Diagram, Vertex, Cell};
use point::{Point, Position};

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
    scan: f64,
    end: f64,
}

pub type Arc = Node<ArcData>;

pub struct Beach {
    arcs: RedBlackTree<ArcData>,  
}

impl Beach {
    pub fn insert(&mut self, cell: Cell, point: Point, diagram: &Diagram) -> Arc {
        let root = self.arcs.root();
        if self.arcs.len() > 1 {
            let mut arc = root;
            let mut use_tree = true;
            loop {
                let prev = self.prev(arc);
                let start = self.intersect_with_next(prev, &point, diagram);
                let end = self.intersect_with_next(arc, &point, diagram);
                if start > end {
                    self.detach(arc);
                    let twin = {
                        let cell = self.cell(arc);
                        let a = if prev == self.arcs.last(root) {
                            Node::invalid()
                        } else {
                            prev
                        };
                        self.insert_after(a, cell)
                    };
                    return self.insert_after(twin, cell);
                }
                if start.min(2.0 * PI - start) < end.min(2.0 * PI - end) {
                    if use_tree {
                        arc = self.arcs.left(arc);
                        if arc.is_invalid() {
                            // the tree has failed us, do the linear search from now on.
                            arc = self.arcs.last(root);
                            use_tree = false;
                        }
                    } else {
                        arc = self.prev(arc);
                    }
                } else {
                    if use_tree {
                        arc = self.arcs.right(arc);
                        if arc.is_invalid() {
                            // the tree has failed us, do the linear search from now on.
                            arc = self.arcs.first(root);
                            use_tree = false;
                        }
                    } else {
                        arc = self.next(arc);
                    }
                }
            }
        } else {
            self.insert_after(root, cell)
        }
    }
    
    fn intersect_with_next(&mut self, arc: Arc, point: &Point, diagram: &Diagram) -> f64 {
        let arc_point = diagram.cell_point(self.cell(arc));
        let next_point = diagram.cell_point(self.cell(self.next(arc)));
        let data = &mut self.arcs[arc];
        if data.scan != point.theta.value {
            data.scan = point.theta.value;
            data.end = Beach::intersect(arc_point, next_point, point);
        }
        data.end
    }

    fn intersect(point0: &Point, point1: &Point, point2: &Point) -> f64 {
        let u1 = (point2.theta.cos - point1.theta.cos) * point0.theta.sin;
        let u2 = (point2.theta.cos - point0.theta.cos) * point1.theta.sin;
        let a = u1 * point0.phi.cos - u2 * point1.phi.cos;
        let b = u1 * point0.phi.sin - u2 * point1.phi.sin;
        let c = (point0.theta.cos - point1.theta.cos) * point2.theta.sin;
        let length = (a * a + b * b).sqrt();
        let gamma = a.atan2(b);
        let phi_plus_gamma = (c / length).asin();
        let mut angle = phi_plus_gamma - gamma - point2.phi.value;
        angle *= 0.5 * FRAC_1_PI;
        angle -= angle.floor();
        angle * 2.0 * PI
    }

    pub fn insert_after(&mut self, arc: Arc, cell: Cell) -> Arc {
        self.arcs.insert_after(arc, ArcData {
            cell: cell,
            start: ArcStart::None,
            center: Position::new(0.0, 0.0, 0.0),
            is_valid: false,
            scan: -1.0,
            end: 0.0,
        })
    }
    
    pub fn neighbors(&self, arc: Arc) -> (Arc, Arc) {
        self.arcs.neighbors(arc)
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

    pub fn next(&self, arc: Arc) -> Arc {
        self.arcs.next(arc)
    }

    pub fn prev(&self, arc: Arc) -> Arc {
        self.arcs.prev(arc)
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
