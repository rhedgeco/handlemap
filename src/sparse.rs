use std::{
    collections::VecDeque,
    ops::{Index, IndexMut},
};

use crate::Handle;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SparseHandleMap<T> {
    values: Vec<(Handle<T>, Option<T>)>,
    available: VecDeque<usize>,
}

impl<T> Default for SparseHandleMap<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> IndexMut<Handle<T>> for SparseHandleMap<T> {
    fn index_mut(&mut self, handle: Handle<T>) -> &mut Self::Output {
        self.get_mut(handle).expect("valid handle")
    }
}

impl<T> Index<Handle<T>> for SparseHandleMap<T> {
    type Output = T;

    fn index(&self, handle: Handle<T>) -> &Self::Output {
        self.get(handle).expect("valid handle")
    }
}

impl<T> SparseHandleMap<T> {
    pub const fn new() -> Self {
        Self {
            values: Vec::new(),
            available: VecDeque::new(),
        }
    }

    /// Returns the number of populated values in the map.
    pub fn len(&self) -> usize {
        self.values.len() - self.available.len()
    }

    /// Returns the total number of elements the map can hold without reallocating.
    pub fn capacity(&self) -> usize {
        self.values.capacity()
    }

    /// Returns `true` if the map contains no elements.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns a reference to the value associated with `handle`.
    ///
    /// Returns `None` if the value does not exist.
    pub fn get(&self, handle: Handle<T>) -> Option<&T> {
        let (found_handle, option) = self.values.get(handle.uindex())?;
        if &handle != found_handle {
            return None;
        }
        option.as_ref()
    }

    /// Returns a mutable reference to the value associated with `handle`.
    ///
    /// Returns `None` if the value does not exist.
    pub fn get_mut(&mut self, handle: Handle<T>) -> Option<&mut T> {
        let (found_handle, option) = self.values.get_mut(handle.uindex())?;
        if &handle != found_handle {
            return None;
        }
        option.as_mut()
    }

    /// Predicts the next handle that will be generated.
    ///
    /// This is just an alias for [`predict_handle(0)`](Self::predict_handle).
    ///
    /// # Panics
    /// This function will panic if the predicted capacity exceeds `u32::MAX`
    pub fn predict_next_handle(&self) -> Handle<T> {
        self.predict_handle(0)
    }

    /// Predicts the handle that will be generated after inserting `count` values.
    ///
    /// This is only accurate for multiple inserts. Once a single removal is made, this prediction can de-sync.
    ///
    /// # Panics
    /// This function will panic if the predicted capacity exceeds `u32::MAX`
    pub fn predict_handle(&self, count: usize) -> Handle<T> {
        // first check if there will be existing handles re-used
        if let Some(index) = self.available.get(count) {
            let (handle, _) = &self.values[*index];
            return Handle::from_parts(handle.meta().wrapping_add(1), handle.index());
        }

        // otherwise generate the next new handle
        let new_count = count - self.available.len();
        let new_index = self.values.len() + new_count;
        match new_index <= u32::MAX as usize {
            true => Handle::from_parts(0, new_index as u32),
            false => panic!("capacity overflow"),
        }
    }

    /// Inserts `value` into the map, returning a [`Handle`] to its location.
    ///
    /// # Panics
    /// Panics if the new capacity exceeds `u32::MAX`
    pub fn insert(&mut self, value: T) -> Handle<T> {
        if let Some(index) = self.available.pop_front() {
            let (handle, option) = &mut self.values[index];
            *handle = Handle::from_parts(handle.meta().wrapping_add(1), handle.index());
            *option = Some(value);
            return *handle;
        }

        let new_index = self.values.len();
        match new_index <= u32::MAX as usize {
            false => panic!("capacity overflow"),
            true => {
                let handle = Handle::from_parts(0, new_index as u32);
                self.values.push((handle, Some(value)));
                handle
            }
        }
    }

    /// Removes the value associated with `handle` from the map.
    ///
    /// Returns `None` if the value does not exist.
    pub fn remove(&mut self, handle: Handle<T>) -> Option<T> {
        let (found_handle, option) = self.values.get_mut(handle.uindex())?;
        if &handle != found_handle {
            return None;
        }

        let value = option.take()?;
        self.available.push_back(handle.uindex());
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

/// An iterator that yields all handles and value refrences in a [`SparseHandleMap`].
pub struct Iter<'a, T> {
    inner: std::slice::Iter<'a, (Handle<T>, Option<T>)>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = (Handle<T>, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            return match self.inner.next()? {
                (handle, Some(value)) => Some((*handle, value)),
                (_, None) => continue,
            };
        }
    }
}

/// An iterator that yields all handles and mutable value refrences in a [`SparseHandleMap`].
pub struct IterMut<'a, T> {
    inner: std::slice::IterMut<'a, (Handle<T>, Option<T>)>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = (Handle<T>, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            return match self.inner.next()? {
                (handle, Some(value)) => Some((*handle, value)),
                (_, None) => continue,
            };
        }
    }
}

/// An iterator that yields all handles in a [`SparseHandleMap`].
pub struct Handles<'a, T> {
    inner: std::slice::Iter<'a, (Handle<T>, Option<T>)>,
}

impl<'a, T> Iterator for Handles<'a, T> {
    type Item = Handle<T>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            return match self.inner.next()? {
                (handle, Some(_)) => Some(*handle),
                (_, None) => continue,
            };
        }
    }
}

/// An iterator that yields all value refrences in a [`SparseHandleMap`].
pub struct Values<'a, T> {
    inner: std::slice::Iter<'a, (Handle<T>, Option<T>)>,
}

impl<'a, T> Iterator for Values<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            return match self.inner.next()? {
                (_, Some(value)) => Some(value),
                (_, None) => continue,
            };
        }
    }
}

/// An iterator that yields all mutable value refrences in a [`SparseHandleMap`].
pub struct ValuesMut<'a, T> {
    inner: std::slice::IterMut<'a, (Handle<T>, Option<T>)>,
}

impl<'a, T> Iterator for ValuesMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            return match self.inner.next()? {
                (_, Some(value)) => Some(value),
                (_, None) => continue,
            };
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn insert() {
        let mut map = SparseHandleMap::new();
        let handle = map.insert(42);
        assert_eq!(map.get(handle), Some(&42));
    }

    #[test]
    pub fn remove() {
        let mut map = SparseHandleMap::new();
        let handle = map.insert(42);
        assert_eq!(map.remove(handle), Some(42));
        assert_eq!(map.get(handle), None);
    }

    #[test]
    pub fn invalidated_handle() {
        let mut map = SparseHandleMap::new();
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
        let mut map = SparseHandleMap::new();
        assert!(map.is_empty());

        let handle = map.insert(123);
        assert_eq!(map.len(), 1);

        map.insert(456);
        assert_eq!(map.len(), 2);

        map.remove(handle);
        assert_eq!(map.len(), 1);
    }
}
