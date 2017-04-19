extern crate cgmath;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate ideal;
mod event;
mod beach_line;
mod builder;

pub use builder::{Builder, Item};
pub type Position = ::cgmath::Vector3<f64>;

struct Centroid {
    sum: Position,
    count: f64,
}

impl Centroid {
    fn new() -> Self {
        Self { sum: Position::new(0.0, 0.0, 0.0), count: 0.0 }
    }

    fn add(&mut self, position: Position) {
        self.sum += position;
        self.count += 1.0;
    }

    fn result(&self) -> Position {
        self.sum / self.count
    }
}

pub fn build<I: IntoIterator<Item = Position>>(positions: I, relaxations: usize) -> Builder {
    let mut builder = Builder::new(positions);
    for _ in 1 .. relaxations {
        let mut centroids = Vec::new();
        for item in builder {
            match item {
                Item::Cell => {
                    centroids.push(Centroid::new());
                },
                Item::Vertex(position, cell0, cell1, cell2) => {
                    centroids[cell0].add(position);
                    centroids[cell1].add(position);
                    centroids[cell2].add(position);
                },
                Item::Edge(_, _) => {},
            }
        }
        builder = Builder::new(centroids.iter().map(Centroid::result));
    }
    builder
}
