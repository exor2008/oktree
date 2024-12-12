//! [`Pool`] implementation.

use std::{
    array::from_fn,
    iter::Enumerate,
    ops::{Index, IndexMut},
};

use smallvec::SmallVec;

use crate::{
    bounding::{Aabb, Unsigned},
    node::{Node, NodeType},
    ElementId, NodeId, TreeError, Volume,
};

/// [`PoolItem`] data structure that combines both the garbage flag
/// and the actual item together for better cache locality.
#[derive(Clone)]
pub(crate) enum PoolItem<T> {
    Filled(T),
    Tombstone(T),
    Empty,
}
impl<T> From<T> for PoolItem<T> {
    fn from(item: T) -> Self {
        PoolItem::Filled(item)
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for PoolItem<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PoolItem::Filled(item) => write!(f, "Filled({:?})", item),
            PoolItem::Tombstone(item) => write!(f, "Garbage({:?})", item),
            PoolItem::Empty => write!(f, "Empty"),
        }
    }
}

/// [`Pool`] data structure.
///
/// When element is removed no memory deallocation happens.
/// Removed elements are only marked as deleted and their memory could be reused.  
#[derive(Clone)]
pub struct Pool<T> {
    pub(crate) vec: Vec<PoolItem<T>>,
    pub(crate) garbage: Vec<usize>,
}

impl<U: Unsigned> Default for Pool<Node<U>> {
    fn default() -> Self {
        let root = Node::default();
        let vec = vec![root.into()];

        Pool {
            vec,
            garbage: Default::default(),
        }
    }
}
impl<U: Unsigned> Pool<Node<U>> {
    /// Clears all the items in the pool
    pub fn clear(&mut self) {
        self.vec.clear();
        self.vec.push(Node::default().into());
        self.garbage.clear();
    }

    /// Clears all the items in the pool and initiates it with an aabb.
    pub fn clear_with_aabb(&mut self, aabb: Aabb<U>) {
        self.vec.clear();
        self.vec.push(Node::from_aabb(aabb, None).into());
        self.garbage.clear();
    }
}

impl<T: Volume> Default for Pool<T> {
    fn default() -> Self {
        Pool {
            vec: Default::default(),
            garbage: Default::default(),
        }
    }
}
impl<T: Volume> Pool<T> {
    /// Clears all the items in the pool
    pub fn clear(&mut self) {
        self.vec.clear();
        self.garbage.clear();
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for Pool<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Pool")
            .field("vec", &self.vec)
            .field("garbage", &self.garbage)
            .finish()
    }
}

impl Default for Pool<SmallVec<[NodeId; 1]>> {
    fn default() -> Self {
        Pool {
            vec: Default::default(),
            garbage: Default::default(),
        }
    }
}
impl Pool<SmallVec<[NodeId; 1]>> {
    /// Clears all the items in the pool
    pub fn clear(&mut self) {
        self.vec.clear();
        self.garbage.clear();
    }
}

/// Indexing a [`pool`](Pool) of [`nodes`](Node) with [`NodeId`]
///
/// ```ignore
/// let node = &tree.nodes[NodeId(42)];
/// // let node = &tree.nodes[ElementId(42)]; // Error
/// ```
impl<U: Unsigned> Index<NodeId> for Pool<Node<U>> {
    type Output = Node<U>;

    fn index(&self, index: NodeId) -> &Self::Output {
        debug_assert!(!self.is_garbage(index), "Indexing garbage node: {index}");
        self.get_unchecked(index)
    }
}

/// Mutable Indexing a [`pool`](Pool) of [`nodes`](Node) with [`NodeId`]
///
/// ```ignore
/// let mut node = &mut tree.nodes[NodeId(42)];
/// // let mut node = &mut tree.nodes[ElementId(42)]; // Error
/// ```
impl<U: Unsigned> IndexMut<NodeId> for Pool<Node<U>> {
    fn index_mut(&mut self, index: NodeId) -> &mut Self::Output {
        debug_assert!(
            !self.is_garbage(index),
            "Mut Indexing garbaged node: {index}"
        );
        self.get_mut_unchecked(index)
    }
}

