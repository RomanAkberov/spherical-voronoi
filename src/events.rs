use std::cmp::Ordering;
use beach_line::Arc;

#[derive(Debug)]
pub struct CircleEvent {
    pub theta: f64,
    pub arc: Arc,
}

impl PartialOrd for CircleEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.theta.partial_cmp(&self.theta)
    }
}

impl Ord for CircleEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PartialEq for CircleEvent {
    fn eq(&self, other: &Self) -> bool {
        self.theta == other.theta
    }
}

impl Eq for CircleEvent {}
