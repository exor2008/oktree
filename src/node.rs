//! [`Node`] implementation.

use core::fmt;

use crate::{
    bounding::{Aabb, TUVec3, Unsigned},
    pool::Pool,
    ElementId, NodeId,
};

/// [`Octree's`](crate::tree::Octree) node.
///
/// Each node has an [`Aabb`], optional parent node link
/// and can be one of the following types:
/// - [`NodeType::Empty`]. Empty node.
/// - [`NodeType::Leaf`]. Node, containig a single [`ElementId`].
/// - [`NodeType::Branch`]. Node, containig a 8 child nodes.
#[derive(Clone, Copy, Debug)]
pub struct Node<U: Unsigned> {
    pub aabb: Aabb<U>,
    pub ntype: NodeType,
    pub parent: Option<NodeId>,
}

impl<U: Unsigned> Default for Node<U> {
    fn default() -> Self {
        Node {
            aabb: Aabb::<U>::default(),
            ntype: Default::default(),
            parent: Default::default(),
        }
    }
}

impl<U: Unsigned> Node<U> {
    pub(crate) fn from_aabb(aabb: Aabb<U>, parent: Option<NodeId>) -> Self {
        Node {
            aabb,
            parent,
            ..Default::default()
        }
    }
}

/// [`Node`] types.
/// - [`NodeType::Empty`]. Empty node.
/// - [`NodeType::Leaf`]. Node, containig a single [`ElementId`].
/// - [`NodeType::Branch`]. Node, containig a 8 child nodes.
#[derive(Default, Clone, Copy, PartialEq, Debug)]
pub enum NodeType {
    #[default]
    Empty,
    Leaf(ElementId),
    Branch(Branch),
}

impl fmt::Display for NodeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeType::Empty => write!(f, "NodeType: Empty"),
            NodeType::Leaf(e) => write!(f, "NodeType: Leaf({e})"),
            NodeType::Branch(branch) => write!(f, "NodeType: Branch({:?})", branch),
        }
    }
}

/// Branch, containig a link to a 8 child [`nodes`](Node).
///
/// Contained by [`branch`](NodeType::Branch) nodes.
#[derive(Default, Clone, Copy, PartialEq, Debug)]
pub struct Branch {
    pub children: [NodeId; 8],
}

impl Branch {
    pub(crate) fn new(children: [NodeId; 8]) -> Self {
        Branch {
            children,
            ..Default::default()
        }
    }

    pub fn x0_y0_z0(&self) -> NodeId {
        self.children[0]
    }

    pub fn x1_y0_z0(&self) -> NodeId {
        self.children[1]
    }

    pub fn x0_y1_z0(&self) -> NodeId {
        self.children[2]
    }

    pub fn x1_y1_z0(&self) -> NodeId {
        self.children[3]
    }

    pub fn x0_y0_z1(&self) -> NodeId {
        self.children[4]
    }

    pub fn x1_y0_z1(&self) -> NodeId {
        self.children[5]
    }

    pub fn x0_y1_z1(&self) -> NodeId {
        self.children[6]
    }

    pub fn x1_y1_z1(&self) -> NodeId {
        self.children[7]
    }

    pub fn center<U: Unsigned>(&self, nodes: &Pool<Node<U>>) -> TUVec3<U> {
        let node = nodes[self.x0_y0_z0()];
        node.aabb.max
    }

    /// Search which octant is suitable for the position.
    ///
    /// * `position`: Element's position
    /// * `center`: center of the current node's [`Aabb`]
    #[inline(always)]
    pub fn find_child<U: Unsigned>(&self, position: TUVec3<U>, center: TUVec3<U>) -> NodeId {
        let x = if position.x < center.x { 0 } else { 1 };
        let y = if position.y < center.y { 0 } else { 1 };
        let z = if position.z < center.z { 0 } else { 1 };

        let idx = x | y << 1 | z << 2;

        self.children[idx]
    }
}
