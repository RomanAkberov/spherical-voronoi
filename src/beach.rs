use events::Circle;
use diagram::{Kind, Vertex, Face};
use red_black_tree::{RedBlackTree, Node};

pub struct ArcData<K: Kind> {
    face: Face<K>,
    start: Option<Vertex<K>>,
    circle: Option<Circle<K>>,
}

pub type Arc<K> = Node<ArcData<K>>;

pub struct Beach<K: Kind> {
    arcs: RedBlackTree<ArcData<K>>,  
}

impl<K: Kind> Beach<K> {
    pub fn root(&self) -> Option<Arc<K>> {
        self.arcs.root()
    }

    pub fn insert_after(&mut self, arc: Option<Arc<K>>, face: Face<K>) -> Arc<K> {
        self.arcs.insert_after(arc, ArcData {
            face: face,
            start: None,
            circle: None,
        })
    }
    
    pub fn remove(&mut self, arc: Arc<K>) {
        self.arcs.remove(arc);
    }

    pub fn circle(&self, arc: Arc<K>) -> Option<Circle<K>> {
        self.arcs[arc].circle    
    }
    
    pub fn set_circle(&mut self, arc: Arc<K>, circle: Option<Circle<K>>) {
        self.arcs[arc].circle = circle;
    }
    
    pub fn face(&self, arc: Arc<K>) -> Face<K> {
        self.arcs[arc].face
    }
    
    pub fn start(&mut self, arc: Arc<K>) -> Option<Vertex<K>> {
        self.arcs[arc].start    
    }
    
    pub fn set_start(&mut self, arc: Arc<K>, start: Option<Vertex<K>>) {
        self.arcs[arc].start = start;
    }
    
    pub fn len(&self) -> usize {
        self.arcs.len()
    }

    pub fn next(&self, arc: Arc<K>) -> Arc<K> {
        self.arcs.next(arc).unwrap_or_else(|| self.arcs.first(self.arcs.root().unwrap()))
    }

    pub fn prev(&self, arc: Arc<K>) -> Arc<K> {
        self.arcs.prev(arc).unwrap_or_else(|| self.arcs.last(self.arcs.root().unwrap()))
    }

    pub fn left(&self, arc: Arc<K>) -> Option<Arc<K>> {
        self.arcs.left(arc)
    }

    pub fn right(&self, arc: Arc<K>) -> Option<Arc<K>> {
        self.arcs.right(arc)
    }

    pub fn first(&self) -> Arc<K> {
        self.arcs.first(self.root().unwrap())
    }

    pub fn last(&self) -> Arc<K> {
        self.arcs.last(self.root().unwrap())
    }
}

impl<K: Kind> Default for Beach<K> {
    fn default() -> Self {
        Beach {
            arcs: Default::default(),
        }
    }
}