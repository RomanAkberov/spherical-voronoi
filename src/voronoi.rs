use std::collections::BTreeSet;
use cgmath::prelude::*;
use event::{SiteEvent, CircleEvent};
use beach_line::{BeachLine, Arc};
use common::{Point, Vertex, Cell};

pub trait Visitor {
    fn vertex(&mut self, point: Point, cells: [usize; 3]);
    fn edge(&mut self,vertices: [usize; 2]);
    fn cell(&mut self);
}

struct Voronoi {
    next_vertex: Vertex,
    next_cell: Cell,
    site_events: Vec<SiteEvent>,
    circle_events: BTreeSet<CircleEvent>,
    beach: BeachLine,
}

impl Voronoi {
    fn new(points: &[Point]) -> Self {
        let mut site_events: Vec<SiteEvent> = points.iter().map(SiteEvent::new).collect();
        site_events.sort();
        Self {
            next_vertex: Vertex(0),
            next_cell: Cell(0),
            site_events,
            circle_events: Default::default(),
            beach: Default::default(),
        }
    }

    fn build<V: Visitor>(mut self, visitor: &mut V) {
        loop {
            let has_sites = self.next_cell.0 < self.site_events.len();
            if let Some(circle) = self.circle_events.iter().next().cloned() {
                if has_sites && self.site_events[self.next_cell.0].theta.value < circle.theta {
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
        let cell = self.next_cell;
        self.next_cell.0 += 1;
        visitor.cell();
        let theta = self.site_events[cell.0].theta.value;
        let arc = self.beach.insert(cell, &self.site_events);
        let (prev, next) = self.beach.neighbors(arc);
        self.beach.add_common_start(arc, prev);
        if prev != next {
            self.detach_circle(prev);
            self.detach_circle(next);
            self.attach_circle(prev, theta);
            self.attach_circle(next, theta);
        }
    }

    fn circle_event<V: Visitor>(&mut self, event: &CircleEvent, visitor: &mut V) {
        self.circle_events.remove(event);
        let arc = event.arc;
        let theta = event.theta;
        let (prev, next) = self.beach.neighbors(arc);
        self.beach.detach_circle(arc);
        self.detach_circle(prev);
        self.detach_circle(next);
        let vertex = self.next_vertex;
        self.next_vertex.0 += 1;
        let point = self.beach.circle_center(arc);
        let cell0 = self.beach.cell(prev);
        let cell1 = self.beach.cell(arc);
        let cell2 = self.beach.cell(next);
        visitor.vertex(point, [cell0.0, cell1.0, cell2.0]);
        self.edge(prev, vertex, visitor);
        self.edge(arc, vertex, visitor);
        self.beach.remove(arc);
        if self.beach.prev(prev) == next {
            self.edge(next, vertex, visitor);
            self.beach.remove(prev);
            self.beach.remove(next);
        } else {
            if self.attach_circle(prev, theta) {
                self.beach.set_start(prev, vertex);
            }
            self.attach_circle(next, theta);
        }
    }

    fn attach_circle(&mut self, arc: Arc, min: f64) -> bool {
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
        self.site_events[self.beach.cell(arc).0].point
    }

    fn edge<V: Visitor>(&mut self, arc: Arc, end: Vertex, visitor: &mut V) {
        if let Some(start) = self.beach.edge(arc, end) {
            visitor.edge([start.0, end.0]);
        }
    }
}

pub fn build<V: Visitor>(points: &[Point], visitor: &mut V) {
    Voronoi::new(points).build(visitor);
}