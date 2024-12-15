//! [![Crates.io](https://img.shields.io/crates/v/oktree.svg)](https://crates.io/crates/oktree)
//! [![Docs.rs](https://docs.rs/oktree/badge.svg)](https://docs.rs/oktree)
//!
//! Fast [`octree`](tree::Octree) implementation.
//!
//! ![Example](https://raw.githubusercontent.com/exor2008/oktree/main/assets/example.gif)
//!
//! Able to operate with [`Position`] or [`Volume`] data.
//!
//! Could be used with the Bevy game engine or as a standalone tree.
//!
//! ## Available methods:
//!
//! - ### Unsigned operations
//!
//!   - [`Insertion`](tree::Octree::insert)
//!   - [`Removing`](tree::Octree::remove)
//!   - [`Searching`](tree::Octree::find)
//!
//! - ### Floating point operations (Bevy integration)
//!
//!   - [`Ray casting`](tree::Octree::ray_cast)
//!   - [`Bouning sphere and bounding box intersection`](tree::Octree::intersect)
//!
//! To enable bevy integrations:
//!
//! ```toml
//! [dependencies]
//! oktree = { version = "0.5.0", features = ["bevy"] }
//! ```
//!
//! Intersection methods are not available without this feature.
//!
//! ## Optimizations:
//!
//! - `Unsigned` arithmetics, bitwise operations.
//! - Tree structure is represented by flat, reusable [`Pool`](`pool::Pool`). Removed data is marked only.
//! - Few memory allocations. [`smallvec`] and [`heapless`] structures are used.
//! - No smart pointers ([`Rc`](`std::rc::Rc`), [`RefCell`](std::cell::RefCell) e.t.c)
//!
//! Compensation for the inconvenience is perfomance.
//!
//! ## Benchmark
//! Octree dimensions: `4096x4096x4096`
//!
//! | Operation           | Quantity                         | Time   |
//! | ------------------- | -------------------------------- | ------ |
//! | insertion           | 65536 cells                      | 21 ms  |
//! | removing            | 65536 cells                      | 1.5 ms |
//! | find                | 65536 searches in 65536 cells    | 12 ms  |
//! | ray intersection    | 4096 rays against 65536 cells    | 37 ms  |
//! | sphere intersection | 4096 spheres against 65536 cells | 8 ms   |
//! | box intersection    | 4096 boxes against 65536 cells   | 7 ms   |
//!
//! Run benchmark:
//!
//! ```sh
//! cargo bench --all-features
//! ```
//!
//! ## Examples
//!
//! ### Simple
//!
//! You have to specify the type for the internal [`Octree`](`tree::Octree`) structure.
//!
//! It must be any [`Unsigned`](`num::Unsigned`) type (`u8`, `u16`, `u32`, `u64`, `u128` or `usize`).
//!
//! Implement [`Position`] or [`Volume`] for the handled type, so that it can return it's spatial coordinates.
//!
//! ```rust
//! use bevy::math::{
//!     bounding::{Aabb3d, BoundingSphere, RayCast3d},
//!     Dir3, Vec3,
//! };
//! use oktree::prelude::*;
//!
//! fn main() -> Result<(), TreeError> {
//!     let aabb = Aabb::new(TUVec3::splat(16), 16u8);
//!     let mut tree = Octree::from_aabb_with_capacity(aabb?, 10);
//!
//!     let c1 = DummyCell::new(TUVec3::splat(1u8));
//!     let c2 = DummyCell::new(TUVec3::splat(8u8));
//!
//!     let c1_id = tree.insert(c1)?;
//!     let c2_id = tree.insert(c2)?;
//!
//!     // Searching by position
//!     assert_eq!(tree.find(&TUVec3::new(1, 1, 1)), Some(c1_id));
//!     assert_eq!(tree.find(&TUVec3::new(8, 8, 8)), Some(c2_id));
//!     assert_eq!(tree.find(&TUVec3::new(1, 2, 8)), None);
//!     assert_eq!(tree.find(&TUVec3::splat(100)), None);
//!
//!     // Searching for the ray intersection
//!     let ray = RayCast3d::new(Vec3::new(1.5, 7.0, 1.9), Dir3::NEG_Y, 100.0);
//!
//!     // Hit!
//!     assert_eq!(
//!         tree.ray_cast(&ray),
//!         HitResult {
//!             element: Some(ElementId(0)),
//!             distance: 5.0
//!         }
//!     );
//!
//!     assert_eq!(tree.remove(ElementId(0)), Ok(()));
//!
//!     // Miss!
//!     assert_eq!(
//!         tree.ray_cast(&ray),
//!         HitResult {
//!             element: None,
//!             distance: 0.0
//!         }
//!     );
//!
//!     let c1 = DummyCell::new(TUVec3::splat(1u8));
//!     let c1_id = tree.insert(c1)?;
//!
//!     // Aabb intersection
//!     let aabb = Aabb3d::new(Vec3::splat(2.0), Vec3::splat(2.0));
//!     assert_eq!(tree.intersect(&aabb), vec![c1_id]);
//!
//!     // Sphere intersection
//!     let sphere = BoundingSphere::new(Vec3::splat(2.0), 2.0);
//!     assert_eq!(tree.intersect(&sphere), vec![c1_id]);
//!     
//!     Ok(())
//! }
//!
//! struct DummyCell {
//!     position: TUVec3<u8>,
//! }
//!
//! impl Position for DummyCell {
//!     type U = u8;
//!     fn position(&self) -> TUVec3<u8> {
//!         self.position
//!     }
//! }
//!
//! impl DummyCell {
//!     fn new(position: TUVec3<u8>) -> Self {
//!         DummyCell { position }
//!     }
//! }
//! ```
//! ### Bevy
//!
//! Run bevy visual example:
//!
//! ```sh
//! cargo run --release --example bevy_tree --all-features
//! ```
//!
//! ### no_std
//!
//! `no_std` is supported, but you steel need to specify a global allocator.
//!
//! See [example](https://github.com/exor2008/oktree/blob/main/examples/no_std.rs) with a custom allocator.
//!
//! Run `no_std` example
//!
//! ```sh
//! cargo run --no-default-features --features bevy --example no_std
//! ```
//!
//! ## Contribution guide
//!
//! Feature and pull requests are welcomed.
//!
//! 1. Clone the Oktree [repo](https://github.com/exor2008/oktree)
//!
//! ```sh
//! git clone https://github.com/exor2008/oktree
//! ```
//!
//! 2. Implement awesome feature
//!
//! 3. Run tests
//!
//! ```sh
//! cargo test --all-targets --all-features --release
//! ```
//!
//! 4. Make sure clippy is happy
//!
//! ```sh
//! cargo clippy --all-targets --all-features
//! ```
//!
//! 5. Run examples
//!
//! ```sh
//! cargo run --all-features --example simple
//! cargo run --all-features --example bevy_tree
//! cargo run --no-default-features --features bevy --example no_std
//! ```
//!
//! 6. Run benchmark
//!
//! ```sh
//! cargo bench --all-features
//! ```
//!
//! 7. Check the docs:
//!
//! ```sh
//! cargo doc --no-deps --open --all-features
//! ```
//!
//! 8. Start pull request
//!

