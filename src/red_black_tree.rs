use std::ops::{Index, IndexMut};
use id::{Id, IdHash};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Color {
    Red,
    Black,
}

struct NodeData<Node: Id, T> {
    value: T,
    next: Option<Node>,
    prev: Option<Node>,
    parent: Option<Node>,
    left: Option<Node>,
    right: Option<Node>,
    color: Color,
}

pub struct RedBlackTree<Node: Id, T> {
    root: Option<Node>,
    nodes: IdVec<Node, NodeData<Node, T>>,
}

impl<Node: Id, T> Default for RedBlackTree<Node, T> {
    fn default() -> Self {
        RedBlackTree {
            root: None,
            nodes: IdVec::default(),
        }
    }
}

impl<Node: Id, T> RedBlackTree<Node, T> {
    pub fn root(&self) -> Option<Node> {
        self.root
    }
    
    pub fn len(&self) -> usize {
        self.nodes.len()
    }
    
    pub fn next(&self, node: Node) -> Option<Node> {
        self.nodes[node].next
    }
    
    pub fn prev(&self, node: Node) -> Option<Node> {
        self.nodes[node].prev
    }
    
    pub fn left(&self, node: Node) -> Option<Node> {
        self.nodes[node].left
    }
    
    pub fn right(&self, node: Node) -> Option<Node> {
        self.nodes[node].right
    }
    
    pub fn parent(&self, node: Node) -> Option<Node> {
        self.nodes[node].parent
    }
    
    pub fn color(&self, node: Node) -> Color {
        self.nodes[node].color
    }
    
    fn set_next(&mut self, node: Node, value: Option<Node>) {
        self.nodes[node].next = value;
    }
    
    fn set_prev(&mut self, node: Node, value: Option<Node>) {
        self.nodes[node].prev = value;
    }
    
    fn set_left(&mut self, node: Node, value: Option<Node>) {
        self.nodes[node].left = value;
    }
    
    fn set_right(&mut self, node: Node, value: Option<Node>) {
        self.nodes[node].right = value;
    }
    
    fn set_parent(&mut self, node: Node, value: Option<Node>) {
        self.nodes[node].parent = value;
    }
    
    fn set_color(&mut self, node: Node, value: Color) {
        self.nodes[node].color = value;
    }
    
    fn if_red(&self, node: Option<Node>) -> Option<Node> {
        match node {
            Some(node) if self.color(node) == Color::Red => Some(node),
            _ => None,
        }
    }
    
    pub fn insert_after(&mut self, node: Option<Node>, value: T) -> Node {
	    let successor = self.nodes.push(NodeData {
	        value: value,
	        next: None,
	        prev: None,
	        parent: None,
	        left: None,
	        right: None,
	        color: Color::Red,
	    });
	    let mut parent_ = if let Some(node) = node {
	        self.set_prev(successor, Some(node));
	        let next = self.next(node);
	        self.set_next(successor, next);
	        if let Some(next) = next {
	            self.set_prev(next, Some(successor));
            }
            self.set_next(node, Some(successor));
            if let Some(right) = self.right(node) {
		        let node = self.first(right);
		        self.set_left(node, Some(successor));
		        Some(node)
		    } else {
			    self.set_right(node, Some(successor));
			    Some(node)
			}
		} else {
		    if let Some(root) = self.root {
		        let node = self.first(root);
		        self.set_prev(successor, None);
		        self.set_next(successor, Some(node));
		        self.set_prev(node, Some(successor));
		        self.set_left(node, Some(successor));
		        Some(node)
		    } else {
		        self.root = Some(successor);
		        None
		    }
		};
	    self.set_parent(successor, parent_);
	    let mut node = successor;
	    while let Some(mut parent) = parent_ {
	        if self.color(parent) == Color::Black {
	            break;
	        }
	        let grandpa = self.parent(parent).unwrap();
		    if Some(parent) == self.left(grandpa) {
			    if let Some(uncle) = self.if_red(self.right(grandpa)) {
			        self.set_color(parent, Color::Black);
			        self.set_color(uncle, Color::Black);
				    self.set_color(grandpa, Color::Red);
				    node = grandpa;
			    } else {
				    if Some(node) == self.right(parent) {
					    let p = self.rotate_left(parent);
					    node = parent;
					    parent = p;
				    }
				    self.set_color(parent, Color::Black);
				    self.set_color(grandpa, Color::Red);
				    self.rotate_right(grandpa);
			    }
			} else {
			    if let Some(uncle) = self.if_red(self.left(grandpa)) {
				    self.set_color(parent, Color::Black);
				    self.set_color(uncle, Color::Black);
				    self.set_color(grandpa, Color::Red);
				    node = grandpa;
				} else {
				    if Some(node) == self.left(parent) {
					    let p = self.rotate_right(parent);
					    node = parent;
					    parent = p;
				    }
    				self.set_color(parent, Color::Black);
    				self.set_color(grandpa, Color::Red);
    				self.rotate_left(grandpa);
				}
			}
		    parent_ = self.parent(node);
	    }
	    let root = self.root.unwrap();
	    self.set_color(root, Color::Black);
	    successor
	}

