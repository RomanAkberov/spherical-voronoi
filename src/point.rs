use std::cmp::Ordering;
use std::fmt;
use cgmath::{Vector3, Point3, InnerSpace, EuclideanSpace};
use angle::Angle;

#[derive(Copy, Clone)]
pub struct Point {
    theta: Angle,
    phi: Angle,
}

impl Point {
    pub fn from_cartesian(x: f64, y: f64, z: f64) -> Self {
        let position = Point3::from_vec(Vector3::new(x, y, z).normalize());
        let (theta, phi) = (position.z.acos(), position.y.atan2(position.x));
        Point {
            theta: Angle::from(theta),
            phi: Angle::from(phi),
        }
    }
    
    pub fn from_angles(theta: Angle, phi: Angle) -> Self {
        Point {
            theta: theta,
            phi: phi,
        }
    }
    
    pub fn distance(&self, other: &Self) -> f64 {
        self.position().dot(other.position().to_vec()).acos()
    }
    
    pub fn position(&self) -> Point3<f64> {
        Point3::new(self.theta.sin() * self.phi.cos(), self.theta.sin() * self.phi.sin(), self.theta.cos())
    }

    pub fn phi(&self) -> &Angle { 
        &self.phi
    }
    
    pub fn theta(&self) -> &Angle {
        &self.theta
    }
}

impl PartialOrd for Point {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.theta().partial_cmp(&other.theta()) {
            Some(Ordering::Equal) => self.phi().partial_cmp(&other.phi()),
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
        (self.theta() == other.theta()) && (self.phi() == other.phi())
    }
}

impl fmt::Debug for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let position = self.position();
        write!(f, "({:.5}, {:.5}, {:.5})", position.x, position.y, position.z)
    }
}
