use std::cmp::Ordering;
use cgmath::{Vector3, InnerSpace};
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


#[derive(Copy, Clone)]
pub struct Angle {
    pub value: f64,
    pub sin: f64,
    pub cos: f64,
}

impl From<f64> for Angle {
    fn from(value: f64) -> Self {
        let (sin, cos) = value.sin_cos();
        Angle {
            value: value,
            sin: sin,
            cos: cos,
        }
    }
}

#[derive(Copy, Clone)]
pub struct SiteEvent {
    pub theta: Angle,
    pub phi: Angle,
    pub position: Vector3<f64>,
}

impl From<Vector3<f64>> for SiteEvent {
    fn from(v: Vector3<f64>) -> Self {
        let position = v.normalize();
        let (theta, phi) = (position.z.acos(), position.y.atan2(position.x));
        SiteEvent {
            theta: Angle::from(theta),
            phi: Angle::from(phi),
            position: position,
        }
    }
}

impl PartialOrd for SiteEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.theta.value.partial_cmp(&other.theta.value)
    }
}

impl Ord for SiteEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PartialEq for SiteEvent {
    fn eq(&self, other: &Self) -> bool {
        self.theta.value == other.theta.value
    }
}

impl Eq for SiteEvent {}
