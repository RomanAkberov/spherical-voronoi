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
pub struct SiteEvent {
    pub point: Point,
    pub theta: f64,
    pub phi: f64,
    pub sin_theta: f64,
}

impl SiteEvent {
    pub fn new(point: &Point) -> Self {
        let point = point.normalize();
        let (theta, phi) = (point.z.acos(), point.y.atan2(point.x));
        let sin_theta = theta.sin();
        SiteEvent {
            point,
            theta,
            phi,
            sin_theta,
        }
    }

    pub fn intersect(&self, prev: &SiteEvent, next: &SiteEvent) -> f64 {
        let d_prev = self.point.z - prev.point.z;
        let d_next = self.point.z - next.point.z;
        let a = d_next * prev.point.x - d_prev * next.point.x;
        let b = d_next * prev.point.y - d_prev * next.point.y;
        let c = (prev.point.z - next.point.z) * self.sin_theta;
        let length = (a * a + b * b).sqrt();
        reduce_angle((c / length).asin() - a.atan2(b) - self.phi)
    }
}

impl PartialOrd for SiteEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.theta.partial_cmp(&other.theta)
    }
}

impl Ord for SiteEvent {
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

impl PartialEq for SiteEvent {
    fn eq(&self, other: &Self) -> bool {
        self.theta == other.theta
    }
}

impl Eq for SiteEvent {}

fn reduce_angle(mut phi: f64) -> f64 {
    phi *= 0.5 * FRAC_1_PI;
    phi -= phi.floor();
    phi * 2.0 * PI
}