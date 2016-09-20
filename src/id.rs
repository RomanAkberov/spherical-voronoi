use std::ops::{Index, IndexMut};
use std::marker::PhantomData;
use std::fmt;
use std::iter::Map;
use std::ops::Range;
use std::usize::{self};
use std::cmp::Ordering;

pub struct Id<T> {
    index: usize,
    marker: PhantomData<T>,
}

impl<T> Id<T> {
    pub fn new(index: usize) -> Self {
        Id { index: index, marker: PhantomData::default() }
    }
    
    pub fn none() -> Self {
        Self::new(usize::MAX)
    }
    
    pub fn index(&self) -> usize {
        self.index
    }
}

impl<T> Copy for Id<T> {}
impl<T> Eq for Id<T> {}

impl<T> Clone for Id<T> {
    fn clone(&self) -> Self { *self }
}

impl<T> PartialEq for Id<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl<T> fmt::Debug for Id<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Id[{}]", self.index)
    }
}

impl<T> PartialOrd for Id<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.index.partial_cmp(&other.index)
    }
}

impl<T> Ord for Id<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.index.cmp(&other.index)
    }
}

pub struct Pool<T> {
    items: Vec<T>
}

pub type IdIter<T> = Map<Range<usize>, fn(usize) -> Id<T>>;

impl<T> Pool<T> {
    pub fn new() -> Self {
        Pool { items: Vec::new() }
    }
    
    pub fn add(&mut self, item: T) -> Id<T> {
        let id = Id::new(self.items.len());
        self.items.push(item);
        id
    }
    
    pub fn ids(&self) -> IdIter<T> {
        (0..self.items.len()).map(Id::new)
    }
    
    pub fn retain<F: FnMut(&T) -> bool>(&mut self, f: F) {
        self.items.retain(f);
    }
}

impl<T> Index<Id<T>> for Pool<T> {
    type Output = T;
    
    fn index(&self, index: Id<T>) -> &Self::Output {
        &self.items[index.index]
    }
}

impl<T> IndexMut<Id<T>> for Pool<T> {
    fn index_mut(&mut self, index: Id<T>) -> &mut Self::Output {
        &mut self.items[index.index]
    }
}
