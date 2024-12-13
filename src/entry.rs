use super::*;
use crate::prelude::*;
use core::fmt;
use core::ops::DerefMut;

impl<U: Unsigned, T: Volume<U = U>> Octree<U, T> {
    /// Gets the given key's corresponding entry in the map for in-place manipulation.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut letters = Octree::new();
    ///
    /// for ch in "a short treatise on fungi".chars() {
    ///     letters.entry(ch).and_modify(|mut counter| *counter += 1).or_insert(1);
    /// }
    /// ```
    #[inline]
    pub fn entry(&mut self, key: TUVec3<U>) -> Entry<U, T> {
        match self.find(&key) {
            Some(value) => Entry::Occupied(OccupiedEntry {
                base: self,
                element: value,
                key,
            }),
            None => Entry::Vacant(VacantEntry { base: self, key }),
        }
    }
}

/// A view into a single entry in a map, which may either be vacant or occupied.
///
/// This `enum` is constructed from the [`entry`] method on [`HashMap`].
///
/// [`entry`]: HashMap::entry
pub enum Entry<'a, U: Unsigned, T: Volume<U = U>> {
    /// An occupied entry.
    Occupied(OccupiedEntry<'a, U, T>),

    /// A vacant entry.
    Vacant(VacantEntry<'a, U, T>),
}

impl<U: Unsigned, T: Volume<U = U>> fmt::Debug for Entry<'_, U, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Vacant(ref v) => f.debug_tuple("Entry").field(v).finish(),
            Self::Occupied(ref o) => f.debug_tuple("Entry").field(o).finish(),
        }
    }
}

impl<'a, U: Unsigned, T: Volume<U = U>> Entry<'a, U, T> {
    /// Ensures a value is in the entry by inserting the default if empty, and returns
    /// a mutable reference to the value in the entry.
    #[inline]
    pub fn or_insert(self, default: T) -> OccupiedEntry<'a, U, T> {
        match self {
            Self::Occupied(entry) => entry,
            Self::Vacant(entry) => entry.insert(default),
        }
    }

    /// Ensures a value is in the entry by inserting the default if empty, and returns
    /// a mutable reference to the value in the entry.
    #[inline]
    pub fn or_try_insert(self, default: T) -> Result<OccupiedEntry<'a, U, T>, TreeError> {
        match self {
            Self::Occupied(entry) => Ok(entry),
            Self::Vacant(entry) => entry.try_insert(default),
        }
    }

    /// Ensures a value is in the entry by inserting the result of the default function if empty,
    /// and returns a mutable reference to the value in the entry.
    #[inline]
    pub fn or_insert_with<F: FnOnce() -> T>(self, default: F) -> OccupiedEntry<'a, U, T> {
        match self {
            Self::Occupied(entry) => entry,
            Self::Vacant(entry) => entry.insert(default()),
        }
    }

    /// Ensures a value is in the entry by inserting the result of the default function if empty,
    /// and returns a mutable reference to the value in the entry.
    #[inline]
    pub fn or_try_insert_with<F: FnOnce() -> T>(
        self,
        default: F,
    ) -> Result<OccupiedEntry<'a, U, T>, TreeError> {
        match self {
            Self::Occupied(entry) => Ok(entry),
            Self::Vacant(entry) => entry.try_insert(default()),
        }
    }

    /// Ensures a value is in the entry by inserting, if empty, the result of the default function.
    /// This method allows for generating key-derived values for insertion by providing the default
    /// function a reference to the key that was moved during the `.entry(key)` method call.
    ///
    /// The reference to the moved key is provided so that cloning or copying the key is
    /// unnecessary, unlike with `.or_insert_with(|| ... )`.
    #[inline]
    pub fn or_insert_with_key<F: FnOnce(&TUVec3<U>) -> T>(
        self,
        default: F,
    ) -> OccupiedEntry<'a, U, T> {
        match self {
            Self::Occupied(entry) => entry,
            Self::Vacant(entry) => {
                let value = default(entry.key());
                entry.insert(value)
            }
        }
    }

    /// Ensures a value is in the entry by inserting, if empty, the result of the default function.
    /// This method allows for generating key-derived values for insertion by providing the default
    /// function a reference to the key that was moved during the `.entry(key)` method call.
    ///
    /// The reference to the moved key is provided so that cloning or copying the key is
    /// unnecessary, unlike with `.or_insert_with(|| ... )`.
    #[inline]
    pub fn or_try_insert_with_key<F: FnOnce(&TUVec3<U>) -> T>(
        self,
        default: F,
    ) -> Result<OccupiedEntry<'a, U, T>, TreeError> {
        match self {
            Self::Occupied(entry) => Ok(entry),
            Self::Vacant(entry) => {
                let value = default(entry.key());
                entry.try_insert(value)
            }
        }
    }

    /// Returns a reference to this entry's key.
    #[inline]
    pub fn key(&self) -> &TUVec3<U> {
        match *self {
            Self::Occupied(ref entry) => entry.key(),
            Self::Vacant(ref entry) => entry.key(),
        }
    }

    /// Provides in-place access to an occupied entry before any
    /// potential inserts into the map.
    #[inline]
    pub fn and<F>(self, f: F) -> Self
    where
        F: FnOnce(&T),
    {
        match self {
            Self::Occupied(entry) => {
                f(entry.get());
                Self::Occupied(entry)
            }
            Self::Vacant(entry) => Self::Vacant(entry),
        }
    }

    /// Provides in-place mutable access to an occupied entry before any
    /// potential inserts into the map.
    #[inline]
    pub fn and_modify<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut T),
    {
        match self {
            Self::Occupied(mut entry) => {
                f(entry.get_mut());
                Self::Occupied(entry)
            }
            Self::Vacant(entry) => Self::Vacant(entry),
        }
    }
}

