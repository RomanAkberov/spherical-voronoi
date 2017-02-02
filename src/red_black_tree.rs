use std::ops::{Index, IndexMut};
use ideal::{Id, IdVec};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Color {
    Red,
    Black,
}

pub type Node<T> = Id<NodeData<T>>;

pub struct NodeData<T> {
    value: T,
    next: Node<T>,
    prev: Node<T>,
    parent: Node<T>,
    left: Node<T>,
    right: Node<T>,
    color: Color,
}

pub struct RedBlackTree<T> {
    root: Node<T>,
    nodes: IdVec<NodeData<T>>,
	len: usize,
}

impl<T> Default for RedBlackTree<T> {
    fn default() -> Self {
        RedBlackTree {
			root: Node::invalid(),
            nodes: Default::default(),
			len: 0,
        }
    }
}

impl<T> RedBlackTree<T> {
	pub fn clear(&mut self) {
		self.root = Node::invalid();
		self.nodes.clear();
		self.len = 0;
	}
	
    pub fn root(&self) -> Node<T> {
        self.root
    }
    
    pub fn len(&self) -> usize {
        self.len
    }
    
    pub fn next(&self, node: Node<T>) -> Node<T> {
        self.nodes[node].next
    }
    
    pub fn prev(&self, node: Node<T>) -> Node<T> {
        self.nodes[node].prev
    }

	pub fn neighbors(&self, node: Node<T>) -> (Node<T>, Node<T>) {
		let data = &self.nodes[node];
		(data.prev, data.next)
	}
    
	pub fn set_neighbors(&mut self, node: Node<T>, prev: Node<T>, next: Node<T>) {
		{
			let data = &mut self.nodes[node];
			data.prev = prev;
			data.next = next;
		}
		self.nodes[next].prev = node;
		self.nodes[prev].next = node;
	}

    pub fn left(&self, node: Node<T>) -> Node<T> {
        self.nodes[node].left
    }
    
    pub fn right(&self, node: Node<T>) -> Node<T> {
        self.nodes[node].right
    }
    
    pub fn parent(&self, node: Node<T>) -> Node<T> {
        self.nodes[node].parent
    }
    
    pub fn color(&self, node: Node<T>) -> Color {
        self.nodes[node].color
    }
    
    fn set_next(&mut self, node: Node<T>, value: Node<T>) {
        self.nodes[node].next = value;
    }
    
    fn set_prev(&mut self, node: Node<T>, value: Node<T>) {
        self.nodes[node].prev = value;
    }
    
    fn set_left(&mut self, node: Node<T>, value: Node<T>) {
        self.nodes[node].left = value;
    }
    
    fn set_right(&mut self, node: Node<T>, value: Node<T>) {
        self.nodes[node].right = value;
    }
    
    fn set_parent(&mut self, node: Node<T>, value: Node<T>) {
        self.nodes[node].parent = value;
    }
    
    fn set_color(&mut self, node: Node<T>, value: Color) {
        self.nodes[node].color = value;
    }
    
    fn if_red(&self, node: Node<T>) -> Node<T> {
		if node.is_valid() && self.color(node) == Color::Red {
			node
		} else {
			Node::invalid()
		}
    }
    
