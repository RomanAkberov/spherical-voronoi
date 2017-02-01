use std::cmp::Ordering;
use std::collections::BinaryHeap;
use beach::Arc;
use ideal::{Id, IdVec};
use diagram::Cell;
use point::Position;

pub struct CircleData {
    pub arc: Arc,
    pub center: Position,
    pub is_invalid: bool,
}
pub type Circle = Id<CircleData>;

#[derive(Debug)]
pub enum EventKind {
    Circle(Circle),
    Site(Cell),
}

#[derive(Debug)]
pub struct Event {
    pub theta: f64,
    pub kind: EventKind,    
}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.theta.partial_cmp(&self.theta)
    }
}

impl Ord for Event {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PartialEq for Event {
    fn eq(&self, other: &Self) -> bool {
        self.theta == other.theta
    }
}

impl Eq for Event {}

pub struct Events {
    circles: IdVec<CircleData>,
    heap: BinaryHeap<Event>,
}

impl Events {
    pub fn add_site(&mut self, face: Cell, theta: f64) {
        self.heap.push(Event { theta: theta, kind: EventKind::Site(face) });
    }
    
    pub fn add_circle(&mut self, arc: Arc, center: Position, theta: f64) -> Circle {
        let circle = self.circles.push(CircleData {
            arc: arc,
            center: center,
            is_invalid: false,
        });
        self.heap.push(Event { theta: theta, kind: EventKind::Circle(circle) });
        circle    
    }
    
    pub fn pop(&mut self) -> Option<Event> {
        self.heap.pop()
    }
    
    pub fn is_invalid(&self, event: Circle) -> bool {
        self.circles[event].is_invalid
    }
    
    pub fn set_invalid(&mut self, event: Circle, is_invalid: bool) {
        self.circles[event].is_invalid = is_invalid;
    }
    
    pub fn arc(&self, event: Circle) -> Arc {
        self.circles[event].arc
    }
    
    pub fn center(&self, event: Circle) -> Position {
        self.circles[event].center
    }

    pub fn clear(&mut self) {
        self.circles.clear();
    }
}

impl Default for Events {
    fn default() -> Self {
        Events {
            circles: Default::default(),
            heap: Default::default(),
        }
    }
}
