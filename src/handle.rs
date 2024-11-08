use std::{
    fmt::{Debug, Display},
    hash::Hash,
    marker::PhantomData,
};

pub struct Handle<T: ?Sized> {
    _type: PhantomData<fn() -> T>,
    id: u64,
}

impl<T: ?Sized> Handle<T> {
    #[inline]
    pub fn cast<T2>(self) -> Handle<T2> {
        Handle {
            _type: PhantomData,
            id: self.id,
        }
    }

    #[inline]
    pub fn from_raw(id: u64) -> Self {
        Self {
            _type: PhantomData,
            id,
        }
    }

    #[inline]
    pub fn from_parts(meta: u32, index: u32) -> Self {
        Self {
            _type: PhantomData,
            id: ((meta as u64) << u32::BITS) + index as u64,
        }
    }

    #[inline]
    pub fn raw(self) -> u64 {
        self.id
    }

    #[inline]
    pub fn index(self) -> u32 {
        self.id as u32
    }

    #[inline]
    pub fn uindex(self) -> usize {
        self.index() as usize
    }

    #[inline]
    pub fn meta(self) -> u32 {
        (self.id >> u32::BITS) as u32
    }
}

impl<T: ?Sized> Debug for Handle<T> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Handle").field("id", &self.id).finish()
    }
}

impl<T: ?Sized> Display for Handle<T> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Handle({})", self.id)
    }
}

impl<T: ?Sized> Copy for Handle<T> {}
impl<T: ?Sized> Clone for Handle<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            _type: PhantomData,
            id: self.id.clone(),
        }
    }
}

impl<T: ?Sized> Eq for Handle<T> {}
impl<T: ?Sized> PartialEq for Handle<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T: ?Sized> Ord for Handle<T> {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}
impl<T: ?Sized> PartialOrd for Handle<T> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl<T: ?Sized> Hash for Handle<T> {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_parts() {
        let meta = 1234;
        let index = 5678;
        let uindex = index as usize;
        let raw = ((meta as u64) << u32::BITS) + index as u64;
        let handle = Handle::<()>::from_parts(meta, index);
        assert_eq!(handle.meta(), meta);
        assert_eq!(handle.index(), index);
        assert_eq!(handle.uindex(), uindex);
        assert_eq!(handle.raw(), raw);
    }

    #[test]
    fn from_raw() {
        let meta = 1234;
        let index = 5678;
        let uindex = index as usize;
        let raw = ((meta as u64) << u32::BITS) + index as u64;
        let handle = Handle::<()>::from_raw(raw);
        assert_eq!(handle.meta(), meta);
        assert_eq!(handle.index(), index);
        assert_eq!(handle.uindex(), uindex);
        assert_eq!(handle.raw(), raw);
    }
}
