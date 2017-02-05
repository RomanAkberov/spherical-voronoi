use std::fmt;
use std::cmp::Ordering;
use cgmath::{Vector3, InnerSpace};

pub type Position = Vector3<f64>;

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
pub struct Point {
    pub theta: Angle,
    pub phi: Angle,
    pub position: Position,
}

impl fmt::Debug for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.position.fmt(f)
    }
}

impl From<Position> for Point {
    fn from(v: Position) -> Self {
        let position = v.normalize();
        let (theta, phi) = (position.z.acos(), position.y.atan2(position.x));
        Point {
            theta: Angle::from(theta),
            phi: Angle::from(phi),
            position: position,
        }
    }
}

impl PartialOrd for Point {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.theta.value.partial_cmp(&other.theta.value)
    }
}

impl Ord for Point {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        self.theta.value == other.theta.value
    }
}

impl Eq for Point {}