/// Indexing a [`pool`](Pool) of `T: Position` with [`ElementId`]
///
/// ```ignore
/// let element = &tree.element[ElementId(42)];
/// // let element = &tree.element[NodeId(42)]; // Error
/// ```
impl<T: Volume> Index<ElementId> for Pool<T> {
    type Output = T;

    fn index(&self, index: ElementId) -> &Self::Output {
        debug_assert!(
            !self.is_garbage(index),
            "Indexing garbaged element: {index}"
        );
        self.get_unchecked(index)
    }
}

/// Mutable Indexing a [`pool`](Pool) of `T: Position` with [`ElementId`]
///
/// ```ignore
/// let mut element = &mut tree.element[ElementId(42)];
/// // let mut element = &mut tree.element[NodeId(42)]; // Error
/// ```
impl<T: Volume> IndexMut<ElementId> for Pool<T> {
    fn index_mut(&mut self, index: ElementId) -> &mut Self::Output {
        debug_assert!(
            !self.is_garbage(index),
            "Mut Indexing garbaged element: {index}"
        );
        self.get_mut_unchecked(index)
    }
}

/// Indexing a [`pool`](Pool) of [`node ids`](NodeId) with [`ElementId`]
///
/// ```ignore
/// let node_id = &tree.map[ElementId(42)];
/// // let node_id = &tree.map[NodeId(42)]; // Error
/// ```
impl Index<ElementId> for Pool<NodeId> {
    type Output = NodeId;

    fn index(&self, index: ElementId) -> &Self::Output {
        debug_assert!(
            !self.is_garbage(index),
            "Indexing garbaged element: {index}"
        );
        self.get_unchecked(index)
    }
}

/// Mutable Indexing a [`pool`](Pool) of [`node ids`](NodeId) with [`ElementId`]
///
/// ```ignore
/// let mut node_id = &mut tree.map[ElementId(42)];
/// // let mut node_id = &mut tree.map[NodeId(42)]; // Error
/// ```
impl IndexMut<ElementId> for Pool<NodeId> {
    fn index_mut(&mut self, index: ElementId) -> &mut Self::Output {
        debug_assert!(
            !self.is_garbage(index),
            "Mut Indexing garbaged element: {index}"
        );
        self.get_mut_unchecked(index)
    }
}

impl<T> Pool<T> {
    #[inline(always)]
    fn _insert(&mut self, t: T) -> usize {
        if let Some(idx) = self.garbage.pop() {
            self.vec[idx] = PoolItem::Filled(t);
            idx
        } else {
            self.vec.push(PoolItem::Filled(t));
            self.vec.len() - 1
        }
    }

    /// Restores all the garbage elements back to real elements. Effectively
    /// this is a rollback of all the remove operations that happened
    pub fn restore_garbage(&mut self) -> Result<(), TreeError> {
        let mut is_err = false;
        let mut carry_over = Vec::with_capacity(self.garbage.len());
        for idx in self.garbage.drain(..) {
            let mut item = PoolItem::Empty;
            std::mem::swap(&mut self.vec[idx], &mut item);
            self.vec[idx] = match item {
                PoolItem::Filled(item) => {
                    is_err = true;
                    PoolItem::Filled(item)
                }
                PoolItem::Tombstone(item) => PoolItem::Filled(item),
                PoolItem::Empty => {
                    carry_over.push(idx);
                    PoolItem::Empty
                }
            }
        }
        self.garbage.extend(carry_over);

        match is_err {
            true => Err(TreeError::CorruptGarbage(
                "PollItem::Filled element was garbaged".into(),
            )),
            false => Ok(()),
        }
    }

    /// Collects all the garbage elements and removes them from the pool
    /// releasing the memory and invoking the destructor
    pub fn collect_garbage(&mut self) {
        for garbage in self.garbage.iter_mut() {
            self.vec[*garbage] = PoolItem::Empty;
        }
    }

