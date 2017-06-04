use ideal::IdVec;
use builder::{Builder, Item};
use super::Position;

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

pub fn build_relaxed<I: IntoIterator<Item = Position>>(positions: I, relaxations: usize) -> Builder {
    let mut builder = Builder::new(positions);
    for _ in 1 .. relaxations {
        let mut centroids = IdVec::new();
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
        builder = Builder::new(centroids.items().iter().map(Centroid::result));
    }
    builder
}