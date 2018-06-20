use std::f64::consts::{PI, FRAC_1_PI};
use cgmath::prelude::*;
use super::Point;

#[derive(Copy, Clone)]
pub struct CellEvent {
    pub point: Point,
    pub theta: f64,
    pub phi: f64,
}

impl CellEvent {
    pub fn new(point: &Point) -> Self {
        let point = point.normalize();
        let (theta, phi) = (point.z.acos(), point.y.atan2(point.x));
        Self { point, theta, phi }
    }

    pub fn intersect(&self, point0: &Point, point1: &Point) -> f64 {
        let dz0 = self.point.z - point0.z;
        let dz1 = self.point.z - point1.z;
        let a = dz1 * point0.x - dz0 * point1.x;
        let b = dz1 * point0.y - dz0 * point1.y;
        let c = (point0.z - point1.z) * self.theta.sin();
        let length = (a * a + b * b).sqrt();
        reduce_angle((c / length).asin() - a.atan2(b) - self.phi)
    }
}

fn reduce_angle(mut phi: f64) -> f64 {
    phi *= 0.5 * FRAC_1_PI;
    phi -= phi.floor();
    phi * 2.0 * PI
}
