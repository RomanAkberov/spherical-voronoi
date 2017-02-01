use std::fmt;
use cgmath::{Vector3, InnerSpace};
use angle::Angle;

pub type Position = Vector3<f64>;

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
