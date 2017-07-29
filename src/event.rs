use std::cmp::Ordering;
use std::f64::consts::{PI, FRAC_1_PI};
use cgmath::InnerSpace;
use beach_line::Arc;
use super::Point;

#[derive(Copy, Clone)]
pub struct CircleEvent {
    pub theta: f64,
    pub arc: Arc,
}

impl PartialOrd for CircleEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.theta.partial_cmp(&other.theta)
    }
}

impl Ord for CircleEvent {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> Ordering {
        if self.theta < other.theta {
            Ordering::Less
        } else if self.theta > other.theta {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

impl PartialEq for CircleEvent {
    fn eq(&self, other: &Self) -> bool {
        self.theta == other.theta
    }
}

impl Eq for CircleEvent {}

#[derive(Copy, Clone)]
struct Angle {
    value: f64,
    sin: f64,
    cos: f64,
}

impl From<f64> for Angle {
    fn from(value: f64) -> Self {
        let (sin, cos) = value.sin_cos();
        Angle { value, sin, cos }
    }
}

#[derive(Copy, Clone)]
pub struct SiteEvent {
    pub point: Point,
    theta: Angle,
    phi: Angle,
}

impl SiteEvent {
    pub fn new<P: AsRef<[f64; 3]>>(point: &P) -> Self {
        let point = Point::from(*point.as_ref()).normalize();
        let (theta, phi) = (point.z.acos(), point.y.atan2(point.x));
        SiteEvent {
            point,
            theta: Angle::from(theta),
            phi: Angle::from(phi),
        }
    }

    pub fn theta(&self) -> f64 {
        self.theta.value
    }

    pub fn intersect(&self, site0: &SiteEvent, site1: &SiteEvent) -> f64 {
        let u1 = (self.theta.cos - site1.theta.cos) * site0.theta.sin;
        let u2 = (self.theta.cos - site0.theta.cos) * site1.theta.sin;
        let a = u1 * site0.phi.cos - u2 * site1.phi.cos;
        let b = u1 * site0.phi.sin - u2 * site1.phi.sin;
        let c = (site0.theta.cos - site1.theta.cos) * self.theta.sin;
        let length = (a * a + b * b).sqrt();
        let gamma = a.atan2(b);
        let phi_plus_gamma = (c / length).asin();
        wrap(phi_plus_gamma - gamma - self.phi.value)
    }
}

impl PartialOrd for SiteEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.theta.value.partial_cmp(&other.theta.value)
    }
}

impl Ord for SiteEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.theta.value < other.theta.value {
            Ordering::Less
        } else if self.theta.value > other.theta.value {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

impl PartialEq for SiteEvent {
    fn eq(&self, other: &Self) -> bool {
        self.theta.value == other.theta.value
    }
}

impl Eq for SiteEvent {}

fn wrap(mut phi: f64) -> f64 {
    phi *= 0.5 * FRAC_1_PI;
    phi -= phi.floor();
    phi * 2.0 * PI
}