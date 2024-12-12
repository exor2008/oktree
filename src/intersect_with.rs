//! Helper functions with a custom intersection closure.

use crate::{
    bounding::{Aabb, Unsigned},
    node::NodeType,
    tree::Octree,
    ElementId, NodeId, Volume,
};
use alloc::vec::Vec;
use heapless::Vec as HVec;

impl<U, T> Octree<U, T>
where
    U: Unsigned,
    T: Volume<U = U>,
{
    /// Intersect [`Octree`] with a custom intersection closure.
    ///
    /// Returns the [`vector`](Vec) of [`elements`](ElementId),
    /// intersected by volume.
    ///
    /// ```rust
    /// use oktree::prelude::*;
    /// use bevy::prelude::*;
    ///
    /// let mut tree = Octree::from_aabb(Aabb::new(TUVec3::splat(16), 16).unwrap());
    ///
    /// let c1 = TUVec3u8::new(1u8, 1, 1);
    /// let c1_id = tree.insert(c1).unwrap();
    ///
    /// // Bounding box intersection
    /// assert_eq!(tree.intersect_with(|_| true), vec![c1_id]);
    /// ```
    pub fn intersect_with<F>(&self, what: F) -> Vec<ElementId>
    where
        F: Fn(&Aabb<U>) -> bool,
    {
        let mut elements = Vec::with_capacity(10);
        self.rintersect_with(self.root, &what, &mut elements);
        elements
    }

    /// Intersect [`Octree`] with a custom intersection closure reusing a
    /// supplied [`vector`](Vec) rather than allocating a new one.
    ///
    /// Returns the [`vector`](Vec) of [`elements`](ElementId),
    /// intersected by volume.
    ///
    /// ```rust
    /// use oktree::prelude::*;
    /// use bevy::prelude::*;
    ///
    /// let mut tree = Octree::from_aabb(Aabb::new(TUVec3::splat(16), 16).unwrap());
    ///
    /// let c1 = TUVec3u8::new(1u8, 1, 1);
    /// let c1_id = tree.insert(c1).unwrap();
    ///
    /// // Bounding box intersection
    /// let mut elements = Vec::new();
    /// tree.extend_intersect_with(|_| true, &mut elements);
    /// assert_eq!(elements, vec![c1_id]);
    /// ```
    pub fn extend_intersect_with<F>(&self, what: F, elements: &mut Vec<ElementId>)
    where
        F: Fn(&Aabb<U>) -> bool,
    {
        self.rintersect_with(self.root, &what, elements);
    }

    fn rintersect_with<F>(&self, node: NodeId, what: &F, elements: &mut Vec<ElementId>)
    where
        F: Fn(&Aabb<U>) -> bool,
    {
        // We use a heapless stack to loop through the nodes until we complete the intersect however
        // if the stack becomes full then then we fallbackon recursive calls.
        let mut stack = HVec::<_, 32>::new();
        stack.push(node).unwrap();
        while let Some(node) = stack.pop() {
            let n = self.nodes[node];
            match n.ntype {
                NodeType::Empty => (),

                NodeType::Leaf(e) => {
                    let aabb = self.elements[e].volume();
                    if what(&aabb) {
                        elements.push(e);
                    };
                }

                NodeType::Branch(branch) => {
                    if what(&n.aabb) {
                        let mut iter = branch.children.iter();
                        while let Some(child) = iter.next() {
                            // If we can't push to the stack (to be processed on the next loop
                            // iteration) then we fallback to recursive calls.
                            if stack.push(*child).is_err() {
                                self.rintersect_with(*child, what, elements);
                                for child in iter.by_ref() {
                                    self.rintersect_with(*child, what, elements);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Intersect [`Octree`] with a custom intersection closure reusing a
    /// supplied [`vector`](Vec) rather than allocating a new one. Each element
    /// that intersects with the volume is passed to the supplied closure.
    ///
    /// ```rust
    /// use oktree::prelude::*;
    /// use bevy::prelude::*;
    ///
    /// let mut tree = Octree::from_aabb(Aabb::new(TUVec3::splat(16), 16).unwrap());
    ///
    /// let c1 = TUVec3u8::new(1u8, 1, 1);
    /// let c1_id = tree.insert(c1).unwrap();
    ///
    /// let mut elements = Vec::new();
    /// tree.intersect_with_for_each(|_| true, |e| elements.push(e.clone()) );
    /// assert_eq!(elements, vec![c1]);
    /// ```
    pub fn intersect_with_for_each<F, F2>(&self, what: F, mut actor: F2)
    where
        F: Fn(&Aabb<U>) -> bool,
        F2: FnMut(&T),
    {
        self.rintersect_with_for_each(self.root, &what, &mut actor);
    }

    fn rintersect_with_for_each<F, F2>(&self, node: NodeId, what: &F, actor: &mut F2)
    where
        F: Fn(&Aabb<U>) -> bool,
        F2: FnMut(&T),
    {
        // We use a heapless stack to loop through the nodes until we complete the intersect however
        // if the stack becomes full then then we fallbackon recursive calls.
        let mut stack = HVec::<_, 32>::new();
        stack.push(node).unwrap();
        while let Some(node) = stack.pop() {
            let n = self.nodes[node];
            match n.ntype {
                NodeType::Empty => (),

                NodeType::Leaf(e) => {
                    let e = &self.elements[e];
                    let aabb = e.volume();
                    if what(&aabb) {
                        actor(e);
                    };
                }

                NodeType::Branch(branch) => {
                    if what(&n.aabb) {
                        let mut iter = branch.children.iter();
                        while let Some(child) = iter.next() {
                            // If we can't push to the stack (to be processed on the next loop
                            // iteration) then we fallback to recursive calls.
                            if stack.push(*child).is_err() {
                                self.rintersect_with_for_each(*child, what, actor);
                                for child in iter.by_ref() {
                                    self.rintersect_with_for_each(*child, what, actor);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
