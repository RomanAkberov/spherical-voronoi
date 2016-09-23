use events::Circle;
use diagram::{Face, Vertex};
use id::{Id, Pool};

pub struct ArcData {
    face: Face,
    start: Option<Vertex>,
    circle: Option<Circle>,
}
pub type Arc = Id<ArcData>;

pub struct Beach {
    data: Pool<ArcData>,
    arcs: Vec<Arc>,   
}

impl Beach {
    pub fn new() -> Self {
        Beach {
            data: Pool::new(),
            arcs: Vec::new(),
        }
    }
    
    pub fn index(&self, arc: Arc) -> Option<usize> {
        self.arcs.iter().position(|x| *x == arc)
    }
    
    pub fn add(&mut self, index: usize, face: Face) -> Arc {
        let arc = self.data.add(ArcData {
            face: face,
            start: None,
            circle: None,
        });
        self.arcs.insert(index, arc);
        arc
    }
    
    pub fn circle(&self, arc: Arc) -> Option<Circle> {
        self.data[arc].circle    
    }
    
    pub fn set_circle(&mut self, arc: Arc, circle: Option<Circle>) {
        self.data[arc].circle = circle;
    }
    
    pub fn face(&self, arc: Arc) -> Face {
        self.data[arc].face
    }
    
    pub fn start(&mut self, arc: Arc) -> Option<Vertex> {
        self.data[arc].start    
    }
    
    pub fn set_start(&mut self, arc: Arc, start: Option<Vertex>) {
        self.data[arc].start = start;
    }
    
    pub fn len(&self) -> usize {
        self.arcs.len()
    }
    
    pub fn get(&self, index: usize) -> Arc {
        self.arcs[index]
    }
    
    pub fn remove(&mut self, index: usize) {
        self.arcs.remove(index);
    }
        
    pub fn prev_index(&self, index: usize) -> usize {
        (index + self.len() - 1) % self.len()
    }
    
    pub fn next_index(&self, index: usize) -> usize {
        (index + 1) % self.len()
    }
}
