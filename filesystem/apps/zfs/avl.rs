use std::rc::Rc;

pub struct Node<T> {
    value: T,
    left: Option<usize>, // ID for left node
    right: Option<usize>, // ID for right node
}

impl<T> Node<T> {
    pub fn value(&self) -> &T {
        &self.value
    }
    pub fn left<K>(&self, tree: &Tree<T, K>) -> Option<NodeId> {
        self.left.map(|l| {
            NodeId {
                index: l,
                time_stamp: tree.nodes[l].time_stamp,
            }
        })
    }
    pub fn right<K>(&self, tree: &Tree<T, K>) -> Option<NodeId> {
        self.right.map(|r| {
            NodeId {
                index: r,
                time_stamp: tree.nodes[r].time_stamp,
            }
        })
    }
}

/// /////////////////////////////////////////////////////////////////////////////////////////////////
#[derive(Copy, Clone)]
pub struct NodeId {
    index: usize,
    time_stamp: u64,
}

impl NodeId {
    pub fn get<'a, T, K>(&self, avl: &'a Tree<T, K>) -> &'a Node<T> {
        let ref slot = avl.nodes[self.index];
        if slot.time_stamp == self.time_stamp {
            slot.node.as_ref().unwrap()
        } else {
            panic!("NodeId had invalid time_stamp");
        }
    }

    pub fn try_get<'a, T, K>(&self, avl: &'a Tree<T, K>) -> Option<&'a Node<T>> {
        avl.nodes
           .get(self.index)
           .and_then(|slot| {
               if slot.time_stamp == self.time_stamp {
                   slot.node.as_ref()
               } else {
                   None
               }
           })
    }

    pub fn get_mut<'a, T, K>(&self, avl: &'a mut Tree<T, K>) -> &'a mut Node<T> {
        let ref mut slot = avl.nodes[self.index];
        if slot.time_stamp == self.time_stamp {
            slot.node.as_mut().unwrap()
        } else {
            panic!("NodeId had invalid time_stamp");
        }
    }

    pub fn try_get_mut<'a, T, K>(&self, avl: &'a mut Tree<T, K>) -> Option<&'a mut Node<T>> {
        avl.nodes
           .get_mut(self.index)
           .and_then(|slot| {
               if slot.time_stamp == self.time_stamp {
                   slot.node.as_mut()
               } else {
                   None
               }
           })
    }
}

/// /////////////////////////////////////////////////////////////////////////////////////////////////

pub struct Tree<T, K> {
    root: Option<usize>, // Index of the root node
    nodes: Vec<Slot<T>>,
    free_list: Vec<usize>,
    key: Rc<Fn(&T) -> K>,
}

impl<T, K: PartialOrd> Tree<T, K> {
    pub fn new(key: Rc<Fn(&T) -> K>) -> Self {
        Tree {
            root: None,
            nodes: Vec::new(),
            free_list: Vec::new(),
            key: key,
        }
    }

    // Inserts a value into the tree, keeping it balanced. Lesser values will be stored on
    // the left, while greater values will be stored on the right. No duplicates are allowed.
    pub fn insert(&mut self, value: T) {
        let root = self.root;
        self.root = Some(self._insert(value, root));
    }

    pub fn in_order<F: Fn(&Node<T>)>(&self, f: F) {
        if let Some(root) = self.root {
            self._in_order(&f, root);
        }
    }

    /// Good ol' binary search. Returns immutable reference
    pub fn find(&self, key: K) -> Option<&T> {
        let root = self.root;
        self._find(key, root)
    }

    /// Good ol' binary search. Returns a mutable reference
    pub fn find_mut(&mut self, key: K) -> Option<&mut T> {
        let root = self.root;
        self._find_mut(key, root)
    }

    // Implementation of insert
    fn _insert(&mut self, value: T, node: Option<usize>) -> usize {
        let node = match node {
            Some(node) => {
                // Node exists, check which way to branch.
                if (self.key)(&value) == (self.key)(&self.node(node).value) {
                    return node;
                } else if (self.key)(&value) < (self.key)(&self.node(node).value) {
                    let l = self.node(node).left;
                    self.node_mut(node).left = Some(self._insert(value, l));
                } else if (self.key)(&value) > (self.key)(&self.node(node).value) {
                    let r = self.node(node).right;
                    self.node_mut(node).right = Some(self._insert(value, r));
                }

                node
            }
            None => {
                // The node doesn't exist, create it here.
                self.allocate_node(value)
            }
        };

        self.rebalance(node)
    }

    pub fn _in_order<F: Fn(&Node<T>)>(&self, f: &F, node: usize) {
        if let Some(l) = self.node(node).left {
            self._in_order(f, l);
        }
        f(self.node(node));
        if let Some(r) = self.node(node).right {
            self._in_order(f, r);
        }
    }

