use std::ops::{Index, IndexMut};
use std::marker::PhantomData;
use std::fmt;
use std::usize::{self};
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::Keys;
use std::iter::Cloned;
use fnv::FnvHashMap;

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

pub struct IdHash<T> {
    items: FnvHashMap<Id<T>, T>,
    next_id: Id<T>,
}

pub type Ids<'a, T> = Cloned<Keys<'a, Id<T>, T>>;
pub type IterMut<'a, T> = ::std::collections::hash_map::IterMut<'a, Id<T>, T>;
pub type Iter<'a, T> = ::std::collections::hash_map::Iter<'a, Id<T>, T>;

impl<T> IdHash<T> {
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
    
    pub fn iter(&self) -> Iter<T> {
        self.items.iter()
    }
}

impl<T> Index<Id<T>> for IdHash<T> {
    type Output = T;
    
    fn index(&self, index: Id<T>) -> &Self::Output {
        &self.items[&index]
    }
}

impl<T> IndexMut<Id<T>> for IdHash<T> {
    fn index_mut(&mut self, index: Id<T>) -> &mut Self::Output {
        self.items.get_mut(&index).unwrap()
    }
}

impl<T> Default for IdHash<T> {
    fn default() -> Self {
        IdHash {
            items: FnvHashMap::default(),
            next_id: Id::new(0),
        }
    }
}

pub struct IdVec<T> {
    items: Vec<T>,
}

pub type IdVecIter<T> = ::std::iter::Map<::std::ops::Range<usize>, fn(usize) -> Id<T>>;

impl<T> IdVec<T> {
    pub fn add(&mut self, item: T) -> Id<T> {
        let id = Id::new(self.items.len());
        self.items.push(item);
        id
    }
    
    pub fn remove(&mut self, id: Id<T>) -> Id<T> {
        let prev_id = Id::new(self.items.len() - 1);
        self.items.swap_remove(id.index);
        prev_id
    }
    
    pub fn retain<F: FnMut(&T) -> bool>(&mut self, f: F) {
        self.items.retain(f);
    }
    
    pub fn ids(&self) -> IdVecIter<T> {
        (0..self.items.len()).map(Id::new)
    }
    
    pub fn len(&self) -> usize {
        self.items.len()
    }
}

impl<T> Index<Id<T>> for IdVec<T> {
    type Output = T;
    
    fn index(&self, index: Id<T>) -> &Self::Output {
        &self.items[index.index]
    }
}

impl<T> IndexMut<Id<T>> for IdVec<T> {
    fn index_mut(&mut self, index: Id<T>) -> &mut Self::Output {
        &mut self.items[index.index]
    }
}

impl<T> Default for IdVec<T> {
    fn default() -> Self {
        IdVec {
            items: Vec::new()
        }
    }
}