#![allow(dead_code)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "bevy")]
pub mod bevy_integration;
pub mod bounding;
mod entry;
pub mod intersect_with;
pub mod node;
pub mod pool;
pub mod prelude;
pub mod tree;
use alloc::{borrow::Cow, boxed::Box, string::String, sync::Arc};
use bounding::{TUVec3, Unsigned};
use core::{
    error::Error,
    fmt::{self},
    ops::Deref,
};
use prelude::Aabb;

extern crate alloc;
/// Implement to represent your object as a point in a [`tree`](tree::Octree)
///
/// Implement on stored type to inform a tree
/// about object's spatial coordinates. You only need
/// to implement either Volume or Position implemnentations
/// and not both
pub trait Position {
    type U: Unsigned;

    fn position(&self) -> TUVec3<Self::U>;
}

impl<T> Position for Box<T>
where
    T: Position,
{
    type U = T::U;

    fn position(&self) -> TUVec3<Self::U> {
        self.deref().position()
    }
}

/// Implement to represent your object as a volume in a [`tree`](tree::Octree).
///
/// Implement on stored type to inform a tree
/// about object's spatial volume. You only need
/// to implement either Volume or Position implemnentations
/// and not both
pub trait Volume {
    type U: Unsigned;

    fn volume(&self) -> Aabb<Self::U>;
}