    /// Returns the number of actual elements.
    ///
    /// Elements marked as deleted are not counted.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.vec.len() - self.garbage_len()
    }

    /// Is the pool is empty.
    ///
    /// Elements marked as deleted are not counted.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the number of deleted elements.
    #[inline(always)]
    pub fn garbage_len(&self) -> usize {
        self.garbage.len()
    }

    /// Returns a [`PoolIterator`], which iterates over an actual elements.
    ///
    /// Elements marked as deleted are skipped.
    pub fn iter(&self) -> PoolIterator<T> {
        PoolIterator::new(self)
    }

    /// Returns a [`PoolIteratorMut`], which iterates over an actual elements.
    ///
    /// Elements marked as deleted are skipped.
    pub fn iter_mut(&mut self) -> PoolIteratorMut<T> {
        PoolIteratorMut::new(self)
    }

    /// Returns a [`PoolIterator`], which iterates over an actual elements and element ids
    ///
    /// Elements marked as deleted are skipped.
    pub fn iter_elements(&self) -> PoolElementIterator<T> {
        PoolElementIterator::new(self)
    }
}

impl<T> IntoIterator for Pool<T> {
    type Item = T;
    type IntoIter = PoolIntoIterator<T>;

    fn into_iter(self) -> Self::IntoIter {
        PoolIntoIterator::new(self)
    }
}

impl<U: Unsigned> Pool<Node<U>> {
    /// Construct a [`Pool`] of [`nodes`](Node) from [`Aabb`].
    ///
    /// Node will adopt aabb's dimensions.
    pub(crate) fn from_aabb(aabb: Aabb<U>) -> Self {
        let root = Node::from_aabb(aabb, None);
        let vec = vec![root.into()];
        Pool {
            vec,
            garbage: Default::default(),
        }
    }

    /// Construct a [`Pool`] of [`nodes`](Node).
    ///
    /// Helps to reduce the amount of the memory reallocations.
    pub(crate) fn with_capacity(capacity: usize) -> Self {
        let root = Node::default();
        let mut vec = Vec::with_capacity(capacity);
        vec.push(root.into());

        Pool {
            vec,
            garbage: Default::default(),
        }
    }

    /// Construct a [`Pool`] of [`nodes`](Node) from [`Aabb`] with capacity.
    ///
    /// Node will adopt aabb's dimensions.
    /// Helps to reduce the amount of the memory reallocations.
    pub(crate) fn from_aabb_with_capacity(aabb: Aabb<U>, capacity: usize) -> Self {
        let root = Node::from_aabb(aabb, None);
        let mut vec = Vec::with_capacity(capacity);
        vec.push(root.into());

        Pool {
            vec,
            garbage: Default::default(),
        }
    }

    #[inline(always)]
    pub(crate) fn insert(&mut self, t: Node<U>) -> NodeId {
        self._insert(t).into()
    }

    #[inline(always)]
    pub(crate) fn branch(&mut self, parent: NodeId) -> [NodeId; 8] {
        let aabbs = self[parent].aabb.split();
        from_fn(|i| self.insert(Node::from_aabb(aabbs[i], Some(parent))))
    }

    pub(crate) fn maybe_collapse(&mut self, parent: NodeId) {
        let mut current = Some(parent);
        while let Some(parent) = current.take() {
            if let NodeType::Branch(ref branch) = self[parent].ntype {
                if branch
                    .children
                    .iter()
                    .all(|&child| self[child].ntype == NodeType::Empty)
                {
                    for child in branch.children {
                        self.tombstone(child);
                    }
                    self[parent].ntype = NodeType::Empty;
                    current = self[parent].parent;
                }
            }
        }
    }
}

impl<T> Pool<T> {
    #[inline(always)]
    pub(crate) fn tombstone(&mut self, element: impl Into<ElementId>) {
        let element = Into::<ElementId>::into(element);
        let index: usize = element.into();

        let mut item = PoolItem::Empty;
        std::mem::swap(&mut self.vec[index], &mut item);
        self.vec[index] = match item {
            PoolItem::Filled(item) => {
                self.garbage.push(index);
                PoolItem::Tombstone(item)
            }
            PoolItem::Tombstone(item) => PoolItem::Tombstone(item),
            PoolItem::Empty => PoolItem::Empty,
        };
    }

