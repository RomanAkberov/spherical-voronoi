use std::cmp::Ordering;
use std::collections::BinaryHeap;
use point::Point;
use beach::Arc;
use diagram::Face;
use id::{Id, Pool};

pub struct CircleData {
    pub arcs: (Arc, Arc, Arc),
    pub center: Point,
    pub radius: f64,
    pub is_invalid: bool,
}
pub type Circle = Id<CircleData>;

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.point.partial_cmp(&self.point)
    }
}

impl Ord for Event {
    fn cmp(&self, other: &Self) -> Ordering {
        other.point.cmp(&self.point)
    }
}

impl PartialEq for Event {
    fn eq(&self, other: &Self) -> bool {
        self.point == other.point
    }
}

impl Eq for Event {}

pub enum EventKind {
    Circle(Circle),
    Site(Face),
}

pub struct Event {
    pub point: Point,
    pub kind: EventKind,    
}

pub struct Events {
    circles: Pool<CircleData>,
    heap: BinaryHeap<Event>,
}

impl Events {
    pub fn new() -> Self {
        Events {
            circles: Pool::new(),
            heap: BinaryHeap::new(),
        }    
    }
    
    pub fn add_site(&mut self, face: Face, point: Point) {
        self.heap.push(Event { point: point, kind: EventKind::Site(face) });
    }
    
    pub fn add_circle(&mut self, arcs: (Arc, Arc, Arc), center: Point, radius: f64, point: Point) -> Circle {
        let circle = self.circles.add(CircleData {
            arcs: arcs,
            center: center,
            radius: radius,
            is_invalid: false,
        });
        self.heap.push(Event { point: point, kind: EventKind::Circle(circle) });
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
    
    pub fn arcs(&self, event: Circle) -> (Arc, Arc, Arc) {
        self.circles[event].arcs
    }
    
    pub fn center(&self, event: Circle) -> Point {
        self.circles[event].center
    }
}
