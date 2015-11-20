use redox::Vec;

pub struct AvlNode<T> {
    value: T,
    left: Option<AvlNodeId>, // ID for left node
    right: Option<AvlNodeId>, // ID for right node
}

#[derive(Copy, Clone)]
pub struct AvlNodeId {
    index: usize,
    time_stamp: u64,
}

impl AvlNodeId {
    pub fn get<'a, T: PartialOrd>(&self, avl: &'a Avl<T>) -> &'a AvlNode<T> {
        let ref slot = avl.nodes[self.index];
        if slot.time_stamp == self.time_stamp {
            slot.node.as_ref().unwrap()
        } else {
            panic!("AvlNodeId had invalid time_stamp");
        }
    }

    pub fn try_get<'a, T: PartialOrd>(&self, avl: &'a Avl<T>) -> Option<&'a AvlNode<T>> {
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

    pub fn get_mut<'a, T: PartialOrd>(&self, avl: &'a mut Avl<T>) -> &'a mut AvlNode<T> {
        let ref mut slot = avl.nodes[self.index];
        if slot.time_stamp == self.time_stamp {
            slot.node.as_mut().unwrap()
        } else {
            panic!("AvlNodeId had invalid time_stamp");
        }
    }

    pub fn try_get_mut<'a, T: PartialOrd>(&self,
                                          avl: &'a mut Avl<T>)
                                          -> Option<&'a mut AvlNode<T>> {
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

pub struct Avl<T: PartialOrd> {
    root: usize, // Index of the root node
    nodes: Vec<AvlSlot<T>>,
    free_list: Vec<usize>,
}

impl<T: PartialOrd> Avl<T> {
    pub fn new() -> Self {
        Avl {
            root: 0,
            nodes: Vec::new(),
            free_list: Vec::new(),
        }
    }

    // Inserts a value into the tree, keeping it balanced. Lesser values will be stored on
    // the left, while greater values will be stored on the right. No duplicates are allowed.
    fn insert(&mut self, value: T, node_index: Option<AvlNodeId>) -> AvlNodeId {
        let node = match node_index {
            Some(node) => {
                // Node exists, check which way to branch.
                if value == node.get(self).value {
                    return node;
                } else if value < node.get(self).value {
                    let l = node.get(self).left;
                    node.get_mut(self).left = Some(self.insert(value, l));
                } else if value > node.get(self).value {
                    let r = node.get(self).right;
                    node.get_mut(self).right = Some(self.insert(value, r));
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


    // Performs a left rotation on a tree/subtree.
    // Returns the replace the specified node with
    fn rotate_left(&mut self, node: AvlNodeId) -> AvlNodeId {
        // Keep track of the original node positions
        // For a rotate left, the right child node must exist
        let r = node.get(self).right.unwrap();
        let rl = r.get(self).left;

        let ret = r;
        node.get_mut(self).right = rl;
        ret.get_mut(self).left = Some(node);

        ret
    }

    // Performs a right rotation on a tree/subtree.
    // Returns the replace the specified node with
    fn rotate_right(&mut self, node: AvlNodeId) -> AvlNodeId {
        // Keep track of the original node positions
        // For a rotate right, the left child node must exist
        let l = node.get(self).left.unwrap();
        let lr = l.get(self).right;

        let ret = l;
        node.get_mut(self).left = lr;
        ret.get_mut(self).right = Some(node);

        ret
    }

    // performs a left-right double rotation on a tree/subtree.
    fn rotate_leftright(&mut self, node: AvlNodeId) -> AvlNodeId {
        let l = node.get(self).left.unwrap();
        let new_l = self.rotate_left(l); // Left node needs to exist
        node.get_mut(self).left = Some(new_l);
        self.rotate_right(node)
    }

    // performs a right-left double rotation on a tree/subtree.
    fn rotate_rightleft(&mut self, node: AvlNodeId) -> AvlNodeId {
        let r = node.get(self).right.unwrap();
        let new_r = self.rotate_right(r); // Right node needs to exist
        node.get_mut(self).right = Some(new_r);
        self.rotate_left(node)
    }

    // _rebalance rebalances the provided node
    fn rebalance(&mut self, node: AvlNodeId) -> AvlNodeId {
        let balance = self.height(node.get(self).left) - self.height(node.get(self).right);
        if balance == 2 {
            // left
            let lbalance = self.height(node.get(self).left.unwrap().get(self).left) -
                           self.height(node.get(self).left.unwrap().get(self).right);
            if lbalance == 0 || lbalance == 1 {
                // left left - need to rotate right
                return self.rotate_right(node);
            } else if lbalance == -1 {
                // left right
                return self.rotate_leftright(node); // function name is just a coincidence
            }
        } else if balance == -2 {
            // right
            let rbalance = self.height(node.get(self).right.unwrap().get(self).left) -
                           self.height(node.get(self).right.unwrap().get(self).right);
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
    fn height(&self, node: Option<AvlNodeId>) -> i64 {
        match node {
            Some(node) => {
                let left_height = self.height(node.get(self).left);
                let right_height = self.height(node.get(self).right);

                if left_height > right_height {
                    left_height + 1
                } else {
                    right_height + 1
                }
            }
            None => {
                -1
            }
        }
    }

    fn allocate_node(&mut self, value: T) -> AvlNodeId {
        match self.free_list.pop() {
            Some(index) => {
                AvlNodeId {
                    time_stamp: self.nodes[index].time_stamp + 1,
                    index: index,
                }
            }
            None => {
                // No free slots, create a new one
                let id = AvlNodeId {
                    index: self.nodes.len(),
                    time_stamp: 0,
                };
                self.nodes.push(AvlSlot {
                    time_stamp: 0,
                    node: Some(AvlNode {
                        value: value,
                        left: None,
                        right: None,
                    }),
                });
                id
            }
        }
    }

    fn free_node(&mut self, id: AvlNodeId) -> AvlNode<T> {
        self.free_list.push(id.index);

        // NOTE: We unwrap here, because we trust that `id` points to a valid node, because
        // only we can create and free AvlNodes and their AvlNodeIds
        self.nodes[id.index].node.take().unwrap()
    }
}

struct AvlSlot<T> {
    time_stamp: u64,
    node: Option<AvlNode<T>>,
}
