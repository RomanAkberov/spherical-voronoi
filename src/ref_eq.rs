use std::rc::Rc;

pub trait RefEq {
    fn ref_eq(&self, other: &Self) -> bool;
}

impl<T> RefEq for Rc<T> {
    fn ref_eq(&self, other: &Self) -> bool {
        &**self as *const T == &**other as *const T
    }  
} 