    #[inline(always)]
    pub(crate) fn remove(&mut self, element: impl Into<ElementId>) -> Option<T> {
        let element = Into::<ElementId>::into(element);
        let index: usize = element.into();

        let mut ret = None;

        let mut item = PoolItem::Empty;
        std::mem::swap(&mut self.vec[index], &mut item);
        self.vec[index] = match item {
            PoolItem::Filled(item) => {
                ret = Some(item);
                self.garbage.push(index);
                PoolItem::Empty
            }
            PoolItem::Tombstone(item) => {
                ret = Some(item);
                PoolItem::Empty
            }
            PoolItem::Empty => PoolItem::Empty,
        };
        ret
    }

    #[inline(always)]
    pub fn get(&self, element: impl Into<ElementId>) -> Option<&T> {
        let element = Into::<ElementId>::into(element);
        self.vec.get(element.0 as usize).and_then(|item| {
            if let PoolItem::Filled(ref item) = item {
                Some(item)
            } else {
                None
            }
        })
    }

    #[inline(always)]
    pub fn get_mut(&mut self, element: impl Into<ElementId>) -> Option<&mut T> {
        let element = Into::<ElementId>::into(element);
        self.vec.get_mut(element.0 as usize).and_then(|item| {
            if let PoolItem::Filled(ref mut item) = item {
                Some(item)
            } else {
                None
            }
        })
    }

    #[inline(always)]
    pub fn get_unchecked(&self, element: impl Into<ElementId>) -> &T {
        let element = Into::<ElementId>::into(element);
        if let PoolItem::Filled(ref item) = self.vec[element.0 as usize] {
            item
        } else {
            unreachable!("Accessing garbaged element: {element}")
        }
    }

    #[inline(always)]
    pub fn get_mut_unchecked(&mut self, element: impl Into<ElementId>) -> &mut T {
        let element = Into::<ElementId>::into(element);
        if let PoolItem::Filled(ref mut item) = self.vec[element.0 as usize] {
            item
        } else {
            unreachable!("Accessing garbaged element: {element}")
        }
    }

    #[inline(always)]
    pub fn is_garbage(&self, element: impl Into<ElementId>) -> bool {
        let idx: usize = Into::<ElementId>::into(element).into();
        match &self.vec[idx] {
            PoolItem::Filled(_) => false,
            PoolItem::Tombstone(_) => true,
            PoolItem::Empty => true,
        }
    }

    #[inline(always)]
    pub fn has_garbage(&self) -> bool {
        !self.garbage.is_empty()
    }
}

impl<T: Volume> Pool<T> {
    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Pool {
            vec: Vec::with_capacity(capacity),
            garbage: Default::default(),
        }
    }

    #[inline(always)]
    pub(crate) fn insert(&mut self, t: T) -> ElementId {
        self._insert(t).into()
    }
}

impl Pool<NodeId> {
    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Pool {
            vec: Vec::with_capacity(capacity),
            garbage: Default::default(),
        }
    }

    #[inline(always)]
    pub(crate) fn insert(&mut self, t: NodeId) -> ElementId {
        self._insert(t).into()
    }
}

/// Iterator for a [`Pool`].
///
/// Yields only an actual elements.
/// Elements marked as removed are skipped.
#[derive(Clone)]
pub struct PoolIterator<'pool, T> {
    inner: std::slice::Iter<'pool, PoolItem<T>>,
}

impl<'pool, T> PoolIterator<'pool, T> {
    fn new(pool: &'pool Pool<T>) -> Self {
        PoolIterator {
            inner: pool.vec.iter(),
        }
    }
}

impl<'pool, T> Iterator for PoolIterator<'pool, T> {
    type Item = &'pool T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let next = self.inner.next()?;
            match next {
                PoolItem::Filled(item) => {
                    return Some(item);
                }
                PoolItem::Empty => continue,
                PoolItem::Tombstone(_) => continue,
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let hint = self.inner.size_hint();
        (0, hint.1)
    }
}

impl<T> DoubleEndedIterator for PoolIterator<'_, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            let next = self.inner.next_back()?;
            match next {
                PoolItem::Filled(item) => {
                    return Some(item);
                }
                PoolItem::Empty => continue,
                PoolItem::Tombstone(_) => continue,
            }
        }
    }
}

impl<'pool, T> std::iter::FusedIterator for PoolIterator<'pool, T> where
    std::slice::Iter<'pool, PoolItem<T>>: std::iter::FusedIterator
{
}