    pub fn remove(&mut self, node: Node) {
	    let prev = self.prev(node);
	    let next = self.next(node);
	    if let Some(next) = next {
		    self.set_prev(next, prev);
		}
		if let Some(prev) = prev {
		    self.set_next(prev, next);
		}
		//////println!("{:?}, {:?}", prev, next);
		//self.set_prev(node, None);
		//self.set_next(node, None);
		let parent = self.parent(node);
		let left = self.left(node);
		let right = self.right(node);
		let node_color = self.color(node);
		//self.nodes.remove(node);
		let next = if let Some(left) = left {
		    if let Some(right) = right {
		        Some(self.first(right))
		    } else {
		        Some(left)
		    }
		} else {
		    right
		};
		//////println!("{:?}", next);
		if let Some(parent) = parent {
		    if Some(node) == self.left(parent) {
		        self.set_left(parent, next);
		    } else {
		        self.set_right(parent, next);
		    }
		} else {
		    self.root = next;
		}
		//////println!("{:?}", self.root);
        let node_parent = parent;
		let color;
	    let (node, parent) = if let (Some(left), Some(right)) = (left, right) {
	        let next = next.unwrap();
		    color = self.color(next);
		    self.set_color(next, node_color);
		    self.set_left(next, Some(left));
		    self.set_parent(left, Some(next));
		    if next != right {
    			let parent = self.parent(next).unwrap();
    			self.set_parent(next, node_parent);
    			let node = self.right(next);
    			self.set_left(parent, node);
    			self.set_right(next, Some(right));
    			self.set_parent(right, Some(next));
    			(node, Some(parent))
    		} else {
    			self.set_parent(next, parent);
    			(self.right(next), Some(next))
    		}
	    } else {
	        color = node_color;
	        (next, parent)
        };
		////println!("{:?}, {:?}, {:?}, {:?}", node, parent, self.left(parent.unwrap()), self.right(parent.unwrap()));
    	if let Some(node) = node {
    	    self.set_parent(node, parent);
    	}
    	if color == Color::Red {
    	    return;
    	}
    	if let Some(node) = self.if_red(node) {
    	    self.set_color(node, Color::Black);
    	    return;
    	}
    	let mut parent_ = parent;
    	let mut node = node;
        loop {
			if node == self.root {
				break;
			}
			let parent = parent_.unwrap();
    		let mut sibling: Node;
		    if node == self.left(parent) {
			    sibling = self.right(parent).unwrap();
			    if self.color(sibling) == Color::Red {
				    self.set_color(sibling, Color::Black);
				    self.set_color(parent, Color::Red);
				    self.rotate_left(parent);
					sibling = self.right(parent).unwrap();
				}
    			let left = self.if_red(self.left(sibling));
    			let right = self.if_red(self.right(sibling));
    			if left.is_some() || right.is_some() { 
    				if let Some(left) = left {
    				    self.set_color(left, Color::Black);
    				    self.set_color(sibling, Color::Red);
    					self.rotate_right(sibling);
    					sibling = self.right(parent).unwrap()
    				}
    				let parent_color = self.color(parent);
    				self.set_color(sibling, parent_color);
    				self.set_color(parent, Color::Black);
    				let right = self.right(sibling).unwrap();
    				self.set_color(right, Color::Black);
    				self.rotate_left(parent);
    				node = self.root;
    				break;
    			}
		    } else {
    			sibling = self.left(parent).unwrap();
    			if self.color(sibling) == Color::Red {
				    self.set_color(sibling, Color::Black);
				    self.set_color(parent, Color::Red);
				    self.rotate_right(parent);
					sibling = self.left(parent).unwrap();
				}
    			let left = self.if_red(self.left(sibling));
    			let right = self.if_red(self.right(sibling));
    			if left.is_some() || right.is_some() { 
    				if let Some(right) = right {
    				    self.set_color(right, Color::Black);
    				    self.set_color(sibling, Color::Red);
    				    self.rotate_left(sibling);
    				    sibling = self.left(parent).unwrap();
    				}
    				let parent_color = self.color(parent);
    				self.set_color(sibling, parent_color);
    				self.set_color(parent, Color::Black);
    				let left = self.left(sibling).unwrap();
    				self.set_color(left, Color::Black);
    				self.rotate_right(parent);
    				node = self.root;
    				break;
    			}
		    }
		    self.set_color(sibling, Color::Red);
		    node = Some(parent);
		    parent_ = self.parent(parent);
	        if self.if_red(node).is_some() {
    	        break;
    	    }
    	}
	    if let Some(node) = node {
	        self.set_color(node, Color::Black);
	    }
    }
    
