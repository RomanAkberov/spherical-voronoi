use std::f64::consts::{PI, FRAC_1_PI};
use cgmath::Vector3;
use ideal::{Id, IdVec};
use event::SiteEvent;
use diagram::{Vertex, Cell};

const HEIGHT: usize = 8;

#[derive(Debug)]
pub struct Start {
    pub vertex: Vertex,
}

#[derive(Debug)]
pub struct ArcData {
    cell: Cell,
    start: Id<Start>,
    center: Option<Vector3<f64>>,
    scan: f64,
    end: f64,
    prev: Arc,
    next: Arc,
    prev_skips: [Arc; HEIGHT],
    next_skips: [Arc; HEIGHT],
}

pub type Arc = Id<ArcData>;

#[derive(Debug)]
pub struct BeachLine {
    pub arcs: IdVec<ArcData>,
    head: Arc,
    len: usize,
    levels: [usize; HEIGHT],
}

impl BeachLine {
    pub fn insert(&mut self, cell: Cell, sites: &[SiteEvent]) -> Arc {
        if self.len > 1 {
            let mut current = self.head;
            let mut level = HEIGHT - 1;
            let mut skips = [Arc::invalid(); HEIGHT];
            let site = &sites[cell.index()];
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
            current = next;
            let current_cell = self.cell(current);
            let twin = self.create_arc(current_cell);
            let prev = self.prev(current);  
            self.insert_after(prev, twin);     
            let new = self.create_arc(cell);
            self.insert_after(twin, new);
            self.add_skips(new, &mut skips);
            self.add_skips(twin, &mut skips);
            new
        } else if self.len > 0 {        
            let new = self.create_arc(cell);
            let head = self.head;
            self.insert_after(head, new);
            let height = self.insertion_height();
            self.levels[height - 1] += 1;
            for level in 0..height {;
                self.set_prev_skip(new, level, head);
                self.set_next_skip(new, level, head);
                self.set_prev_skip(head, level, new);
                self.set_next_skip(head, level, new);
            }
            new
        } else {
            let new = self.create_arc(cell);
            {
                let data = &mut self.arcs[new];
                data.prev = new;
                data.next = new;
                data.prev_skips = [new; HEIGHT];
                data.next_skips = [new; HEIGHT];
            }
            self.head = new;
            self.levels[HEIGHT - 1] += 1;
            self.len += 1;
            new
        }
    }
    
    fn create_arc(&mut self, cell: Cell) -> Arc {
        self.arcs.push(ArcData {
            cell: cell,
            start: Id::invalid(),
            center: None,
            scan: -1.0,
            end: 0.0,
            prev: Arc::invalid(),
            next: Arc::invalid(),
            prev_skips: [Arc::invalid(); HEIGHT],
            next_skips: [Arc::invalid(); HEIGHT],
        })
    }

    // fn range_end(&mut self, arc1: Arc, arc2: Arc, site: &SiteEvent, diagram: &Diagram) -> f64 {
    //     if self.arcs[arc1].scan < site.theta.value {
    //         self.arcs[arc1].end = self.intersect_with_next(arc1, site, diagram);
    //         self.arcs[arc2].end = self.intersect_with_next(arc2, site, diagram);
    //         self.arcs[arc1].scan = site.theta.value;
    //         self.arcs[arc2].scan = site.theta.value;
    //     }
    //     self.arcs[arc1].end
    // }

    fn skips(&self, arc: Arc, level: usize) -> (Arc, Arc) {
        let data = &self.arcs[arc];
        (data.prev_skips[level], data.next_skips[level])
    }

    // fn prev_skip(&self, arc: Arc, level: usize) -> Arc {
    //     self.arcs[arc].prev_skips[level]
    // }

    fn set_prev_skip(&mut self, arc: Arc, level: usize, prev: Arc) {
        self.arcs[arc].prev_skips[level] = prev;
    }

    fn next_skip(&self, arc: Arc, level: usize) -> Arc {
        self.arcs[arc].next_skips[level]
    }

    fn set_next_skip(&mut self, arc: Arc, level: usize, next: Arc) {
        self.arcs[arc].next_skips[level] = next;
    }

    fn add_skips(&mut self, arc: Arc, skips: &mut[Arc; HEIGHT]) {
        let height = self.insertion_height();
        self.levels[height - 1] += 1;
        for level in 0..height {
            let prev = skips[level];
            let next = self.next_skip(prev, level);
            self.set_prev_skip(arc, level, prev);
            self.set_next_skip(arc, level, next);
            self.set_prev_skip(next, level, arc);
            self.set_next_skip(prev, level, arc);
            skips[level] = arc;
        }
    }

