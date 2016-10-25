use std::cmp::Ordering;
use std::collections::BinaryHeap;
use point::Point;
use beach::Arc;
use ideal::{Id, IdVec};
use diagram::{Kind, Face};

pub struct CircleData<K: Kind> {
    pub arcs: (Arc<K>, Arc<K>, Arc<K>),
    pub center: Point,
    pub radius: f64,
    pub is_invalid: bool,
}
pub type Circle<K> = Id<CircleData<K>>;

pub enum EventKind<K: Kind> {
    Circle(Circle<K>),
    Site(Face<K>),
}

pub struct Event<K: Kind> {
    pub point: Point,
    pub kind: EventKind<K>,    
}

impl<K: Kind> PartialOrd for Event<K> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.point.partial_cmp(&self.point)
    }
}

impl<K: Kind> Ord for Event<K> {
    fn cmp(&self, other: &Self) -> Ordering {
        other.point.cmp(&self.point)
    }
}

impl<K: Kind> PartialEq for Event<K> {
    fn eq(&self, other: &Self) -> bool {
        self.point == other.point
    }
}

impl<K: Kind> Eq for Event<K> {}

pub struct Events<K: Kind> {
    circles: IdVec<CircleData<K>>,
    heap: BinaryHeap<Event<K>>,
}

impl<K: Kind> Events<K> {
    pub fn add_site(&mut self, face: Face<K>, point: Point) {
        self.heap.push(Event { point: point, kind: EventKind::Site(face) });
    }
    
    pub fn add_circle(&mut self, arcs: (Arc<K>, Arc<K>, Arc<K>), center: Point, radius: f64, point: Point) -> Circle<K> {
        let circle = self.circles.push(CircleData {
            arcs: arcs,
            center: center,
            radius: radius,
            is_invalid: false,
        });
        self.heap.push(Event { point: point, kind: EventKind::Circle(circle) });
        circle    
    }
    
    pub fn pop(&mut self) -> Option<Event<K>> {
        self.heap.pop()
    }
    
    pub fn is_invalid(&self, event: Circle<K>) -> bool {
        self.circles[event].is_invalid
    }
    
    pub fn set_invalid(&mut self, event: Circle<K>, is_invalid: bool) {
        self.circles[event].is_invalid = is_invalid;
    }
    
    pub fn arcs(&self, event: Circle<K>) -> (Arc<K>, Arc<K>, Arc<K>) {
        self.circles[event].arcs
    }
    
    pub fn center(&self, event: Circle<K>) -> Point {
        self.circles[event].center
    }
}

impl<K: Kind> Default for Events<K> {
    fn default() -> Self {
        Events {
            circles: Default::default(),
            heap: Default::default(),
        }
    }
}