/// Iterator for a [`Pool`].
///
/// Yields only an actual elements.
/// Elements marked as removed are skipped.
pub struct PoolIteratorMut<'pool, T> {
    inner: std::slice::IterMut<'pool, PoolItem<T>>,
}

impl<'pool, T> PoolIteratorMut<'pool, T> {
    fn new(pool: &'pool mut Pool<T>) -> Self {
        Self {
            inner: pool.vec.iter_mut(),
        }
    }
}

impl<'pool, T> Iterator for PoolIteratorMut<'pool, T> {
    type Item = &'pool mut T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let next = self.inner.next()?;
            match next {
                PoolItem::Filled(item) => {
                    return Some(item);
                }
                PoolItem::Empty => continue,
                PoolItem::Tombstone(_) => continue,
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let hint = self.inner.size_hint();
        (0, hint.1)
    }
}

impl<T> DoubleEndedIterator for PoolIteratorMut<'_, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            let next = self.inner.next_back()?;
            match next {
                PoolItem::Filled(item) => {
                    return Some(item);
                }
                PoolItem::Empty => continue,
                PoolItem::Tombstone(_) => continue,
            }
        }
    }
}

impl<'pool, T> std::iter::FusedIterator for PoolIteratorMut<'pool, T> where
    std::slice::IterMut<'pool, PoolItem<T>>: std::iter::FusedIterator
{
}

/// Iterator for a [`Pool`] that includes element IDs
///
/// Yields only an actual elements.
/// Elements marked as removed are skipped.
#[derive(Clone)]
pub struct PoolElementIterator<'pool, T> {
    inner: Enumerate<std::slice::Iter<'pool, PoolItem<T>>>,
    garbage_len: usize,
}

impl<'pool, T> PoolElementIterator<'pool, T> {
    fn new(pool: &'pool Pool<T>) -> Self {
        PoolElementIterator {
            inner: pool.vec.iter().enumerate(),
            garbage_len: pool.garbage_len(),
        }
    }
}

impl<'pool, T> Iterator for PoolElementIterator<'pool, T> {
    type Item = (ElementId, &'pool T);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let next = self.inner.next()?;
            match next.1 {
                PoolItem::Filled(item) => {
                    return Some((ElementId(next.0 as u32), item));
                }
                PoolItem::Empty => continue,
                PoolItem::Tombstone(_) => continue,
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let hint = self.inner.size_hint();
        (
            hint.0.saturating_sub(self.garbage_len),
            hint.1.map(|x| x.saturating_sub(self.garbage_len)),
        )
    }
}

impl<T> DoubleEndedIterator for PoolElementIterator<'_, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            let next = self.inner.next_back()?;
            match next.1 {
                PoolItem::Filled(item) => {
                    return Some((ElementId(next.0 as u32), item));
                }
                PoolItem::Empty => continue,
                PoolItem::Tombstone(_) => continue,
            }
        }
    }
}

impl<T> ExactSizeIterator for PoolElementIterator<'_, T> {
    fn len(&self) -> usize {
        self.inner.len() - self.garbage_len
    }
}

impl<'pool, T> std::iter::FusedIterator for PoolElementIterator<'pool, T> where
    std::slice::Iter<'pool, PoolItem<T>>: std::iter::FusedIterator
{
}

/// IntoIterator for a [`Pool`] that includes elements
///
/// Yields only an actual elements.
/// Elements marked as removed are skipped.
#[derive(Clone)]
pub struct PoolIntoIterator<T> {
    inner: std::vec::IntoIter<PoolItem<T>>,
    garbage_len: usize,
}

impl<T> PoolIntoIterator<T> {
    fn new(pool: Pool<T>) -> Self {
        PoolIntoIterator {
            garbage_len: pool.garbage_len(),
            inner: pool.vec.into_iter(),
        }
    }
}

impl<T> Iterator for PoolIntoIterator<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let next = self.inner.next()?;
            match next {
                PoolItem::Filled(item) => {
                    return Some(item);
                }
                PoolItem::Empty => continue,
                PoolItem::Tombstone(_) => continue,
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let hint = self.inner.size_hint();
        (
            hint.0.saturating_sub(self.garbage_len),
            hint.1.map(|x| x.saturating_sub(self.garbage_len)),
        )
    }
}