    pub fn insert_after(&mut self, node: Node<T>, value: T) -> Node<T> {
		self.len += 1;
	    let new = self.nodes.push(NodeData {
	        value: value,
			next: Node::invalid(),
			prev: Node::invalid(),
			parent: Node::invalid(),
	        left: Node::invalid(),
	        right: Node::invalid(),
	        color: Color::Red,
	    });
	    let mut parent_ = if node.is_valid() {
	        let next = self.next(node);
	        self.set_neighbors(new, node, next);
			let right = self.right(node);
			if right.is_valid() {
		        let node = self.first(right);
		        self.set_left(node, new);
		        node
		    } else {
			    self.set_right(node, new);
			    node
			}
		} else {
		    if self.root.is_valid() {
		        let next = self.first(self.root);
				let prev = self.prev(next);
		        self.set_neighbors(new, prev, next);
		        self.set_left(next, new);
		        next
		    } else {
		        self.root = new;
				self.set_neighbors(new, new, new);
				Node::invalid()
		    }
		};
	    self.set_parent(new, parent_);
	    let mut node = new;
	    while parent_.is_valid() {
			let mut parent = parent_;
	        if self.color(parent) == Color::Black {
	            break;
	        }
	        let grandpa = self.parent(parent);
		    if self.left(grandpa) == parent {
				let uncle = self.if_red(self.right(grandpa));
			    if uncle.is_valid() {
			        self.set_color(parent, Color::Black);
			        self.set_color(uncle, Color::Black);
				    self.set_color(grandpa, Color::Red);
				    node = grandpa;
			    } else {
				    if self.right(parent) == node {
					    self.rotate_left(parent);
					    node = parent;
					    parent = self.parent(node);
				    }
				    self.set_color(parent, Color::Black);
				    self.set_color(grandpa, Color::Red);
				    self.rotate_right(grandpa);
			    }
			} else {
				let uncle = self.if_red(self.left(grandpa));
			    if uncle.is_valid() {
				    self.set_color(parent, Color::Black);
				    self.set_color(uncle, Color::Black);
				    self.set_color(grandpa, Color::Red);
				    node = grandpa;
				} else {
				    if self.left(parent) == node {
					    self.rotate_right(parent);
					    node = parent;
					    parent = self.parent(node);
				    }
    				self.set_color(parent, Color::Black);
    				self.set_color(grandpa, Color::Red);
    				self.rotate_left(grandpa);
				}
			}
		    parent_ = self.parent(node);
	    }
	    let root = self.root;
	    self.set_color(root, Color::Black);
	    new
	}

    pub fn remove(&mut self, node: Node<T>) {
		self.len -= 1;
		let (prev, next) = self.neighbors(node);
	    self.set_prev(next, prev);
		self.set_next(prev, next);
		let parent = self.parent(node);
		let left = self.left(node);
		let right = self.right(node);
		let node_color = self.color(node);
		let next = if left.is_valid() {
			if right.is_valid() {
		        self.first(right)
		    } else {
		        left
		    }
		} else {
		    right
		};
		if parent.is_valid() {
		    if self.left(parent) == node {
		        self.set_left(parent, next);
		    } else {
		        self.set_right(parent, next);
		    }
		} else {
		    self.root = next;
		}
        let node_parent = parent;
		let color;
		let (node, parent) = if left.is_valid() && right.is_valid() {
		    color = self.color(next);
		    self.set_color(next, node_color);
		    self.set_left(next, left);
		    self.set_parent(left, next);
		    if next != right {
    			let parent = self.parent(next);
    			self.set_parent(next, node_parent);
    			let node = self.right(next);
    			self.set_left(parent, node);
    			self.set_right(next, right);
    			self.set_parent(right, next);
    			(node, parent)
    		} else {
    			self.set_parent(next, parent);
    			(self.right(next), next)
    		}
	    } else {
	        color = node_color;
	        (next, parent)
        };
    	if node.is_valid() {
    	    self.set_parent(node, parent);
    	}
    	if color == Color::Red {
    	    return;
    	}
		let red = self.if_red(node);
		if red.is_valid() {
    	    self.set_color(node, Color::Black);
    	    return;
    	}
    	let mut parent_ = parent;
    	let mut node = node;
        loop {
			if node == self.root {
				break;
			}
			let parent = parent_;
    		let mut sibling: Node<T>;
		    if node == self.left(parent) {
			    sibling = self.right(parent);
			    if self.color(sibling) == Color::Red {
				    self.set_color(sibling, Color::Black);
				    self.set_color(parent, Color::Red);
				    self.rotate_left(parent);
					sibling = self.right(parent);
				}
    			let left = self.if_red(self.left(sibling));
    			let right = self.if_red(self.right(sibling));
				if left.is_valid() || right.is_valid() { 
    				if left.is_valid() {
    				    self.set_color(left, Color::Black);
    				    self.set_color(sibling, Color::Red);
    					self.rotate_right(sibling);
    					sibling = self.right(parent);
    				}
    				let parent_color = self.color(parent);
    				self.set_color(sibling, parent_color);
    				self.set_color(parent, Color::Black);
    				let right = self.right(sibling);
    				self.set_color(right, Color::Black);
    				self.rotate_left(parent);
    				node = self.root;
    				break;
    			}
		    } else {
    			sibling = self.left(parent);
    			if self.color(sibling) == Color::Red {
				    self.set_color(sibling, Color::Black);
				    self.set_color(parent, Color::Red);
				    self.rotate_right(parent);
					sibling = self.left(parent);
				}
    			let left = self.if_red(self.left(sibling));
    			let right = self.if_red(self.right(sibling));
    			if left.is_valid() || right.is_valid() {  
					if right.is_valid() {
    				    self.set_color(right, Color::Black);
    				    self.set_color(sibling, Color::Red);
    				    self.rotate_left(sibling);
    				    sibling = self.left(parent);
    				}
    				let parent_color = self.color(parent);
    				self.set_color(sibling, parent_color);
    				self.set_color(parent, Color::Black);
    				let left = self.left(sibling);
    				self.set_color(left, Color::Black);
    				self.rotate_right(parent);
    				node = self.root;
    				break;
    			}
		    }
		    self.set_color(sibling, Color::Red);
		    node = parent;
		    parent_ = self.parent(parent);
	        if self.if_red(node).is_valid() {
    	        break;
    	    }
    	}
	    if node.is_valid() {
	        self.set_color(node, Color::Black);
	    }
    }
    
