use std::collections::BTreeSet;
use cgmath::prelude::*;
use event::{SiteEvent, CircleEvent};
use beach_line::{BeachLine, Arc};
use super::Point;

pub trait Visitor {
    fn vertex(&mut self, point: Point, cells: [usize; 3]);
    fn edge(&mut self, vertices: [usize; 2]);
    fn cell(&mut self);
}

struct Voronoi {
    vertex_index: usize,
    site_index: usize,
    site_events: Vec<SiteEvent>,
    circle_events: BTreeSet<CircleEvent>,
    beach: BeachLine,
}

impl Voronoi {
    fn new(points: &[Point]) -> Self {
        let mut site_events: Vec<SiteEvent> = points.iter().map(SiteEvent::new).collect();
        site_events.sort();
        Self {
            vertex_index: 0,
            site_index: 0,
            site_events,
            circle_events: Default::default(),
            beach: Default::default(),
        }
    }

    fn build<V: Visitor>(mut self, visitor: &mut V) {
        loop {
            let has_sites = self.site_index < self.site_events.len();
            if let Some(circle) = self.circle_events.iter().next().cloned() {
                if has_sites && self.site_events[self.site_index].theta() < circle.theta {
                    self.site_event(visitor);
                } else {
                    self.circle_event(&circle, visitor);
                }
            } else if has_sites {
                self.site_event(visitor);
            } else {
                break;
            }
        }
    }

    fn site_event<V: Visitor>(&mut self, visitor: &mut V) {
        visitor.cell();
        let theta = self.site_events[self.site_index].theta();
        let arc = self.beach.insert(self.site_index, &self.site_events);
        let (prev, next) = self.beach.neighbors(arc);
        self.beach.add_common_start(arc, prev);
        if prev != next {
            self.attach_circle(prev, theta);
            self.attach_circle(next, theta);
        }
        self.site_index += 1;
    }

    fn circle_event<V: Visitor>(&mut self, event: &CircleEvent, visitor: &mut V) {
        let arc = event.arc;
        let theta = event.theta;
        let (prev, next) = self.beach.neighbors(arc);
        self.detach_circle(arc);
        self.detach_circle(prev);
        self.detach_circle(next);
        let point = self.beach.circle_center(arc);
        let cell0 = self.beach.site_index(prev);
        let cell1 = self.beach.site_index(arc);
        let cell2 = self.beach.site_index(next);
        visitor.vertex(point, [cell0, cell1, cell2]);
        let vertex_index = self.vertex_index;
        self.edge(prev, vertex_index, visitor);
        self.edge(arc, vertex_index, visitor);
        self.beach.remove(arc);
        if self.beach.prev(prev) == next {
            self.edge(next, vertex_index, visitor);
            self.beach.remove(prev);
            self.beach.remove(next);
        } else {
            if self.attach_circle(prev, theta) {
                self.beach.set_start(prev, vertex_index);
            }
            self.attach_circle(next, theta);
        }
        self.vertex_index += 1;
    }

    fn attach_circle(&mut self, arc: Arc, min: f64) -> bool {
        self.detach_circle(arc);
        let (prev, next) = self.beach.neighbors(arc);
        let point = self.arc_point(arc);
        let from_prev = self.arc_point(prev) - point;
        let from_next = self.arc_point(next) - point;
        let center = from_prev.cross(from_next).normalize();
        let theta = center.z.acos() + center.dot(point).acos();
        if theta >= min {
            self.beach.attach_circle(arc, theta, center);
            self.circle_events.insert(CircleEvent {
                theta: theta,
                arc: arc,
            });
            true
        } else {
            false
        }
    }

    fn detach_circle(&mut self, arc: Arc) {
        let theta = self.beach.circle_theta(arc);
        if theta >= 0.0 {
            self.circle_events.remove(&CircleEvent {
                arc: arc,
                theta: theta,
            });
            self.beach.detach_circle(arc);
        }
    }

    fn arc_point(&self, arc: Arc) -> Point {
        self.site_events[self.beach.site_index(arc)].point
    }

    fn edge<V: Visitor>(&mut self, arc: Arc, start: usize, visitor: &mut V) {
        if let Some(end) = self.beach.edge(arc, start) {
            visitor.edge([start, end]);
        }
    }
}

pub fn build<V: Visitor>(points: &[Point], visitor: &mut V) {
    Voronoi::new(points).build(visitor);
}