impl<U: Unsigned, T> Volume for T
where
    T: Position<U = U>,
{
    type U = U;
    fn volume(&self) -> Aabb<U> {
        self.position().unit_aabb()
    }
}

impl<U: Unsigned, T: Clone> Volume for Cow<'_, T>
where
    T: Position<U = U>,
{
    type U = U;
    fn volume(&self) -> Aabb<U> {
        self.deref().volume()
    }
}

impl<U: Unsigned, T> Volume for Arc<T>
where
    T: Position<U = U>,
{
    type U = U;

    fn volume(&self) -> Aabb<U> {
        self.deref().volume()
    }
}

/// Index [`tree.nodes`](pool::Pool) with it.
///
#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct NodeId(pub u32);

impl From<NodeId> for ElementId {
    fn from(value: NodeId) -> Self {
        ElementId(value.0)
    }
}

impl From<NodeId> for usize {
    fn from(value: NodeId) -> Self {
        value.0 as usize
    }
}

impl From<usize> for NodeId {
    fn from(value: usize) -> Self {
        NodeId(value as u32)
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NodeId {}", self.0)
    }
}

/// Index [`tree.elements`](pool::Pool) with it.
/// Stored type element will be returned.
///
/// ```rust
/// use oktree::prelude::*;
///
/// let mut tree = Octree::from_aabb_with_capacity(Aabb::new(TUVec3::splat(16), 16u16).unwrap(), 10);
/// tree.insert(TUVec3u16::new(5, 5, 5)).unwrap();
/// let element: &TUVec3u16 = tree.get_element(ElementId(0)).unwrap();
/// ```
#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct ElementId(pub u32);

impl From<ElementId> for usize {
    fn from(value: ElementId) -> Self {
        value.0 as usize
    }
}

impl From<usize> for ElementId {
    fn from(value: usize) -> Self {
        ElementId(value as u32)
    }
}

impl fmt::Display for ElementId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ElementId: {}", self.0)
    }
}

/// Enum of all possible errors of the octree's operations.
#[derive(Debug, PartialEq)]
pub enum TreeError {
    /// Object is out of bounds of tree's [`Aabb`].
    OutOfTreeBounds(String),

    /// Attempt to treat a [`Node`](node::Node) of different type
    /// as a [`Branch`](node::NodeType::Branch).
    NotBranch(String),

    /// Attempt to treat a [`Node`](node::Node) of different type
    /// as a [`Leaf`](node::NodeType::Leaf).
    NotLeaf(String),

    /// Only a [`Branch`](node::NodeType::Branch) [`Node`](node::Node) can be collapsed.
    CollapseNonEmpty(String),

    /// [`Aabb`] bounds are not positive.
    NotPositive(String),

    /// [`Aabb`] size should be the power of 2.
    NotPower2(String),

    /// [`ElementId`] is already occupied by an item.
    AlreadyOccupied(String),

    /// [`ElementId`] is not found in an [`tree`](tree::Octree)
    ElementNotFound(String),

    /// [`tree`](tree::Octree)'s garbage is corrupted.
    CorruptGarbage(String),
}

impl Error for TreeError {}

