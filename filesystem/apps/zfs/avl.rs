use redox::Vec;

pub struct AvlNode<T> {
    value: T,
    left: Option<usize>, // Index for right node
    right: Option<usize>, // Index for left node
}

#[derive(Copy, Clone)]
pub struct AvlNodeId {
    index: usize,
    time_stamp: u64,
}

impl AvlNodeId {
    pub fn get<'a, T>(&self, avl: &'a Avl<T>) -> Option<&'a AvlNode<T>> {
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
}

pub struct Avl<T> {
    root: usize, // Index of the root node
    nodes: Vec<AvlSlot<T>>,
    free_list: Vec<usize>,
}

impl<T> Avl<T> {
    pub fn new() -> Self {
        Avl {
            root: 0,
            nodes: Vec::new(),
            free_list: Vec::new(),
        }
    }

    pub fn insert(&mut self, value: T) -> AvlNodeId {
        // TODO this is just a placeholder, we need to deal with all the fancy rotation stuff that
        // AVL trees do
        self.allocate_node(value)
    }

    fn allocate_node(&mut self, value: T) -> AvlNodeId {
        match self.free_list.pop() {
            Some(index) => {
                AvlNodeId { time_stamp: self.nodes[index].time_stamp+1, index: index }
            },
            None => {
                // No free slots, create a new one
                let id = AvlNodeId { index: self.nodes.len(), time_stamp: 0 };
                self.nodes.push(AvlSlot { time_stamp: 0,
                                          node: Some(AvlNode { value: value, left: None, right: None }) });
                id
            },
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
