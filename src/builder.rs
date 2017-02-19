use std::collections::BTreeSet;
use cgmath::InnerSpace;
use event::{SiteEvent, CircleEvent};
use beach_line::{BeachLine, Arc};
use generator::Generator;
use ::Position;

#[derive(Default)]
struct Builder<G: Generator> {
    site_index: usize,
    site_events: Vec<SiteEvent>,
    circle_events: BTreeSet<CircleEvent>,
    beach: BeachLine,
    generator: G,
}

impl<G: Generator> Builder<G> {
    fn build<I: IntoIterator<Item = Position>>(mut self, positions: I) -> G::Result {
        self.site_events.extend(positions.into_iter().map(SiteEvent::from));
        self.site_events.sort();
        loop {
            match (self.site_index == self.site_events.len(), self.circle_events.is_empty()) {
                (true, true) => break,
                (true, false) => self.circle_event(),
                (false, true) => self.site_event(),
                (false, false) => {
                    if self.site_events[self.site_index].theta.value <
                       self.circle_events.iter().next().unwrap().theta {
                        self.site_event()
                    } else {
                        self.circle_event()
                    }
                }
            }
        }
        self.generator.result()
    }

    fn site_event(&mut self) {
        let cell = self.generator.cell();
        let theta = self.site_events[self.site_index].theta.value;
        self.site_index += 1;
        let arc = self.beach.insert(cell, &self.site_events);
        let (prev, next) = self.beach.neighbors(arc);
        self.generator.temporary(self.beach.index(arc), self.beach.index(prev));
        if prev != next {
            self.detach_circle(prev);
            self.detach_circle(next);
            self.attach_circle(prev, theta);
            self.attach_circle(next, theta);
        }
    }

    fn circle_event(&mut self) {
        let circle = *self.circle_events.iter().next().unwrap();
        self.circle_events.remove(&circle);
        let arc = circle.arc;
        let theta = self.beach.circle_theta(arc);
        if theta >= 0.0 {
            let (prev, next) = self.beach.neighbors(arc);
            self.beach.detach_circle(arc);
            self.detach_circle(prev);
            self.detach_circle(next);
            let vertex = self.generator.vertex(self.beach.circle_center(arc),
                                               self.beach.cell(prev),
                                               self.beach.cell(arc),
                                               self.beach.cell(next));
            self.generator.edge(self.beach.index(prev), vertex);
            self.generator.edge(self.beach.index(arc), vertex);
            self.beach.remove(arc);
            if self.beach.prev(prev) == next {
                self.generator.edge(self.beach.index(next), vertex);
                self.beach.remove(prev);
                self.beach.remove(next);
            } else {
                if self.attach_circle(prev, theta) {
                    self.generator.start(self.beach.index(prev), vertex);
                }
                self.attach_circle(next, theta);
            }
        }
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
        self.site_events[self.beach.cell(arc).index()].position
    }
}

pub fn build<G: Generator, I: IntoIterator<Item = Position>>(positions: I) -> G::Result {
    Builder::<G>::default().build(positions)
}
