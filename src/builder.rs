use std::collections::BTreeSet;
use std::iter::FromIterator;
use cgmath::InnerSpace;
use event::{SiteEvent, CircleEvent};
use beach_line::{BeachLine, Arc};
use ::Position;
use ideal::{Id, IdVec};

create_id!(Vertex);
create_id!(Cell);

pub struct Builder {
    next_cell: Cell,
    next_vertex: Vertex,
    site_events: IdVec<Cell, SiteEvent>,
    circle_events: BTreeSet<CircleEvent>,
    beach: BeachLine,
    items: Vec<Item>,
}

impl Builder {
    pub fn new<I: IntoIterator<Item = Position>>(positions: I) -> Self {
        let mut site_events = Vec::from_iter(positions.into_iter().map(SiteEvent::from));
        site_events.sort();
        Builder {
            next_cell: Cell::new(0),
            next_vertex: Vertex::new(0),
            site_events: IdVec::from(site_events),
            circle_events: Default::default(),
            beach: Default::default(),
            items: Default::default(),
        }
    }

    fn site_event(&mut self) -> Item {
        let cell = self.next_cell;
        let theta = self.site_events[self.next_cell].theta.value;
        self.next_cell = self.next_cell.next();
        let arc = self.beach.insert(cell, &self.site_events);
        let (prev, next) = self.beach.neighbors(arc);
        self.beach.temporary(arc, prev);
        if prev != next {
            self.detach_circle(prev);
            self.detach_circle(next);
            self.attach_circle(prev, theta);
            self.attach_circle(next, theta);
        }
        Item::Cell
    }

    fn circle_event(&mut self, circle: &CircleEvent) -> Item {
        self.circle_events.remove(circle);
        let arc = circle.arc;
        let theta = circle.theta;
        let (prev, next) = self.beach.neighbors(arc);
        self.beach.detach_circle(arc);
        self.detach_circle(prev);
        self.detach_circle(next);
        let vertex = self.next_vertex;
        self.next_vertex = self.next_vertex.next();
        let position = self.beach.circle_center(arc);
        let cell0 = self.beach.cell(prev).index();
        let cell1 = self.beach.cell(arc).index();
        let cell2 = self.beach.cell(next).index();    
        self.edge(prev, vertex);
        self.edge(arc, vertex);
        self.beach.remove(arc);
        if self.beach.prev(prev) == next {
            self.edge(next, vertex);
            self.beach.remove(prev);
            self.beach.remove(next);
        } else {
            if self.attach_circle(prev, theta) {
                self.beach.start(prev, vertex);
            }
            self.attach_circle(next, theta);
        }
        Item::Vertex(position, cell0, cell1, cell2)
    }

    fn attach_circle(&mut self, arc: Arc, min: f64) -> bool {
        let (prev, next) = self.beach.neighbors(arc);
        let position = self.arc_position(arc);
        let from_prev = self.arc_position(prev) - position;
        let from_next = self.arc_position(next) - position;
        let center = from_prev.cross(from_next).normalize();
        let theta = center.z.acos() + center.dot(position).acos();
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

    fn arc_position(&self, arc: Arc) -> Position {
        self.site_events[self.beach.cell(arc)].position
    }

    fn edge(&mut self, arc: Arc, end: Vertex) {
        if let Some(start) = self.beach.edge(arc, end) {
            self.items.push(Item::Edge(start.index(), end.index()));
        }
    }
}

pub enum Item {
    Cell,
    Vertex(Position, usize, usize, usize),
    Edge(usize, usize),
}

impl Iterator for Builder {
    type Item = Item;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(item) = self.items.pop() {
            return Some(item);
        }
        let has_sites = self.next_cell.index() < self.site_events.len();
        if let Some(circle) = self.circle_events.iter().next().cloned() {
            if has_sites && self.site_events[self.next_cell].theta.value < circle.theta {
                Some(self.site_event())
            } else {
                Some(self.circle_event(&circle))
            }
        } else if has_sites {
            Some(self.site_event())
        } else {
            None
        }
    }
}
