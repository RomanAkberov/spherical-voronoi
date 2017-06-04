#[derive(Copy, Clone)]
pub struct Vertex(pub usize);

#[derive(Copy, Clone)]
pub struct Cell(pub usize);

pub type Point = ::cgmath::Vector3<f64>;