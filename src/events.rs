use std::cmp::Ordering;
use beach::Arc;
use diagram::Cell;

#[derive(Debug)]
pub enum EventKind {
    Circle(Arc),
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