    pub fn first(&self, mut node: Node<T>) -> Node<T> {
		loop {
			let left = self.left(node);
			if left.is_valid() {
				node = left;
			} else {
				return node;
			}
		}
    }
    
    pub fn last(&self, mut node: Node<T>) -> Node<T> {
		loop {
			let right = self.right(node);
			if right.is_valid() {
				node = right;
			} else {
				return node;
			}
		}
    }

    fn rotate_left(&mut self, node: Node<T>) {
	    let child = self.right(node); //can't be None
	    let parent = self.parent(node);
	    if parent.is_valid() {
		    if self.left(parent) == node {
			    self.set_left(parent, child);
		    } else {
			    self.set_right(parent, child);
		    }
		} else {
		    self.root = child;
	    }
	    self.set_parent(child, parent);
	    self.set_parent(node, child);
	    let left = self.left(child);
	    self.set_right(node, left);
		if left.is_valid() {
		    self.set_parent(left, node);
	    }
	    self.set_left(child, node);
    }
    
    fn rotate_right(&mut self, node: Node<T>) {
	    let child = self.left(node); //can't be None
	    let parent = self.parent(node);
		if parent.is_valid() {
		    if self.left(parent) == node {
			    self.set_left(parent, child);
		    } else {
			    self.set_right(parent, child);
		    }
		} else {
		    self.root = child;
	    }
	    self.set_parent(child, parent);
	    self.set_parent(node, child);
	    let right = self.right(child);
	    self.set_left(node, right);
	    if right.is_valid() {
		    self.set_parent(right, node)
	    }
	    self.set_right(child, node);
    }
}

impl<T> Index<Node<T>> for RedBlackTree<T> {
    type Output = T;

    fn index(&self, index: Node<T>) -> &Self::Output {
        &self.nodes[index].value
    }
}

impl<T> IndexMut<Node<T>> for RedBlackTree<T> {
    fn index_mut(&mut self, index: Node<T>) -> &mut Self::Output {
        &mut self.nodes[index].value
    }
}
