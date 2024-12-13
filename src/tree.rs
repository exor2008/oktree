//! [Octree] implementation

use crate::{
    bounding::{Aabb, TUVec3, Unsigned},
    node::{Branch, Node, NodeType},
    pool::{Pool, PoolElementIterator, PoolIntoIterator, PoolItem, PoolIterator, PoolIteratorMut},
    ElementId, NodeId, TreeError, Volume,
};
use alloc::format;
use alloc::vec::Vec;
use core::{fmt, iter};
use smallvec::SmallVec;

/// Fast implementation of the octree data structure.
///
/// Helps to speed up spatial operations with stored data,
/// such as intersections, ray casting e.t.c
/// All coordinates should be positive and integer ([`Unsigned`](num::Unsigned)),
/// due to applied optimisations.
#[derive(Default, Clone)]
pub struct Octree<U, T>
where
    U: Unsigned,
    T: Volume<U = U>,
{
    /// aabb used for clearing the octree
    aabb: Option<Aabb<U>>,

    /// [`Pool`] of stored elements. Access it by [`ElementId`]
    pub(crate) elements: Pool<T>,

    /// [`Pool`] of tree [`Nodes`](crate::node::Node). Access it by [`NodeId`]
    pub(crate) nodes: Pool<Node<U>>,

    pub(crate) root: NodeId,
}