impl<T> DoubleEndedIterator for PoolIntoIterator<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            let next = self.inner.next_back()?;
            match next {
                PoolItem::Filled(item) => {
                    return Some(item);
                }
                PoolItem::Empty => continue,
                PoolItem::Tombstone(_) => continue,
            }
        }
    }
}

impl<T> ExactSizeIterator for PoolIntoIterator<T> {
    fn len(&self) -> usize {
        self.inner.len() - self.garbage_len
    }
}

impl<T> std::iter::FusedIterator for PoolIntoIterator<T> where
    std::vec::IntoIter<PoolItem<T>>: std::iter::FusedIterator
{
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;

    struct DummyNotClonableNotSend<'a> {
        pos: TUVec3<u8>,
        special: &'a str,
    }
    impl Position for DummyNotClonableNotSend<'_> {
        type U = u8;

        fn position(&self) -> TUVec3<Self::U> {
            self.pos
        }
    }

    #[test]
    fn test_non_clonable_compile() {
        let mut test_field = "TEST".to_string();

        let mut pool = Pool::<DummyNotClonableNotSend>::default();
        let element = DummyNotClonableNotSend {
            pos: TUVec3::new(1, 2, 3),
            special: &mut test_field,
        };
        let element_id = pool.insert(element);
        let element = &pool[element_id];
        assert_eq!(element.pos, TUVec3::new(1, 2, 3));
    }

    #[test]
    fn test_remove() {
        let mut pool = Pool::<TUVec3u8>::with_capacity(16);
        for i in 0..16 {
            assert_eq!(pool.insert(TUVec3u8::new(i, i, i)), ElementId(i as u32));
            assert_eq!(pool.len(), (i + 1) as usize);
            assert_eq!(pool.garbage_len(), 0_usize);
        }

        for i in 0..8 {
            pool.tombstone(NodeId(i));
            assert_eq!(pool.len(), (15 - i) as usize);
            assert_eq!(pool.garbage_len(), (i + 1) as usize);
        }

        for i in 0..8 {
            pool.remove(NodeId(i));
            assert_eq!(pool.len(), 8_usize);
            assert_eq!(pool.garbage_len(), 8_usize);
        }

        for i in 8..16 {
            pool.remove(NodeId(i));
            assert_eq!(pool.len(), (15 - i) as usize);
            assert_eq!(pool.garbage_len(), (i + 1) as usize);
        }
    }

    #[test]
    fn test_collect_garbage() {
        let mut pool = Pool::<TUVec3u8>::with_capacity(16);

        for i in 0..16 {
            assert_eq!(pool.insert(TUVec3u8::new(i, i, i)), ElementId(i as u32));
        }

        for i in 0..4 {
            pool.tombstone(NodeId(i));
        }

        for i in 4..8 {
            pool.remove(NodeId(i));
        }

        pool.collect_garbage();

        assert_eq!(pool.garbage_len(), 8);
        assert_eq!(pool.len(), 8);
    }

    #[test]
    fn test_restore_garbage_tombstone() {
        let mut pool = Pool::<TUVec3u8>::with_capacity(16);

        for i in 0..16 {
            assert_eq!(pool.insert(TUVec3u8::new(i, i, i)), ElementId(i as u32));
        }

        pool.tombstone(ElementId(4));
        pool.tombstone(ElementId(6));
        pool.tombstone(ElementId(10));

        assert_eq!(pool.len(), 13);
        assert_eq!(pool.garbage_len(), 3);

        assert!(pool.restore_garbage().is_ok());

        assert_eq!(pool.len(), 16);
        assert_eq!(pool.garbage_len(), 0);
    }

    #[test]
    fn test_restore_garbage_remove() {
        let mut pool = Pool::<TUVec3u8>::with_capacity(16);

        for i in 0..16 {
            assert_eq!(pool.insert(TUVec3u8::new(i, i, i)), ElementId(i as u32));
        }

        pool.remove(ElementId(4));
        pool.remove(ElementId(6));
        pool.remove(ElementId(10));

        assert_eq!(pool.len(), 13);
        assert_eq!(pool.garbage_len(), 3);

        assert!(pool.restore_garbage().is_ok());

        assert_eq!(pool.len(), 13);
        assert_eq!(pool.garbage_len(), 3);
    }

    #[test]
    fn test_restore_garbage_remove_tombstone() {
        let mut pool = Pool::<TUVec3u8>::with_capacity(16);

        for i in 0..16 {
            assert_eq!(pool.insert(TUVec3u8::new(i, i, i)), ElementId(i as u32));
        }

        pool.tombstone(ElementId(4));
        pool.remove(ElementId(6));
        pool.tombstone(ElementId(8));
        pool.remove(ElementId(10));
        pool.tombstone(ElementId(12));
        pool.remove(ElementId(14));

        assert_eq!(pool.len(), 10);
        assert_eq!(pool.garbage_len(), 6);

        assert!(pool.restore_garbage().is_ok());

        assert_eq!(pool.len(), 13);
        assert_eq!(pool.garbage_len(), 3);
    }

    #[test]
    fn test_iterator() {
        let mut pool = Pool::<TUVec3u8>::with_capacity(16);

        let iter = pool.iter();
        assert_eq!(iter.size_hint(), (0, Some(0)));

        for i in 0..16 {
            assert_eq!(pool.insert(TUVec3u8::new(i, i, i)), ElementId(i as u32));

            let i = i as usize;
            let iter = pool.iter();
            assert_eq!(iter.size_hint(), (0, Some(i + 1)));
        }

        for i in [0, 2, 4, 6].into_iter() {
            pool.tombstone(ElementId(i as u32));

            let iter = pool.iter();
            assert_eq!(iter.size_hint(), (0, Some(16)));
        }

        for i in [1, 3, 5, 7].into_iter() {
            pool.remove(ElementId(i));

            let iter = pool.iter();
            assert_eq!(iter.size_hint(), (0, Some(16)));
        }

        let mut iter = pool.iter();
        assert_eq!(iter.size_hint(), (0, Some(16)));

        let mut idx = 0;
        // After iter.next() iter.inner.len() suddenly changes from 16 to 7.
        // This is why we can not implement PollIterator::len() as iter.inner.len() - garbage_len
        // because len becomes 16 -> 7 and garbage_len remains 8
        // Resulting len() will be 7 - 8, which is overflow.
        while let Some(el) = iter.next() {
            assert_eq!(el, &TUVec3u8::new(idx + 8, idx + 8, idx + 8));
            assert_eq!(iter.size_hint(), (0, Some(7 - idx as usize)));
            idx += 1;
        }
    }

    #[test]
    fn test_iterator_mut() {
        let mut pool = Pool::<TUVec3u8>::with_capacity(16);

        let iter = pool.iter_mut();
        assert_eq!(iter.size_hint(), (0, Some(0)));

        for i in 0..16 {
            assert_eq!(pool.insert(TUVec3u8::new(i, i, i)), ElementId(i as u32));

            let i = i as usize;
            let iter = pool.iter_mut();
            assert_eq!(iter.size_hint(), (0, Some(i + 1)));
        }

        for i in [0, 2, 4, 6].into_iter() {
            pool.tombstone(ElementId(i as u32));

            let iter = pool.iter_mut();
            assert_eq!(iter.size_hint(), (0, Some(16)));
        }

        for i in [1, 3, 5, 7].into_iter() {
            pool.remove(ElementId(i));

            let iter = pool.iter_mut();
            assert_eq!(iter.size_hint(), (0, Some(16)));
        }

        let mut iter = pool.iter_mut();
        assert_eq!(iter.size_hint(), (0, Some(16)));

        let mut idx = 0;
        // After iter.next() iter.inner.len() suddenly changes from 16 to 7.
        // This is why we can not implement PollIteratorMut::len() as iter.inner.len() - garbage_len
        // because len becomes 16 -> 7 and garbage_len remains 8
        // Resulting len() will be 7 - 8, which is overflow.
        while let Some(el) = iter.next() {
            assert_eq!(el, &mut TUVec3u8::new(idx + 8, idx + 8, idx + 8));
            assert_eq!(iter.size_hint(), (0, Some(7 - idx as usize)));
            idx += 1;
        }
    }
}
