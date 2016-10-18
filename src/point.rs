use std::cmp::Ordering;
use std::fmt;
use cgmath::prelude::*;
use cgmath::{Vector3, Point3};

#[derive(Copy, Clone)]
pub struct SinCosCache {
    pub value: f64,
    pub sin: f64,
    pub cos: f64,
}

impl From<f64> for SinCosCache {
    fn from(value: f64) -> Self {
        let (sin, cos) = value.sin_cos();
        SinCosCache {
            value: value,
            sin: sin,
            cos: cos,
        }
    }
}

#[derive(Copy, Clone)]
pub struct Point {
    pub theta: SinCosCache,
    pub phi: SinCosCache,
    pub position: Point3<f64>,
}

impl Point {
    pub fn from_cartesian(x: f64, y: f64, z: f64) -> Self {
        let position = Point3::from_vec(Vector3::new(x, y, z).normalize());
        let (theta, phi) = (position.z.acos(), position.y.atan2(position.x));
        Point {
            theta: theta.into(),
            phi: phi.into(),
            position: position
        }
    }
    
    pub fn from_cache(theta: SinCosCache, phi: SinCosCache) -> Self {
        Point {
            theta: theta,
            phi: phi,
            position: Point3::new(theta.sin * phi.cos, theta.sin * phi.sin, theta.cos),
        }
    }

    pub fn from_spherical(theta: f64, phi: f64) -> Self {
        Point::from_cache(theta.into(), phi.into())
    }
    
    pub fn distance(&self, other: &Self) -> f64 {
        self.position.dot(other.position.to_vec()).acos()
    }
    
    pub fn x(&self) -> f64 { self.position.x }
    pub fn y(&self) -> f64 { self.position.y }
    pub fn z(&self) -> f64 { self.position.z }

    pub fn phi(&self) -> f64 { self.phi.value }
    pub fn theta(&self) -> f64 { self.theta.value }
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
        write!(f, "({:.5}, {:.5}, {:.5})", self.x(), self.y(), self.z())
    }
}
