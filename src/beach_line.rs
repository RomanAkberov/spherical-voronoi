use std::usize::MAX as INVALID;
use std::f64::MIN as F64_MIN;
use cgmath::prelude::*;
use event::SiteEvent;
use super::Point;

const HEIGHT: usize = 5;

struct Intersection {
    theta: f64,
    phi: f64,
}

struct Circle {
    theta: f64,
    center: Point,
}

struct ArcData {
    site_index: usize,
    circle: Circle,
    intersection: Intersection,
    prev: Arc,
    next: Arc,
    prev_skips: [Arc; HEIGHT],
    next_skips: [Arc; HEIGHT],
    start: Start,
}

#[derive(Copy, Clone, PartialEq)]
pub struct Arc(usize);

#[derive(Copy, Clone)]
struct Start(usize);

pub struct BeachLine {
    arcs: Vec<ArcData>,
    free: Vec<Arc>,
    head: Arc,
    len: usize,
    levels: [usize; HEIGHT],
    starts: Vec<usize>,
}

impl BeachLine {
    pub fn insert(&mut self, site_index: usize, sites: &[SiteEvent]) -> Arc {
        let arc = self.create_arc(site_index);
        if self.len > 1 {
            let mut current = self.head;
            let mut level = HEIGHT - 1;
            let mut skips = [Arc(INVALID); HEIGHT];
            let site = &sites[site_index];
            loop {
                let next_skip = self.next_skip(current, level);
                let start = self.intersect_with_next(current, site, sites);
                let end = self.intersect_with_next(next_skip, site, sites);
                if start < end {
                    current = next_skip;
                } else {
                    skips[level] = current;
                    if level > 0 {
                        level -= 1;
                    } else {
                        break;
                    }
                }
            }
            let mut next = self.next(current);
            let mut start = self.intersect_with_next(current, site, sites);
            let mut end = self.intersect_with_next(next, site, sites);
            while start < end {
                next = self.next(next);
                start = end;
                end = self.intersect_with_next(next, site, sites);
            }
            let site_index = self.site_index(next);
            let twin = self.create_arc(site_index);
            self.add_links(twin, current, next, &mut skips);
            self.add_links(arc, twin, next, &mut skips);
        } else {
            if self.len == 0 {
                self.head = arc;
            }
            let head = self.head;
            self.add_links(arc, head, head, &mut [head; HEIGHT]);
        }
        arc
    }

    pub fn edge(&mut self, arc: Arc, end: usize) -> Option<usize> {
        let start = self.data(arc).start;
        if start.0 == INVALID {
            return None;
        }
        let vertex = self.starts[start.0];
        if vertex == INVALID {
            self.starts[start.0] = end;
            None
        } else {
            Some(vertex)
        }
    }

    pub fn set_start(&mut self, arc: Arc, vertex: usize) {
        self.data_mut(arc).start = self.add_start(vertex);
    }

    pub fn neighbors(&self, arc: Arc) -> (Arc, Arc) {
        let data = self.data(arc);
        (data.prev, data.next)
    }

    pub fn remove(&mut self, arc: Arc) {
        let head = self.head;
        if arc == head {
            let next_skip = self.next_skip(self.head, HEIGHT - 1);
            if next_skip != self.head {
                self.head = next_skip;
            } else {
                // promote next to HEIGHT
                let next = self.next(self.head);
                let height = self.height(next);
                self.levels[height - 1] -= 1;
                self.levels[HEIGHT - 1] += 1;
                for level in height..HEIGHT {
                    let next_skip = self.next_skip(self.head, level);
                    self.set_prev_skip(next_skip, level, next);
                    self.set_next_skip(next, level, next_skip);
                    self.set_prev_skip(next, level, head);
                    self.set_next_skip(head, level, next);
                }
                self.head = next;
            }
        }
        self.remove_links(arc);
        self.free.push(arc);
    }

    pub fn add_common_start(&mut self, arc0: Arc, arc1: Arc) {
        if arc0 != arc1 {
            let start = self.add_start(INVALID);
            self.data_mut(arc0).start = start;
            self.data_mut(arc1).start = start;
        } else {
            self.data_mut(arc0).start = Start(INVALID);
        }
    }
    
    pub fn site_index(&self, arc: Arc) -> usize {
        self.data(arc).site_index
    }

    pub fn circle_theta(&self, arc: Arc) -> f64 {
        self.data(arc).circle.theta
    }

    pub fn circle_center(&self, arc: Arc) -> Point {
        self.data(arc).circle.center
    }

    pub fn attach_circle(&mut self, arc: Arc, theta: f64, center: Point) {
        self.data_mut(arc).circle = Circle { theta, center };
    }

