use std::cmp::Ordering;
use std::rc::Rc;
use std::cell::Cell;
use point::Point;
use arc::Arc;

pub struct CircleEvent {
    pub arcs: (Rc<Arc>, Rc<Arc>, Rc<Arc>),
    pub center: Point,
    pub radius: f64,
    pub point: Point,
    pub is_invalid: Cell<bool>,
}

pub enum Event {
    Site(usize, Point),
    Circle(Rc<CircleEvent>),
}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.point().partial_cmp(&self.point())
    }
}

impl Ord for Event {
    fn cmp(&self, other: &Self) -> Ordering {
        other.point().cmp(&self.point())
    }
}

impl PartialEq for Event {
    fn eq(&self, other: &Self) -> bool {
        self.point() == other.point()
    }
}

impl Eq for Event {}

impl Event {
    pub fn point(&self) -> Point {
        match self {
            &Event::Site(_, point) => point,
            &Event::Circle(ref circle) => circle.point,
        }
    }
}