    fn intersect_with_next(&mut self, arc: Arc, site: &SiteEvent, sites: &[SiteEvent]) -> f64 {
        let arc_point = &sites[self.cell(arc).index()];
        let next_point = &sites[self.cell(self.next(arc)).index()];
        let data = &mut self.arcs[arc];
        if data.scan < site.theta.value {
            data.scan = site.theta.value;
            data.end = BeachLine::intersect(arc_point, next_point, site);
        }
        data.end
    }

    fn intersect(site0: &SiteEvent, site1: &SiteEvent, site2: &SiteEvent) -> f64 {
        let u1 = (site2.theta.cos - site1.theta.cos) * site0.theta.sin;
        let u2 = (site2.theta.cos - site0.theta.cos) * site1.theta.sin;
        let a = u1 * site0.phi.cos - u2 * site1.phi.cos;
        let b = u1 * site0.phi.sin - u2 * site1.phi.sin;
        let c = (site0.theta.cos - site1.theta.cos) * site2.theta.sin;
        let length = (a * a + b * b).sqrt();
        let gamma = a.atan2(b);
        let phi_plus_gamma = (c / length).asin();
        BeachLine::wrap(phi_plus_gamma - gamma - site2.phi.value)
    }

    fn wrap(mut phi: f64) -> f64 {
        phi *= 0.5 * FRAC_1_PI;
        phi -= phi.floor();
        phi * 2.0 * PI
    }

    pub fn insert_after(&mut self, prev: Arc, arc: Arc) {
        let next = self.next(prev);
        self.arcs[arc].prev = prev;
        self.arcs[arc].next = next;
        self.arcs[prev].next = arc;
        self.arcs[next].prev = arc;
        self.len += 1;
    }
    
    pub fn neighbors(&self, arc: Arc) -> (Arc, Arc) {
        let data = &self.arcs[arc];
        (data.prev, data.next)
    }

    pub fn remove(&mut self, arc: Arc) {
        let head = self.head;
        if arc == head {
            let next_skip = self.next_skip(self.head, HEIGHT - 1);
            if next_skip != self.head {
                self.head = next_skip;
            } else {
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
        self.remove_skips(arc);
        let (prev, next) = self.neighbors(arc);
        self.arcs[prev].next = next;
        self.arcs[next].prev = prev;
        self.len -= 1;
        if self.len > 1 {
            assert_eq!(self.height(self.head), HEIGHT);
            assert_ne!(self.levels[HEIGHT - 1], 0);
        }
    }

    fn remove_skips(&mut self, arc: Arc) {
        let height = self.height(arc);
        assert!(self.levels[height - 1] > 0);
        for level in 0..height {
            let (prev_skip, next_skip) = self.skips(arc, level);
            self.set_prev_skip(next_skip, level, prev_skip);
            self.set_next_skip(prev_skip, level, next_skip);
        }
        self.levels[height - 1] -= 1;
    }

    fn height(&self, arc: Arc) -> usize {
        for level in 0..HEIGHT {
            if self.next_skip(arc, level).is_invalid() {
                return level;
            }
        }
        HEIGHT
    }
    
    pub fn cell(&self, arc: Arc) -> Cell {
        self.arcs[arc].cell
    }
    
    pub fn start(&self, arc: Arc) -> Id<Start> {
        self.arcs[arc].start    
    }
    
    pub fn set_start(&mut self, arc: Arc, start: Id<Start>) {
        self.arcs[arc].start = start;
    }
    
    pub fn center(&self, arc: Arc) -> Option<Vector3<f64>> {
        self.arcs[arc].center
    }

    pub fn attach_circle(&mut self, arc: Arc, center: Vector3<f64>) {
        self.arcs[arc].center = Some(center);
    }

    pub fn detach_circle(&mut self, arc: Arc) {
        self.arcs[arc].center = None;
    }


    pub fn prev(&self, arc: Arc) -> Arc {
        self.arcs[arc].prev
    }


    pub fn next(&self, arc: Arc) -> Arc {
        self.arcs[arc].next
    }

    pub fn clear(&mut self) {
        self.arcs.clear();
        self.head = Arc::invalid();
        self.len = 0;
        self.levels = [0; HEIGHT];
    }

    fn insertion_height(&self) -> usize {
        let mut best_height = 1;
        let mut best_ratio = self.levels[0];
        let mut multiplier = 1;
        for level in 0..HEIGHT {
            let ratio = self.levels[level] * multiplier;
            if ratio < best_ratio {
                best_ratio = ratio;
                best_height = level + 1;
            }
            multiplier *= 2;
        }
        best_height
    }
}

impl Default for BeachLine {
    fn default() -> Self {
        BeachLine {
            arcs: Default::default(),
            head: Arc::invalid(),
            len: 0,
            levels: [0; HEIGHT],
        }
    }
}