    pub fn _find(&self, key: K, node: Option<usize>) -> Option<&T> {
        node.and_then(|n| {
            if (self.key)(&self.node(n).value) < key {
                let left = self.node(n).left;
                self._find(key, left)
            } else if (self.key)(&self.node(n).value) > key {
                let right = self.node(n).right;
                self._find(key, right)
            } else {
                // Found it!
                Some(&self.node(n).value)
            }
        })
    }

    pub fn _find_mut(&mut self, key: K, node: Option<usize>) -> Option<&mut T> {
        match node {
            Some(n) => {
                if (self.key)(&self.node(n).value) < key {
                    let left = self.node(n).left;
                    self._find_mut(key, left)
                } else if (self.key)(&self.node(n).value) > key {
                    let right = self.node(n).right;
                    self._find_mut(key, right)
                } else {
                    // Found it!
                    Some(&mut self.node_mut(n).value)
                }
            }
            None => None,
        }
    }

    // Performs a left rotation on a tree/subtree.
    // Returns the replace the specified node with
    fn rotate_left(&mut self, node: usize) -> usize {
        // Keep track of the original node positions
        // For a rotate left, the right child node must exist
        let r = self.node(node).right.unwrap();
        let rl = self.node(r).left;

        let ret = r;
        self.node_mut(node).right = rl;
        self.node_mut(ret).left = Some(node);

        ret
    }

    // Performs a right rotation on a tree/subtree.
    // Returns the replace the specified node with
    fn rotate_right(&mut self, node: usize) -> usize {
        // Keep track of the original node positions
        // For a rotate right, the left child node must exist
        let l = self.node(node).left.unwrap();
        let lr = self.node(l).right;

        let ret = l;
        self.node_mut(node).left = lr;
        self.node_mut(ret).right = Some(node);

        ret
    }

    // Performs a left-right double rotation on a tree/subtree.
    fn rotate_leftright(&mut self, node: usize) -> usize {
        let l = self.node(node).left.unwrap();
        let new_l = self.rotate_left(l); // Left node needs to exist
        self.node_mut(node).left = Some(new_l);
        self.rotate_right(node)
    }

    // Performs a right-left double rotation on a tree/subtree.
    fn rotate_rightleft(&mut self, node: usize) -> usize {
        let r = self.node(node).right.unwrap();
        let new_r = self.rotate_right(r); // Right node needs to exist
        self.node_mut(node).right = Some(new_r);
        self.rotate_left(node)
    }

    // Rebalances the provided node and returns the node to replace it with if rotations
    // occur
    fn rebalance(&mut self, node: usize) -> usize {
        let balance = self.height(self.node(node).left) - self.height(self.node(node).right);
        if balance == 2 {
            // left
            let lbalance = self.height(self.node(self.node(node).left.unwrap()).left) -
                           self.height(self.node(self.node(node).left.unwrap()).right);
            if lbalance == 0 || lbalance == 1 {
                // left left - need to rotate right
                return self.rotate_right(node);
            } else if lbalance == -1 {
                // left right
                return self.rotate_leftright(node); // function name is just a coincidence
            }
        } else if balance == -2 {
            // right
            let rbalance = self.height(self.node(self.node(node).right.unwrap()).left) -
                           self.height(self.node(self.node(node).right.unwrap()).right);
            if rbalance == 1 {
                // right left
                return self.rotate_rightleft(node); // function name is just a coincidence
            } else if rbalance == 0 || rbalance == -1 {
                // right right - need to rotate left
                return self.rotate_left(node);
            }
        }

        node
    }

    // height gets the height of a tree or subtree
    fn height(&self, node: Option<usize>) -> i64 {
        match node {
            Some(node) => {
                let left_height = self.height(self.node(node).left);
                let right_height = self.height(self.node(node).right);

                if left_height > right_height {
                    left_height + 1
                } else {
                    right_height + 1
                }
            }
            None => -1,
        }
    }

    fn allocate_node(&mut self, value: T) -> usize {
        match self.free_list.pop() {
            Some(index) => {
                self.nodes[index].time_stamp += 1;
                index
            }
            None => {
                // No free slots, create a new one
                let index = self.nodes.len();
                self.nodes.push(Slot {
                    time_stamp: 0,
                    node: Some(Node {
                        value: value,
                        left: None,
                        right: None,
                    }),
                });
                index
            }
        }
    }

    fn free_node(&mut self, index: usize) -> Node<T> {
        self.free_list.push(index);

        // NOTE: We unwrap here, because we trust that `id` points to a valid node, because
        // only we can create and free Nodes and their NodeIds
        self.nodes[index].node.take().unwrap()
    }

    fn node(&self, index: usize) -> &Node<T> {
        self.nodes[index].node.as_ref().unwrap()
    }

    fn node_mut(&mut self, index: usize) -> &mut Node<T> {
        self.nodes[index].node.as_mut().unwrap()
    }
}

/// /////////////////////////////////////////////////////////////////////////////////////////////////

struct Slot<T> {
    time_stamp: u64,
    node: Option<Node<T>>,
}
