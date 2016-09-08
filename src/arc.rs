use std::rc::Rc;
use std::cell::{RefCell, Cell};
use event::CircleEvent;

pub struct Arc {
    pub face_id: usize,
    pub start_id: Cell<Option<usize>>,
    pub event: RefCell<Option<Rc<CircleEvent>>>,
}

impl Arc {
    pub fn new(face_id: usize) -> Self {
        Arc { 
            face_id: face_id,
            start_id: Cell::new(None),
            event: RefCell::new(None),
        }
    }
}