/// A view into an occupied entry in a `Octree`.
/// It is part of the [`Entry`] enum.
pub struct OccupiedEntry<'a, U: Unsigned, T: Volume<U = U>> {
    base: &'a mut Octree<U, T>,
    key: TUVec3<U>,
    element: ElementId,
}

impl<U: Unsigned, T: Volume<U = U>> fmt::Debug for OccupiedEntry<'_, U, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OccupiedEntry")
            .field("key", &self.key)
            .field("element", &self.element)
            .finish_non_exhaustive()
    }
}

impl<'a, U: Unsigned, T: Volume<U = U>> OccupiedEntry<'a, U, T> {
    /// Gets a reference to the key in the entry.
    #[inline]
    pub fn key(&self) -> &TUVec3<U> {
        &self.key
    }

    #[inline]
    pub fn element(&self) -> ElementId {
        self.element
    }

    /// Gets a reference to the value in the entry.
    #[inline]
    pub fn get(&self) -> &T {
        self.base.get_element(self.element).unwrap()
    }

    /// Gets a mutable reference to the value in the entry.
    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        self.base.get_element_mut(self.element).unwrap()
    }

    /// Gets a reference to the value in the entry.
    #[inline]
    pub fn into_ref(self) -> &'a T {
        self.base.get_element(self.element).unwrap()
    }

    /// Gets a mutable reference to the value in the entry.
    #[inline]
    pub fn into_mut(self) -> &'a mut T {
        self.base.get_element_mut(self.element).unwrap()
    }

    /// Sets the value of the entry, and returns the entry's old value.
    #[inline]
    pub fn insert(&'a mut self, value: T) -> &'a mut T {
        let ret = self.base.get_element_mut(self.element).unwrap();
        *ret = value;
        ret
    }
}

impl<U: Unsigned, T: Volume<U = U>> Deref for OccupiedEntry<'_, U, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.get()
    }
}

impl<U: Unsigned, T: Volume<U = U>> DerefMut for OccupiedEntry<'_, U, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.get_mut()
    }
}

/// A view into a vacant entry in a `Octree`.
/// It is part of the [`Entry`] enum.
pub struct VacantEntry<'a, U: Unsigned, T: Volume<U = U>> {
    base: &'a mut Octree<U, T>,
    key: TUVec3<U>,
}

impl<U: Unsigned, T: Volume<U = U>> fmt::Debug for VacantEntry<'_, U, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VacantEntry")
            .field("key", &self.key)
            .finish_non_exhaustive()
    }
}

impl<'a, U: Unsigned, T: Volume<U = U>> VacantEntry<'a, U, T> {
    /// Gets a reference to the key that would be used when inserting a value
    /// through the `VacantEntry`.
    #[inline]
    pub fn key(&self) -> &TUVec3<U> {
        &self.key
    }

    /// Take ownership of the key.
    #[inline]
    pub fn into_key(self) -> TUVec3<U> {
        self.key
    }

    /// Sets the value of the entry with the `VacantEntry`'s key,
    /// and returns a mutable reference to it.
    #[inline]
    pub fn insert(self, value: T) -> OccupiedEntry<'a, U, T> {
        let element = self.base.insert(value).unwrap();
        OccupiedEntry {
            base: self.base,
            key: self.key,
            element,
        }
    }

    /// Sets the value of the entry with the `VacantEntry`'s key,
    /// and returns a mutable reference to it.
    #[inline]
    pub fn try_insert(self, value: T) -> Result<OccupiedEntry<'a, U, T>, TreeError> {
        let element = self.base.insert(value)?;
        Ok(OccupiedEntry {
            base: self.base,
            key: self.key,
            element,
        })
    }
}
