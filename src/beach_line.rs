use std::f64::{MAX as F64_MAX};
use std::usize::{MAX as USIZE_MAX};
use std::ops::{Index, IndexMut};
use cgmath::prelude::*;
use event::CellEvent;
use super::Point;

const HEIGHT: usize = 5;

#[derive(Copy, Clone, Default)]
struct Spherical {
    theta: f64,
    phi: f64,
}

#[derive(Copy, Clone, PartialEq)]
pub struct ArcId(u32);

impl ArcId {
    pub const NONE: Self = ArcId(::std::u32::MAX);
}

impl Default for ArcId {
    fn default() -> Self {
        ArcId::NONE
    }
}

#[derive(Copy, Clone, Default)]
pub struct Link {
    pub prev: ArcId,
    pub next: ArcId,
}

pub struct Arc {
    pub cell_index: usize,
    pub focus: Point,
    pub vertex: Point,
    pub theta: f64,
    cached_intersection: Spherical,
    links: [Link; HEIGHT],
    heap_index: usize,
}

impl Arc {
    #[inline]
    pub fn neighbors(&self) -> Link {
        self.links[0]
    }
    
    #[inline]
    fn next(&self) -> ArcId {
        self.links[0].next
    }

    fn height(&self) -> usize {
        (0 .. HEIGHT).filter(|&level| self.links[level].next == ArcId::NONE).next().unwrap_or(HEIGHT)
    }
}

#[derive(Default)]
pub struct BeachLine {
    arcs: Vec<Arc>,
    free: Vec<ArcId>,
    head: ArcId,
    len: usize,
    levels: [usize; HEIGHT],
    heap: Vec<ArcId>,
}

impl BeachLine {
    pub fn insert(&mut self, cell_index: usize, event: &CellEvent) -> ArcId {
        let arc = self.create_arc(cell_index, event.point);
        if self.len > 1 {
            let mut current = self.head;
            let mut next = ArcId::NONE;
            let mut height = HEIGHT;
            let mut links = [ArcId::NONE; HEIGHT];
            while height > 0 {
                next = self[current].links[height - 1].next;
                if self.intersect(current, event) < self.intersect(next, event) {
                    current = next;
                } else {
                    links[height - 1] = current;
                    height -= 1;
                }
            }
            let twin_index = self[next].cell_index;
            let twin_focus = self[next].focus;
            let twin = self.create_arc(twin_index, twin_focus);
            self.add_links(twin, &mut links);
            self.add_links(arc, &mut links);
        } else {
            if self.len == 0 {
                self.head = arc;
            }
            let head = self.head;
            self.add_links(arc, &mut [head; HEIGHT]);
        }
        arc
    }

    pub fn remove(&mut self, arc: ArcId) {
        let head = self.head;
        if arc == head {
            let mut new_head = self[self.head].links[HEIGHT - 1].next;
            if new_head == self.head {
                new_head = self[self.head].next();
                // promote new_head to HEIGHT
                let height = self[new_head].height();
                self.levels[height - 1] -= 1;
                self.levels[HEIGHT - 1] += 1;
                for level in height .. HEIGHT {
                    let next = self[self.head].links[level].next;
                    self[next].links[level].prev = new_head;
                    self[new_head].links[level] = Link { prev: head, next };
                    self[head].links[level].next = new_head;
                }
            }
            self.head = new_head;
        }
        let height = self[arc].height();
        for level in 0 .. height {
            let link = self[arc].links[level];
            self[link.next].links[level].prev = link.prev;
            self[link.prev].links[level].next = link.next;
        }
        self.len -= 1;
        self.levels[height - 1] -= 1;
        self.free.push(arc);
    }

    pub fn heap_update(&mut self, arc: ArcId) {
        let mut index = self[arc].heap_index;
        let theta = self[arc].theta;
        if index == USIZE_MAX {
            index = self.heap.len();
            self.heap.push(arc);
        }
        while index > 0 {
            let parent_index = (index - 1) / 2;
            let parent = self.heap[parent_index];
            if theta < self[parent].theta {
                self.heap[index] = parent;
                self[parent].heap_index = index;
            } else {
                break;
            }
            index = parent_index;
        }
        self.heap[index] = arc;
        self[arc].heap_index = index;
    }

    pub fn has_vertices(&self) -> bool {
        !self.heap.is_empty()
    }

    pub fn heap_pop(&mut self) -> ArcId {
        let first = self.heap[0];
        let last = self.heap.pop().unwrap();
        let theta = self[last].theta;
        let mut index = 0;
        while 2 * index + 1 < self.heap.len() {
            // left child by default
            let mut child_index = 2 * index + 1;
            let mut child = self.heap[child_index];
            // check right child
            let right_index = child_index + 1;
            if right_index < self.heap.len() {
                let right_child = self.heap[right_index];
                if self[right_child].theta < self[child].theta {
                    child_index = right_index;
                    child = right_child;
                }
            }
            if theta < self[child].theta {
                break;
            }
            self.heap[index] = child;
            self[child].heap_index = index;
            index = child_index;
        }
        if self.heap.len() > 0 {
            self.heap[index] = last;
            self[last].heap_index = index;
        }
        first
    }

    pub fn clear(&mut self) {
        self.head = ArcId::NONE;
        self.len = 0;
        self.levels = [0; HEIGHT];
    }

    pub fn top_theta(&self) -> f64 {
        if let Some(&first) = self.heap.first() {
            self[first].theta
        } else {
            F64_MAX
        }
    }
        
    fn create_arc(&mut self, cell_index: usize, focus: Point) -> ArcId {
        let data = Arc {
            cell_index,
            focus,
            vertex: Point::zero(),
            theta: F64_MAX,
            cached_intersection: Spherical::default(),
            links: Default::default(),
            heap_index: USIZE_MAX,
        };
        if let Some(arc) = self.free.pop() {
            self[arc] = data;
            arc
        } else {
            self.arcs.push(data);
            ArcId(self.arcs.len() as u32 - 1)
        }
    }

    fn intersect(&mut self, arc: ArcId, event: &CellEvent) -> f64 {
        let cached = self[arc].cached_intersection;
        let theta = event.theta;
        if cached.theta >= theta {
            return cached.phi;
        }       
        let point0 = self[arc].focus;
        let point1 = self[self[arc].next()].focus;
        let phi = event.intersect(&point0, &point1);
        self[arc].cached_intersection = Spherical { theta, phi };
        phi
    }

    fn add_links(&mut self, arc: ArcId, links: &mut [ArcId; HEIGHT]) {
        let height = self.insertion_height();
        for level in 0 .. height {
            let prev = links[level];
            let mut next = self[prev].links[level].next;
            if next == ArcId::NONE {
                next = prev;
            }
            self[arc].links[level] = Link { prev, next };
            self[next].links[level].prev = arc;
            self[prev].links[level].next = arc;
            links[level] = arc;
        }
        self.len += 1;
        self.levels[height - 1] += 1;
    }

    fn insertion_height(&self) -> usize {
        if self.len == 0 {
            HEIGHT
        } else {
            1 + (0 .. HEIGHT).min_by_key(|&level| self.levels[level] * (1 << level)).unwrap()
        }
    }
}

impl Index<ArcId> for BeachLine {
    type Output = Arc;

    fn index(&self, arc: ArcId) -> &Self::Output {
        &self.arcs[arc.0 as usize]
    }
}

impl IndexMut<ArcId> for BeachLine {
    fn index_mut(&mut self, arc: ArcId) -> &mut Self::Output {
        &mut self.arcs[arc.0 as usize]
    }
}
