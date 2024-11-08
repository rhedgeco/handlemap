use std::ops::{Index, IndexMut};

use crate::{Handle, SparseHandleMap};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DenseHandleMap<T> {
    link: SparseHandleMap<usize>,
    values: Vec<(Handle<T>, T)>,
}

impl<T> Default for DenseHandleMap<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> IndexMut<Handle<T>> for DenseHandleMap<T> {
    fn index_mut(&mut self, handle: Handle<T>) -> &mut Self::Output {
        self.get_mut(handle).expect("valid handle")
    }
}

impl<T> Index<Handle<T>> for DenseHandleMap<T> {
    type Output = T;

    fn index(&self, handle: Handle<T>) -> &Self::Output {
        self.get(handle).expect("valid handle")
    }
}

impl<T> DenseHandleMap<T> {
    pub const fn new() -> Self {
        Self {
            link: SparseHandleMap::new(),
            values: Vec::new(),
        }
    }

    /// Returns the number values in the map.
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Returns the total number of elements the map can hold without reallocating.
    pub fn capacity(&self) -> usize {
        self.values.capacity()
    }

    /// Returns `true` if the map contains no elements.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Returns a reference to the value associated with `handle`.
    ///
    /// Returns `None` if the value does not exist.
    pub fn get(&self, handle: Handle<T>) -> Option<&T> {
        let index = *self.link.get(handle.cast())?;
        Some(&self.values[index].1)
    }

    /// Returns a mutable reference to the value associated with `handle`.
    ///
    /// Returns `None` if the value does not exist.
    pub fn get_mut(&mut self, handle: Handle<T>) -> Option<&mut T> {
        let index = *self.link.get(handle.cast())?;
        Some(&mut self.values[index].1)
    }

    /// Inserts `value` into the map, returning a [`Handle`] to its location.
    ///
    /// # Panics
    /// Panics if the new capacity exceeds `u32::MAX` bytes.
    pub fn insert(&mut self, value: T) -> Handle<T> {
        let new_index = self.values.len();
        let handle = self.link.insert(new_index).cast();
        self.values.push((handle, value));
        handle
    }

    /// Removes the value associated with `handle` from the map.
    ///
    /// Returns `None` if the value does not exist.
    pub fn remove(&mut self, handle: Handle<T>) -> Option<T> {
        // get the index for the value
        let index = self.link.remove(handle.cast())?;

        // swap remove the value from the vec
        let (_, value) = self.values.swap_remove(index);
        // if a swapped value exists in its place, correct its linked index
        if let Some((swapped_handle, _)) = self.values.get(index) {
            self.link[swapped_handle.cast()] = index;
        }

        // then return the removed value
        Some(value)
    }

    /// Returns an iterator that yields all value refrences in the map and their associated [`Handle`].
    pub fn iter(&self) -> Iter<T> {
        Iter {
            inner: self.values.iter(),
        }
    }

    /// Returns an iterator that yields all mutable value refrences in the map and their associated [`Handle`].
    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut {
            inner: self.values.iter_mut(),
        }
    }

    /// Returns an iterator that yields every valid value [`Handle`] in the map.
    pub fn handles(&self) -> Handles<T> {
        Handles {
            inner: self.values.iter(),
        }
    }

    /// Returns an iterator that yields all value refrences in the map.
    pub fn values(&self) -> Values<T> {
        Values {
            inner: self.values.iter(),
        }
    }

    /// Returns an iterator that yields all mutable value refrences in the map.
    pub fn values_mut(&mut self) -> ValuesMut<T> {
        ValuesMut {
            inner: self.values.iter_mut(),
        }
    }
}

/// An iterator that yields all handles and value refrences in a [`DenseHandleMap`].
pub struct Iter<'a, T> {
    inner: std::slice::Iter<'a, (Handle<T>, T)>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = (Handle<T>, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(handle, value)| (*handle, value))
    }
}

/// An iterator that yields all handles and mutable value refrences in a [`DenseHandleMap`].
pub struct IterMut<'a, T> {
    inner: std::slice::IterMut<'a, (Handle<T>, T)>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = (Handle<T>, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(handle, value)| (*handle, value))
    }
}

/// An iterator that yields all handles in a [`DenseHandleMap`].
pub struct Handles<'a, T> {
    inner: std::slice::Iter<'a, (Handle<T>, T)>,
}

impl<'a, T> Iterator for Handles<'a, T> {
    type Item = Handle<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(handle, _)| *handle)
    }
}

/// An iterator that yields all value refrences in a [`DenseHandleMap`].
pub struct Values<'a, T> {
    inner: std::slice::Iter<'a, (Handle<T>, T)>,
}

impl<'a, T> Iterator for Values<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(_, value)| value)
    }
}

/// An iterator that yields all mutable value refrences in a [`DenseHandleMap`].
pub struct ValuesMut<'a, T> {
    inner: std::slice::IterMut<'a, (Handle<T>, T)>,
}

impl<'a, T> Iterator for ValuesMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(_, value)| value)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn insert() {
        let mut map = DenseHandleMap::new();
        let handle = map.insert(42);
        assert_eq!(map.get(handle), Some(&42));
    }

    #[test]
    pub fn remove() {
        let mut map = DenseHandleMap::new();
        let handle = map.insert(42);
        assert_eq!(map.remove(handle), Some(42));
        assert_eq!(map.get(handle), None);
    }

    #[test]
    pub fn invalidated_handle() {
        let mut map = DenseHandleMap::new();
        let handle = map.insert(123);
        map.insert(456);
        map.remove(handle);
        let new_handle = map.insert(789);
        assert_ne!(handle, new_handle);
        assert_eq!(map.get(handle), None);
        assert_eq!(map.get(new_handle), Some(&789));
    }

    #[test]
    pub fn length() {
        let mut map = DenseHandleMap::new();
        assert!(map.is_empty());

        let handle = map.insert(123);
        assert_eq!(map.len(), 1);

        map.insert(456);
        assert_eq!(map.len(), 2);

        map.remove(handle);
        assert_eq!(map.len(), 1);
    }
}
