use std::cmp::Ordering;
use nalgebra::{Vector3, Dot, Norm};
use std::fmt;

#[derive(Copy, Clone)]
pub struct Point {
    pub theta: f64,
    pub phi: f64,
    pub position: Vector3<f64>,
}

impl Point {
    pub fn from_cartesian(direction: Vector3<f64>) -> Self {
        let position = direction.normalize();
        Point {
            theta: position.z.acos(),
            phi: position.y.atan2(position.x),
            position: position,
        }
    }
    
    pub fn from_spherical(theta: f64, phi: f64) -> Self {
        let (sin_theta, cos_theta) = theta.sin_cos();
        let (sin_phi, cos_phi) = phi.sin_cos();
        Point {
            theta: theta,
            phi: phi,
            position: Vector3::new(sin_theta * cos_phi, sin_theta * sin_phi, cos_theta)
        }
    }
    
    pub fn distance(&self, other: &Self) -> f64 {
        self.position.dot(&other.position).acos()
    }
    
    pub fn x(&self) -> f64 { self.position.x }
    pub fn y(&self) -> f64 { self.position.y }
    pub fn z(&self) -> f64 { self.position.z }
}

impl PartialOrd for Point {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.theta.partial_cmp(&other.theta) {
            Some(Ordering::Equal) => self.phi.partial_cmp(&other.phi),
            other => other,
        }
    }
}

impl Ord for Point {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(&other).unwrap()
    }
}

impl Eq for Point {}

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        (self.theta == other.theta) && (self.phi == other.phi)
    }
}

impl fmt::Debug for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({:.5}, {:.5}, {:.5})", self.x(), self.y(), self.z())
    }
}