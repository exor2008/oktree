//! [`Node`] implementation.

use crate::{
    bounding::{Aabb, TUVec3, Unsigned},
    pool::Pool,
    ElementId, NodeId,
};
use core::fmt;

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
        Branch { children }
    }

    #[inline(always)]
    pub fn x0_y0_z0(&self) -> NodeId {
        self.children[0]
    }

    #[inline(always)]
    pub fn x1_y0_z0(&self) -> NodeId {
        self.children[1]
    }

    #[inline(always)]
    pub fn x0_y1_z0(&self) -> NodeId {
        self.children[2]
    }

    #[inline(always)]
    pub fn x1_y1_z0(&self) -> NodeId {
        self.children[3]
    }

    #[inline(always)]
    pub fn x0_y0_z1(&self) -> NodeId {
        self.children[4]
    }

    #[inline(always)]
    pub fn x1_y0_z1(&self) -> NodeId {
        self.children[5]
    }

    #[inline(always)]
    pub fn x0_y1_z1(&self) -> NodeId {
        self.children[6]
    }

    #[inline(always)]
    pub fn x1_y1_z1(&self) -> NodeId {
        self.children[7]
    }

    #[inline]
    pub fn center<U: Unsigned>(&self, nodes: &Pool<Node<U>>) -> TUVec3<U> {
        let node = nodes[self.x0_y0_z0()];
        node.aabb.max
    }

    #[inline]
    pub(crate) fn walk_children_inclusive<U: Unsigned>(
        &self,
        nodes: &Pool<Node<U>>,
        aabb: &Aabb<U>,
        mut f: impl FnMut(NodeId),
    ) {
        let branch_center = self.center(nodes);
        if aabb.min.x <= branch_center.x {
            if aabb.min.y <= branch_center.y {
                if aabb.min.z <= branch_center.z {
                    f(self.x0_y0_z0());
                }
                if aabb.max.z >= branch_center.z {
                    f(self.x0_y0_z1());
                }
            }
            if aabb.max.y >= branch_center.y {
                if aabb.min.z <= branch_center.z {
                    f(self.x0_y1_z0());
                }
                if aabb.max.z >= branch_center.z {
                    f(self.x0_y1_z1());
                }
            }
        }
        if aabb.max.x >= branch_center.x {
            if aabb.min.y <= branch_center.y {
                if aabb.min.z <= branch_center.z {
                    f(self.x1_y0_z0());
                }
                if aabb.max.z >= branch_center.z {
                    f(self.x1_y0_z1());
                }
            }
            if aabb.max.y >= branch_center.y {
                if aabb.min.z <= branch_center.z {
                    f(self.x1_y1_z0());
                }
                if aabb.max.z >= branch_center.z {
                    f(self.x1_y1_z1());
                }
            }
        }
    }

    #[inline]
    pub(crate) fn walk_children_exclusive<U: Unsigned>(
        &self,
        nodes: &Pool<Node<U>>,
        aabb: &Aabb<U>,
        mut f: impl FnMut(NodeId),
    ) {
        let branch_center = self.center(nodes);
        if aabb.min.x < branch_center.x {
            if aabb.min.y < branch_center.y {
                if aabb.min.z < branch_center.z {
                    f(self.x0_y0_z0());
                }
                if aabb.max.z > branch_center.z {
                    f(self.x0_y0_z1());
                }
            }
            if aabb.max.y > branch_center.y {
                if aabb.min.z < branch_center.z {
                    f(self.x0_y1_z0());
                }
                if aabb.max.z > branch_center.z {
                    f(self.x0_y1_z1());
                }
            }
        }
        if aabb.max.x > branch_center.x {
            if aabb.min.y < branch_center.y {
                if aabb.min.z < branch_center.z {
                    f(self.x1_y0_z0());
                }
                if aabb.max.z > branch_center.z {
                    f(self.x1_y0_z1());
                }
            }
            if aabb.max.y > branch_center.y {
                if aabb.min.z < branch_center.z {
                    f(self.x1_y1_z0());
                }
                if aabb.max.z > branch_center.z {
                    f(self.x1_y1_z1());
                }
            }
        }
    }

    /// Search which octant is suitable for the position.
    ///
    /// * `position`: Element's position
    /// * `center`: center of the current node's [`Aabb`]
    #[inline(always)]
    pub fn find_child<U: Unsigned>(&self, position: &TUVec3<U>, center: TUVec3<U>) -> NodeId {
        let x = if position.x < center.x { 0 } else { 1 };
        let y = if position.y < center.y { 0 } else { 1 };
        let z = if position.z < center.z { 0 } else { 1 };

        let idx = x | y << 1 | z << 2;

        self.children[idx]
    }
}
