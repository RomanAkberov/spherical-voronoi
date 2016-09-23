use std::ops::{Index, IndexMut};
use std::marker::PhantomData;
use std::fmt;
use std::usize::{self};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::{Iter, Keys};
use std::iter::Cloned;

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

impl<T> Hash for Id<T> {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.index.hash(hasher);
    }
}

pub struct Pool<T> {
    items: HashMap<Id<T>, T>,
    next_id: Id<T>,
}

pub type Ids<'a, T> = Cloned<Keys<'a, Id<T>, T>>;
pub type IterMut<'a, T> = ::std::collections::hash_map::IterMut<'a, Id<T>, T>;

impl<T> Pool<T> {
    pub fn new() -> Self {
        Pool { 
            items: HashMap::new(),
            next_id: Id::new(0),
        }
    }
    
    pub fn add(&mut self, item: T) -> Id<T> {
        let id = self.next_id;
        self.items.insert(id, item);
        self.next_id = Id::new(id.index + 1);
        id
    }
    
    pub fn remove(&mut self, id: Id<T>) {
        self.items.remove(&id);
    }
    
    pub fn ids(&self) -> Ids<T> {
        self.items.keys().cloned()
    }
    
    pub fn iter_mut(&mut self) -> IterMut<T> {
        self.items.iter_mut()
    }
}

impl<T> Index<Id<T>> for Pool<T> {
    type Output = T;
    
    fn index(&self, index: Id<T>) -> &Self::Output {
        &self.items[&index]
    }
}

impl<T> IndexMut<Id<T>> for Pool<T> {
    fn index_mut(&mut self, index: Id<T>) -> &mut Self::Output {
        self.items.get_mut(&index).unwrap()
    }
}
