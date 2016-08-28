use std::rc::Rc;
use std::cell::{RefCell, Cell};
use std::usize::{self};
use event::CircleEvent;

pub struct BeachArc {
    pub face_id: usize,
    pub start_id: Cell<usize>,
    pub event: RefCell<Option<Rc<CircleEvent>>>,
}

impl BeachArc {
    pub fn new(face_id: usize) -> Self {
        BeachArc { 
            face_id: face_id,
            start_id: Cell::new(usize::MAX),
            event: RefCell::new(None),
        }
    }
}