impl fmt::Display for TreeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TreeError::OutOfTreeBounds(info) => write!(f, "Out of tree bounds. {info}"),
            TreeError::NotBranch(info) => write!(f, "Node is not a Branch. {info}"),
            TreeError::NotLeaf(info) => write!(f, "Node is not a Leaf. {info}"),
            TreeError::CollapseNonEmpty(info) => write!(f, "Collapsing non empty branch. {info}"),
            TreeError::NotPositive(info) => {
                write!(f, "All AABB dimensions should be positive. {info}")
            }
            TreeError::NotPower2(info) => {
                write!(f, "All AABB dimensions should be the power of 2. {info}")
            }
            TreeError::AlreadyOccupied(info) => write!(f, "Volume is already occupied. {info}"),
            TreeError::ElementNotFound(info) => write!(f, "Element not found. {info}"),
            TreeError::CorruptGarbage(info) => write!(f, "Tree's garbage is corrupted. {info}"),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use bounding::Aabb;
    use node::NodeType;
    use rand::Rng;
    use std::collections::HashSet;
    use tree::Octree;

    const RANGE: usize = 65536;

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct DummyCell<U: Unsigned> {
        position: TUVec3<U>,
        node: NodeId,
    }

    impl<U: Unsigned> Position for DummyCell<U> {
        type U = U;
        fn position(&self) -> TUVec3<U> {
            self.position
        }
    }

    impl<U: Unsigned> DummyCell<U> {
        fn new(position: TUVec3<U>) -> Self {
            DummyCell {
                position,
                node: Default::default(),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    struct DummyVolume<U: Unsigned> {
        aabb: Aabb<U>,
        node: NodeId,
    }

    impl<U: Unsigned> Volume for DummyVolume<U> {
        type U = U;
        fn volume(&self) -> Aabb<U> {
            self.aabb
        }
    }

    impl<U: Unsigned> DummyVolume<U> {
        fn new(aabb: Aabb<U>) -> Self {
            Self {
                aabb,
                node: Default::default(),
            }
        }
    }

    #[test]
    fn test_insert() {
        let mut tree = Octree::from_aabb(Aabb::new_unchecked(TUVec3::new(4, 4, 4), 4));

        assert_eq!(tree.elements.len(), 0);
        assert_eq!(tree.elements.garbage_len(), 0);

        assert_eq!(tree.nodes.len(), 1);
        assert_eq!(tree.nodes.garbage_len(), 0);

        assert_eq!(tree.nodes[0.into()].ntype, NodeType::Empty);
        assert_eq!(tree.nodes[0.into()].parent, None);

        let c1 = DummyCell::new(TUVec3::new(1u8, 1, 1));
        assert_eq!(tree.insert(c1), Ok(ElementId(0)));

        assert_eq!(tree.elements.len(), 1);
        assert_eq!(tree.elements.garbage_len(), 0);

        assert_eq!(tree.nodes.len(), 1);
        assert_eq!(tree.nodes.garbage_len(), 0);

        assert_eq!(tree.nodes[0.into()].ntype, NodeType::Leaf(0.into()));
        assert_eq!(tree.nodes[0.into()].parent, None);

        let c2 = DummyCell::new(TUVec3::new(7, 7, 7));
        assert_eq!(tree.insert(c2), Ok(ElementId(1)));

        assert_eq!(tree.elements.len(), 2);
        assert_eq!(tree.elements.garbage_len(), 0);

        assert_eq!(tree.nodes.len(), 9);
        assert_eq!(tree.nodes.garbage_len(), 0);

        assert_eq!(tree.nodes[0.into()].parent, None);

        assert_eq!(tree.nodes[1.into()].ntype, NodeType::Leaf(0.into()));
        assert_eq!(tree.nodes[1.into()].parent, Some(0.into()));
        assert_eq!(tree.nodes[8.into()].ntype, NodeType::Leaf(1.into()));
        assert_eq!(tree.nodes[8.into()].parent, Some(0.into()));
        for i in 2..8 {
            assert_eq!(tree.nodes[i.into()].ntype, NodeType::Empty);
        }
    }

    #[test]
    fn test_remove() {
        let mut tree = Octree::from_aabb(Aabb::new_unchecked(TUVec3::new(8u16, 8, 8), 8));

        let c1 = DummyCell::new(TUVec3::new(1, 1, 1));
        assert_eq!(tree.insert(c1), Ok(ElementId(0)));
        let c2 = DummyCell::new(TUVec3::new(2, 2, 2));
        assert_eq!(tree.insert(c2), Ok(ElementId(1)));
        assert_eq!(tree.nodes[17.into()].ntype, NodeType::Leaf(0.into()));

        assert_eq!(tree.nodes.len(), 25);

        let c2r = DummyCell::new(TUVec3::new(1, 1, 1));
        assert!(tree.insert(c2r).is_err());
        assert_eq!(tree.find(&TUVec3::new(1, 1, 1)), Some(ElementId(0)));

        assert_eq!(tree.nodes.len(), 25);
        assert_eq!(tree.elements.len(), 2);

        tree.remove(0.into()).unwrap();

        assert_eq!(tree.elements.len(), 1);
        assert_eq!(tree.nodes.len(), 25);

        tree.remove(1.into()).unwrap();

        assert_eq!(tree.elements.len(), 0);
        assert_eq!(tree.nodes.len(), 1);

        assert_eq!(tree.nodes[0.into()].ntype, NodeType::Empty)
    }

    #[test]
    fn test_insert_remove() {
        let mut tree = Octree::from_aabb(Aabb::new_unchecked(TUVec3::new(4u8, 4, 4), 4));

        let c1 = DummyCell::new(TUVec3::new(1, 1, 1));
        assert_eq!(tree.insert(c1), Ok(ElementId(0)));

        let c2 = DummyCell::new(TUVec3::new(2, 2, 1));
        assert_eq!(tree.insert(c2), Ok(ElementId(1)));

        let c3 = DummyCell::new(TUVec3::new(6, 6, 1));
        assert_eq!(tree.insert(c3), Ok(ElementId(2)));

        let c4 = DummyCell::new(TUVec3::new(7, 7, 1));
        assert_eq!(tree.insert(c4), Ok(ElementId(3)));

        let c5 = DummyCell::new(TUVec3::new(6, 7, 1));
        assert_eq!(tree.insert(c5), Ok(ElementId(4)));

        assert_eq!(tree.remove(0.into()), Ok(()));

        assert_eq!(tree.remove(1.into()), Ok(()));

        assert_eq!(tree.nodes[1.into()].ntype, NodeType::Empty);

        assert_eq!(tree.remove(2.into()), Ok(()));
        assert_eq!(tree.remove(3.into()), Ok(()));

        assert_eq!(tree.nodes[1.into()].ntype, NodeType::Empty);

        assert_eq!(tree.remove(4.into()), Ok(()));

        assert_eq!(tree.nodes[0.into()].ntype, NodeType::Empty);
        assert_eq!(tree.nodes.len(), 1);
        assert_eq!(tree.elements.len(), 0);
    }

    fn random_point() -> DummyCell<usize> {
        let mut rnd = rand::thread_rng();

        let x = rnd.gen_range(0..=RANGE);
        let y = rnd.gen_range(0..=RANGE);
        let z = rnd.gen_range(0..=RANGE);
        let position = TUVec3::new(x, y, z);
        DummyCell::new(position)
    }

    #[test]
    fn test_65536() {
        let mut tree = Octree::from_aabb(Aabb::new_unchecked(TUVec3::splat(RANGE / 2), RANGE / 2));

        for _ in 0..RANGE {
            let p = random_point();
            let pos = p.position;
            if let Ok(e) = tree.insert(p) {
                assert_eq!(tree.find(&pos), Some(e));
            }
        }

        assert!(tree.elements.len() > (RANGE as f32 * 0.98) as usize);

        for element in 0..tree.len() {
            let e = ElementId(element as u32);
            let pos = tree.elements[e].position;
            assert_eq!(tree.find(&pos), Some(e));
            assert_eq!(tree.remove(element.into()), Ok(()));
            assert_eq!(tree.find(&pos), None);
        }

        assert_eq!(tree.elements.len(), 0);
        assert_eq!(tree.nodes.len(), 1);
    }

    #[test]
    fn test_volumes() {
        let mut tree = Octree::from_aabb(Aabb::new_unchecked(TUVec3::splat(16u16), 16u16));

        tree.insert(DummyVolume::new(Aabb::new_unchecked(
            TUVec3::new(13, 13, 13),
            3,
        )))
        .unwrap();
        tree.insert(DummyVolume::new(Aabb::new_unchecked(
            TUVec3::new(19, 13, 13),
            3,
        )))
        .unwrap();

        assert_eq!(tree.find(&TUVec3::new(9, 13, 13)), None);
        assert_eq!(tree.find(&TUVec3::new(10, 13, 13)), Some(ElementId(0)));
        assert_eq!(tree.find(&TUVec3::new(13, 13, 13)), Some(ElementId(0)));
        assert_eq!(tree.find(&TUVec3::new(15, 13, 13)), Some(ElementId(0)));
        assert_eq!(tree.find(&TUVec3::new(16, 13, 13)), Some(ElementId(1)));
        assert_eq!(tree.find(&TUVec3::new(19, 13, 13)), Some(ElementId(1)));
        assert_eq!(tree.find(&TUVec3::new(21, 13, 13)), Some(ElementId(1)));
        assert_eq!(tree.find(&TUVec3::new(22, 13, 13)), None);

        assert_eq!(tree.find(&TUVec3::new(13, 9, 13)), None);

        assert!(tree
            .insert(DummyVolume::new(Aabb::new_unchecked(
                TUVec3::new(20, 13, 13),
                3,
            )))
            .is_err());

        assert_eq!(tree.find(&TUVec3::new(19, 13, 13)), Some(ElementId(1)));
        assert_eq!(tree.find(&TUVec3::new(21, 13, 13)), Some(ElementId(1)));
        assert_eq!(tree.find(&TUVec3::new(22, 13, 13)), None);

        let mut hits = HashSet::new();
        tree.intersect_with_for_each(
            |aabb| {
                Aabb::from_min_max(TUVec3::new(10, 13, 13), TUVec3::new(24, 14, 14)).overlaps(aabb)
            },
            |e| {
                hits.insert(e.clone());
            },
        );
        assert_eq!(hits.len(), 2);

        let mut hits = HashSet::new();
        tree.intersect_with_for_each(
            |aabb| {
                Aabb::from_min_max(TUVec3::new(10, 13, 13), TUVec3::new(20, 14, 14)).overlaps(aabb)
            },
            |e| {
                hits.insert(e.clone());
            },
        );
        assert_eq!(hits.len(), 2);
    }

    #[test]
    fn test_iterator() {
        let mut tree = Octree::from_aabb(Aabb::new_unchecked(TUVec3::splat(16), 16));

        for i in 0..16u32 {
            assert_eq!(
                tree.insert(DummyCell::new(TUVec3::splat(i))),
                Ok(ElementId(i))
            );
            assert_eq!(tree.elements.len(), (i + 1) as usize);
            assert_eq!(tree.elements.vec.len(), (i + 1) as usize);
            assert_eq!(tree.elements.garbage_len(), 0);
        }

        for i in 0..16u32 {
            assert_eq!(
                tree.elements.iter().next().unwrap().position,
                TUVec3::splat(i)
            );

            assert_eq!(tree.remove(ElementId(i)), Ok(()));
            assert_eq!(tree.elements.len(), (15 - i) as usize);
            assert_eq!(tree.elements.vec.len(), 16);
            assert_eq!(tree.elements.garbage_len(), (i + 1) as usize);
        }

        for i in 0..16u32 {
            assert_eq!(
                tree.insert(DummyCell::new(TUVec3::splat(i))),
                Ok(ElementId(15 - i))
            );
            assert_eq!(tree.elements.len(), (i + 1) as usize);
            assert_eq!(tree.elements.vec.len(), 16);
            assert_eq!(tree.elements.garbage_len(), (15 - i) as usize);
        }

        for i in 0..16u32 {
            assert_eq!(
                tree.elements.iter().next().unwrap().position,
                TUVec3::splat(15 - i)
            );

            assert_eq!(tree.remove(ElementId(i)), Ok(()));
            assert_eq!(tree.elements.len(), (15 - i) as usize);
            assert_eq!(tree.elements.vec.len(), 16);
            assert_eq!(tree.elements.garbage_len(), (i + 1) as usize);
        }
    }

    #[test]
    fn test_constructors() {
        let aabb = Aabb::default();

        let tree: Octree<u8, DummyCell<u8>> = Octree::from_aabb(aabb);
        assert_eq!(tree.elements.len(), 0);
        assert_eq!(tree.elements.garbage_len(), 0);
        assert_eq!(tree.nodes.len(), 1);
        assert_eq!(tree.nodes.garbage_len(), 0);
        assert_eq!(tree.nodes[0.into()].aabb, aabb);

        let tree: Octree<u8, DummyCell<u8>> = Octree::with_capacity(100);
        assert_eq!(tree.elements.len(), 0);
        assert_eq!(tree.elements.garbage_len(), 0);
        assert_eq!(tree.elements.vec.capacity(), 100);
        assert_eq!(tree.nodes.len(), 1);
        assert_eq!(tree.nodes.garbage_len(), 0);
        assert_eq!(tree.nodes.vec.capacity(), 100);
        assert_eq!(tree.nodes[0.into()].aabb, Aabb::default());

        let tree: Octree<u8, DummyCell<u8>> = Octree::from_aabb_with_capacity(aabb, 50);
        assert_eq!(tree.elements.len(), 0);
        assert_eq!(tree.elements.garbage_len(), 0);
        assert_eq!(tree.elements.vec.capacity(), 50);
        assert_eq!(tree.nodes.len(), 1);
        assert_eq!(tree.nodes.garbage_len(), 0);
        assert_eq!(tree.nodes.vec.capacity(), 50);
        assert_eq!(tree.nodes[0.into()].aabb, aabb);
    }

    #[test]
    fn test_to_vec() {
        let mut tree = Octree::from_aabb(Aabb::new_unchecked(TUVec3::splat(16), 16));
        assert_eq!(
            tree.insert(DummyCell::new(TUVec3::splat(1u8))),
            Ok(ElementId(0))
        );
        assert_eq!(
            tree.insert(DummyCell::new(TUVec3::splat(2u8))),
            Ok(ElementId(1))
        );
        assert_eq!(
            tree.insert(DummyCell::new(TUVec3::splat(3u8))),
            Ok(ElementId(2))
        );

        assert_eq!(tree.remove(1.into()), Ok(()));

        assert_eq!(
            tree.to_vec(),
            vec![
                DummyCell::new(TUVec3::splat(1u8)),
                DummyCell::new(TUVec3::splat(3u8))
            ]
        )
    }

    #[test]
    fn test_overlap() {
        let mut tree = Octree::from_aabb(Aabb::new_unchecked(TUVec3::splat(8u16), 8));

        let v1_volume = Aabb::new(TUVec3::new(9, 5, 4), 4).unwrap();
        let v1 = DummyVolume::new(v1_volume);
        assert_eq!(tree.insert(v1), Ok(ElementId(0)));

        let v2_volume = Aabb::new(TUVec3::new(14, 14, 4), 4).unwrap();
        let v2 = DummyVolume::new(v2_volume);
        assert_eq!(tree.insert(v2), Ok(ElementId(1)));

        let v3_volume = Aabb::new(TUVec3::new(7, 5, 4), 4).unwrap();
        let v3 = DummyVolume::new(v3_volume);
        assert!(tree.insert(v3).is_err());

        assert!(!v1_volume.overlaps(&v2_volume));
        assert!(v1_volume.overlaps(&v3_volume));
    }
}