impl<U, T> Octree<U, T>
where
    U: Unsigned,
    T: Volume<U = U>,
{
    /// Construct a tree from [`Aabb`].
    ///
    /// `aabb` should be positive and it's dimensions should be the power of 2.
    /// The root node will adopt aabb's dimensions.
    pub fn from_aabb(aabb: Aabb<U>) -> Self {
        Octree {
            aabb: Some(aabb),
            elements: Default::default(),
            nodes: Pool::from_aabb(aabb),
            root: Default::default(),
        }
    }

    /// Construct a tree with capacity for it's pools.
    ///
    /// Helps to reduce the amount of the memory reallocations.
    pub fn with_capacity(capacity: usize) -> Self {
        Octree {
            aabb: None,
            elements: Pool::<T>::with_capacity(capacity),
            nodes: Pool::<Node<U>>::with_capacity(capacity),
            root: Default::default(),
        }
    }

    /// Construct a tree from [`Aabb`] and capacity.
    ///
    /// `aabb` should be positive and it's dimensions should be the power of 2.
    /// Helps to reduce the amount of the memory reallocations.
    /// The root node will adopt aabb's dimensions.
    pub fn from_aabb_with_capacity(aabb: Aabb<U>, capacity: usize) -> Self {
        Octree {
            aabb: Some(aabb),
            elements: Pool::<T>::with_capacity(capacity),
            nodes: Pool::<Node<U>>::from_aabb_with_capacity(aabb, capacity),
            root: Default::default(),
        }
    }

    /// Insert an element into a tree.
    ///
    /// Recursively subdivide the space, creating new [`nodes`](crate::node::Node)
    /// Returns inserted element's [`id`](ElementId)
    ///
    /// ```rust
    /// use oktree::prelude::*;
    ///
    /// let mut tree = Octree::from_aabb(Aabb::new(TUVec3::splat(16), 16).unwrap());
    /// let c1 = TUVec3u8::new(1u8, 1, 1);
    /// let c1_id = tree.insert(c1).unwrap();
    ///
    /// assert_eq!(c1_id, ElementId(0))
    /// ```
    pub fn insert(&mut self, elem: T) -> Result<ElementId, TreeError> {
        let volume = elem.volume();
        if self.nodes[self.root].aabb.overlaps(&volume) {
            let element = self.elements.insert(elem);

            let mut insertions: SmallVec<[Insertion<U>; 10]> = SmallVec::new();
            insertions.push(Insertion {
                element,
                node: self.root,
                volume,
            });

            let mut was_inserted = false;
            while let Some(insertion) = insertions.pop() {
                match self._insert(insertion, &mut insertions) {
                    Ok(e) => was_inserted |= e == Some(element),
                    Err(err) => {
                        self.elements.tombstone(element);
                        return Err(err);
                    }
                }
            }

            if !was_inserted {
                self.elements.tombstone(element);
                return Err(TreeError::AlreadyOccupied(format!(
                    "Elements for volume: {} already exists",
                    volume
                )));
            }

            Ok(element)
        } else {
            Err(TreeError::OutOfTreeBounds(format!(
                "{volume} is outside of aabb: min: {} max: {}",
                self.nodes[self.root].aabb.min, self.nodes[self.root].aabb.max,
            )))
        }
    }

    #[inline]
    fn _insert<const C: usize>(
        &mut self,
        insertion: Insertion<U>,
        insertions: &mut SmallVec<[Insertion<U>; C]>,
    ) -> Result<Option<ElementId>, TreeError> {
        let Insertion {
            element,
            node,
            volume,
        } = insertion;

        let n = &mut self.nodes[node];
        match n.ntype {
            NodeType::Empty => {
                n.ntype = NodeType::Leaf(element);
                Ok(Some(element))
            }

            NodeType::Leaf(e) => {
                if n.aabb.unit() {
                    return Ok(None); // ignore
                }

                let e1 = self.elements[e].volume();
                let e2 = self.elements[element].volume();
                if e1.overlaps(&e2) {
                    return Ok(None);
                }

                let children = self.nodes.branch(node);
                let n = &mut self.nodes[node];

                n.ntype = NodeType::Branch(Branch::new(children));
                insertions.push(insertion);
                insertions.push(Insertion {
                    element: e,
                    node,
                    volume: e1,
                });
                Ok(None)
            }

            NodeType::Branch(branch) => {
                branch.walk_children_exclusive(&self.nodes, &volume, |child| {
                    insertions.push(Insertion {
                        element,
                        node: child,
                        volume,
                    });
                });
                Ok(None)
            }
        }
    }

    /// Remove an element(s) from the tree
    ///
    /// Recursively collapse an empty [`nodes`](crate::node::Node).
    /// No memory deallocaton happening.
    /// Element is only marked as removed and could be reused.
    ///
    /// ```rust
    /// use oktree::prelude::*;
    ///
    /// let mut tree = Octree::from_aabb(Aabb::new(TUVec3::splat(16), 16).unwrap());
    /// let c1 = TUVec3u8::new(1u8, 1, 1);
    /// let c1_id = tree.insert(c1).unwrap();
    ///
    /// assert!(tree.remove(c1_id).is_ok());
    /// ```
    pub fn remove(&mut self, elem: ElementId) -> Result<(), TreeError> {
        if let Some(element) = self.get_element(elem) {
            let volume = element.volume();
            if self.nodes[self.root].aabb.overlaps(&volume) {
                let mut removals: SmallVec<[Removal; 16]> = SmallVec::new();
                removals.push(Removal {
                    parent: None,
                    node: self.root,
                });
                while let Some(removal) = removals.pop() {
                    self._remove(elem, volume, removal, &mut removals)?;
                }
                self.elements.tombstone(elem);
                Ok(())
            } else {
                Err(TreeError::OutOfTreeBounds(format!(
                    "{volume} is outside of aabb: min: {} max: {}",
                    self.nodes[self.root].aabb.min, self.nodes[self.root].aabb.max,
                )))
            }
        } else {
            Err(TreeError::ElementNotFound(format!(
                "Element with id: {} not found",
                elem.0
            )))
        }
    }

    #[inline]
    fn _remove(
        &mut self,
        element: ElementId,
        volume: Aabb<U>,
        removal: Removal,
        removals: &mut SmallVec<[Removal; 16]>,
    ) -> Result<(), TreeError> {
        let Removal { parent, node } = removal;

        if self.nodes.is_garbage(node) {
            return Ok(());
        }

        let ntype = self.nodes[node].ntype;
        match ntype {
            NodeType::Empty => Ok(()),

            NodeType::Leaf(e) if e == element => {
                self.nodes[node].ntype = NodeType::Empty;
                if let Some(parent) = parent {
                    self.nodes.maybe_collapse(parent);
                }
                Ok(())
            }

            NodeType::Leaf(_) => Ok(()),

            NodeType::Branch(branch) => {
                branch.walk_children_inclusive(&self.nodes, &volume, |child| {
                    removals.push(Removal {
                        parent: Some(node),
                        node: child,
                    });
                });
                Ok(())
            }
        }
    }

    /// Clear all the elements in the octree and reset it to the initial state.
    ///
    /// The capacity of the octree is preserved and thus the octree can be immediately
    /// reused for new elements without causing any memory reallocations.
    pub fn clear(&mut self) {
        self.elements.clear();
        if let Some(aabb) = self.aabb {
            self.nodes.clear_with_aabb(aabb);
        } else {
            self.nodes.clear();
        }
        self.root = Default::default();
    }

    /// Restores all the garbage elements back to real elements. Effectively
    /// this is a rollback of all the remove operations that happened
    pub fn restore_garbage(&mut self) -> Result<(), TreeError> {
        self.elements.restore_garbage()?;
        self.nodes.restore_garbage()?;
        Ok(())
    }

    /// Search for the element at the [`point`](TUVec3)
    ///
    /// Returns element's [`id`](ElementId) or [`None`] if elements if not found.
    ///
    /// ```rust
    /// use oktree::prelude::*;
    ///
    /// let mut tree = Octree::from_aabb(Aabb::new(TUVec3::splat(16), 16).unwrap());
    /// let c1 = TUVec3u8::new(1u8, 1, 1);
    /// tree.insert(c1).unwrap();
    ///
    /// let c2 = TUVec3u8::new(4, 5, 6);
    /// let eid = tree.insert(c2).unwrap();
    ///
    /// assert_eq!(tree.find(&TUVec3::new(4, 5, 6)), Some(eid));
    /// assert_eq!(tree.find(&TUVec3::new(2, 2, 2)), None);
    /// ```
    pub fn find(&self, point: &TUVec3<U>) -> Option<ElementId> {
        self.rfind(self.root, point)
    }

    fn rfind(&self, mut node: NodeId, point: &TUVec3<U>) -> Option<ElementId> {
        loop {
            let ntype = self.nodes[node].ntype;
            return match ntype {
                NodeType::Empty => None,

                NodeType::Leaf(e) => {
                    if self.elements[e].volume().contains(point) {
                        Some(e)
                    } else {
                        None
                    }
                }

                NodeType::Branch(ref branch) => {
                    node = branch.find_child(point, self.nodes[node].aabb.center());
                    continue;
                }
            };
        }
    }

    /// Returns the element if element exists and not garbaged.
    pub fn get_element(&self, element: ElementId) -> Option<&T> {
        if self.elements.is_garbage(element) {
            None
        } else {
            Some(&self.elements[element])
        }
    }

    /// Returns the element if element exists and not garbaged.
    pub fn get_element_mut(&mut self, element: ElementId) -> Option<&mut T> {
        if self.elements.is_garbage(element) {
            None
        } else {
            Some(&mut self.elements[element])
        }
    }

    /// Returns the element if element exists and not garbaged.
    pub fn get(&self, point: &TUVec3<U>) -> Option<&T> {
        let element = self.find(point)?;
        if self.elements.is_garbage(element) {
            None
        } else {
            Some(&self.elements[element])
        }
    }

    /// Returns the element if element exists and not garbaged.
    pub fn get_mut(&mut self, point: &TUVec3<U>) -> Option<&mut T> {
        let element = self.find(point)?;
        if self.elements.is_garbage(element) {
            None
        } else {
            Some(&mut self.elements[element])
        }
    }

    /// Consumes a tree, converting it into a [`vector`](Vec).
    pub fn to_vec(self) -> Vec<T> {
        self.elements
            .vec
            .into_iter()
            .filter_map(|e| {
                if let PoolItem::Filled(item) = e {
                    Some(item)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Returns the number of actual elements in the tree
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    #[inline(always)]
    /// Is the tree empty
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// Returns an iterator over the elements in the tree.
    pub fn iter(&self) -> PoolIterator<'_, T> {
        self.elements.iter()
    }

    /// Returns an mutable iterator over the elements in the tree.
    pub fn iter_mut(&mut self) -> PoolIteratorMut<'_, T> {
        self.elements.iter_mut()
    }

    /// Returns an iterator over the nodes in the tree.
    pub fn iter_nodes(&self) -> PoolIterator<Node<U>> {
        self.nodes.iter()
    }

    /// Returns an iterator over the elements in the tree.
    pub fn iter_elements(&self) -> PoolElementIterator<'_, T> {
        self.elements.iter_elements()
    }
}

impl<U: Unsigned, T: Volume<U = U>> iter::IntoIterator for Octree<U, T> {
    type Item = T;
    type IntoIter = PoolIntoIterator<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.elements.into_iter()
    }
}

impl<U: Unsigned, T: Volume<U = U>> fmt::Debug for Octree<U, T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Octree")
            .field("elements", &self.elements)
            .field("nodes", &self.nodes)
            .field("root", &self.root)
            .finish()
    }
}

#[derive(Debug)]
struct Insertion<U: Unsigned> {
    element: ElementId,
    node: NodeId,
    volume: Aabb<U>,
}

#[derive(Debug)]
struct Removal {
    parent: Option<NodeId>,
    node: NodeId,
}