    pub fn detach_circle(&mut self, arc: Arc) {
        self.data_mut(arc).circle.theta = F64_MIN;
    }

    pub fn prev(&self, arc: Arc) -> Arc {
        self.data(arc).prev
    }

    pub fn next(&self, arc: Arc) -> Arc {
        self.data(arc).next
    }

    fn create_arc(&mut self, site_index: usize) -> Arc {
        let data = ArcData {
            site_index,
            circle: Circle { center: Point::zero(), theta: F64_MIN },
            intersection: Intersection { theta: F64_MIN, phi: F64_MIN },
            prev: Arc(INVALID),
            next: Arc(INVALID),
            prev_skips: [Arc(INVALID); HEIGHT],
            next_skips: [Arc(INVALID); HEIGHT],
            start: Start(INVALID),
        };
        if let Some(arc) = self.free.pop() {
            *self.data_mut(arc) = data;
            arc
        } else {
            self.arcs.push(data);
            Arc(self.arcs.len() - 1)
        }
    }

    fn skips(&self, arc: Arc, level: usize) -> (Arc, Arc) {
        let data = self.data(arc);
        (data.prev_skips[level], data.next_skips[level])
    }

    fn set_prev_skip(&mut self, arc: Arc, level: usize, prev: Arc) {
        self.data_mut(arc).prev_skips[level] = prev;
    }

    fn next_skip(&self, arc: Arc, level: usize) -> Arc {
        self.data(arc).next_skips[level]
    }

    fn set_next_skip(&mut self, arc: Arc, level: usize, next: Arc) {
        self.data_mut(arc).next_skips[level] = next;
    }

    fn intersect_with_next(&mut self, arc: Arc, site: &SiteEvent, sites: &[SiteEvent]) -> f64 {       
        let prev_site = &sites[self.site_index(arc)];
        let next_site = &sites[self.site_index(self.next(arc))];
        let intersection = &mut self.data_mut(arc).intersection;
        let theta = site.theta();
        if intersection.theta < theta {
            intersection.theta = theta;
            intersection.phi = site.intersect(prev_site, next_site);
        }
        intersection.phi
    }

    fn add_links(&mut self, arc: Arc, prev: Arc, next: Arc, skips: &mut [Arc; HEIGHT]) {
        self.data_mut(arc).prev = prev;
        self.data_mut(arc).next = next;
        self.data_mut(prev).next = arc;
        self.data_mut(next).prev = arc;
        let height = self.insertion_height();
        for level in 0 .. height {
            let prev = skips[level];
            let mut next = self.next_skip(prev, level);
            if next.0 == INVALID {
                next = prev;
            }
            self.set_prev_skip(arc, level, prev);
            self.set_next_skip(arc, level, next);
            self.set_prev_skip(next, level, arc);
            self.set_next_skip(prev, level, arc);
            skips[level] = arc;
        }
        self.len += 1;
        self.levels[height - 1] += 1;
    }

    fn remove_links(&mut self, arc: Arc) {
        let (prev, next) = self.neighbors(arc);
        self.data_mut(prev).next = next;
        self.data_mut(next).prev = prev;
        let height = self.height(arc);
        for level in 0..height {
            let (prev_skip, next_skip) = self.skips(arc, level);
            self.set_prev_skip(next_skip, level, prev_skip);
            self.set_next_skip(prev_skip, level, next_skip);
        }
        self.len -= 1;
        self.levels[height - 1] -= 1;
    }

    fn height(&self, arc: Arc) -> usize {
        for level in 0 .. HEIGHT {
            if self.next_skip(arc, level).0 == INVALID {
                return level;
            }
        }
        HEIGHT
    }

    fn insertion_height(&self) -> usize {
        if self.len == 0 {
            return HEIGHT;
        }
        let mut best_height = 1;
        let mut best_ratio = self.levels[0];
        let mut multiplier = 1;
        for level in 0 .. HEIGHT {
            let ratio = self.levels[level] * multiplier;
            if ratio < best_ratio {
                best_ratio = ratio;
                best_height = level + 1;
            }
            multiplier *= 2;
        }
        best_height
    }

    fn add_start(&mut self, vertex: usize) -> Start {
        self.starts.push(vertex);
        Start(self.starts.len() - 1)
    }

    fn data(&self, arc: Arc) -> &ArcData {
        &self.arcs[arc.0]
    }

    fn data_mut(&mut self, arc: Arc) -> &mut ArcData {
        &mut self.arcs[arc.0]
    }
}

impl Default for BeachLine {
    fn default() -> Self {
        Self {
            arcs: Vec::default(),
            free: Vec::default(),
            head: Arc(INVALID),
            len: 0,
            levels: [0; HEIGHT],
            starts: Vec::default(),
        }
    }
}