use events::Circle;
use diagram::{Vertex, Face};
use id::IdVec;

pub struct ArcData {
    face: Face,
    start: Option<Vertex>,
    circle: Option<Circle>,
}
create_id!(Arc);

#[derive(Default)]
pub struct Beach {
    all_arcs: IdVec<Arc, ArcData>,
    active_arcs: Vec<Arc>,   
}

impl Beach {
    pub fn index(&self, arc: Arc) -> Option<usize> {
        self.active_arcs.iter().position(|x| *x == arc)
    }
    
    pub fn add(&mut self, index: usize, face: Face) -> Arc {
        let arc = self.all_arcs.push(ArcData {
            face: face,
            start: None,
            circle: None,
        });
        self.active_arcs.insert(index, arc);
        arc
    }
    
    pub fn circle(&self, arc: Arc) -> Option<Circle> {
        self.all_arcs[arc].circle    
    }
    
    pub fn set_circle(&mut self, arc: Arc, circle: Option<Circle>) {
        self.all_arcs[arc].circle = circle;
    }
    
    pub fn face(&self, arc: Arc) -> Face {
        self.all_arcs[arc].face
    }
    
    pub fn start(&mut self, arc: Arc) -> Option<Vertex> {
        self.all_arcs[arc].start    
    }
    
    pub fn set_start(&mut self, arc: Arc, start: Option<Vertex>) {
        self.all_arcs[arc].start = start;
    }
    
    pub fn len(&self) -> usize {
        self.active_arcs.len()
    }
    
    pub fn get(&self, index: usize) -> Arc {
        self.active_arcs[index]
    }
    
    pub fn remove(&mut self, index: usize) {
        self.active_arcs.remove(index);
    }
        
    pub fn prev_index(&self, index: usize) -> usize {
        (index + self.len() - 1) % self.len()
    }
    
    pub fn next_index(&self, index: usize) -> usize {
        (index + 1) % self.len()
    }
}
