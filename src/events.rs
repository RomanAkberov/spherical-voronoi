use std::cmp::Ordering;
use beach::Arc;
use point::Point;

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

#[derive(Debug)]
pub struct SiteEvent {
    pub point: Point,
}

impl PartialOrd for SiteEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.point.theta.value.partial_cmp(&self.point.theta.value)
    }
}

impl Ord for SiteEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PartialEq for SiteEvent {
    fn eq(&self, other: &Self) -> bool {
        self.point.theta.value == other.point.theta.value
    }
}

impl Eq for SiteEvent {}