    pub fn first(&self, node: Node) -> Node {
        let mut result = node;
        while let Some(left) = self.left(result) {
            result = left;
        }
        result
    }
    
    pub fn last(&self, node: Node) -> Node {
        let mut result = node;
        while let Some(left) = self.right(result) {
            result = left;
        }
        result
    }

    fn rotate_left(&mut self, node: Node) -> Node {
		////println!("LEFT {:?}", self.nodes[&node].value);
	    let child = self.right(node).unwrap(); //can't be None
	    let parent = self.parent(node);
	    if let Some(parent) = parent {
		    if Some(node) == self.left(parent) {
			    self.set_left(parent, Some(child));
		    } else {
			    self.set_right(parent, Some(child));
		    }
		} else {
		    self.root = Some(child);
	    }
	    self.set_parent(child, parent);
	    self.set_parent(node, Some(child));
	    let left = self.left(child);
	    self.set_right(node, left);
	    if let Some(left) = left {
		    self.set_parent(left, Some(node))
	    }
	    self.set_left(child, Some(node));
	    child
    }
    
    fn rotate_right(&mut self, node: Node) -> Node {
		////println!("RIGHT {:?}", self.nodes[&node].value);
	    let child = self.left(node).unwrap(); //can't be None
	    let parent = self.parent(node);
	    if let Some(parent) = parent {
		    if Some(node) == self.left(parent) {
			    self.set_left(parent, Some(child));
		    } else {
			    self.set_right(parent, Some(child));
		    }
		} else {
		    self.root = Some(child);
	    }
	    self.set_parent(child, parent);
	    self.set_parent(node, Some(child));
	    let right = self.right(child);
	    self.set_left(node, right);
	    if let Some(right) = right {
		    self.set_parent(right, Some(node))
	    }
	    self.set_right(child, Some(node));
	    child
    }
}

impl<Node: Id, T> Index<Node> for RedBlackTree<Node, T> {
    type Output = T;

    fn index(&self, index: Node) -> &Self::Output {
        &self.nodes[index].value
    }
}

impl<Node: Id, T> IndexMut<Node> for RedBlackTree<Node, T> {
    fn index_mut(&mut self, index: Node) -> &mut Self::Output {
        &mut self.nodes[index].value
    }
}


impl<Node: Id, T> RedBlackTree<Node, T> {
	pub fn dump(&self) {
		self.dump_node(self.root, 0);
	}

	fn dump_node(&self, node: Option<Node>, indent: usize) {
		for _ in 0..indent {
			print!("-");
		}
		if let Some(node) = node {
			println!("{:?} - {:?}", node, self.color(node));
			self.dump_node(self.left(node), indent + 1);
			self.dump_node(self.right(node), indent + 1);
		} else {
			println!("Nil");
		}
	}
}

