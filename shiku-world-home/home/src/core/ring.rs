pub struct Cycle<T: Sized, const N: usize> {
    data: [T; N],
    i: usize,
}

impl<T: Sized, const N: usize> Cycle<T, N> {
    pub fn new(data: [T; N]) -> Cycle<T, N> {
        Cycle { i: 0, data }
    }

    pub fn next(&mut self) -> Option<&T> {
        let next_data = &self.data[self.i];
        self.i = (self.i + 1) % N;
        Some(next_data)
    }
}

use std::ops::{Index, IndexMut};

#[derive(Debug, Clone)]
pub struct RingVec<T: Default> {
    pub data: Vec<T>,
    default: T,
}

impl<T: Default> From<Vec<T>> for RingVec<T> {
    fn from(value: Vec<T>) -> Self {
        let mut r = RingVec::new(10);
        r.data.extend(value);
        r
    }
}

impl<T: Default> RingVec<T> {
    pub fn new(capacity: usize) -> Self {
        RingVec {
            data: Vec::with_capacity(capacity),
            default: T::default(),
        }
    }

    fn push(&mut self, item: T) {
        self.data.push(item);
    }

    pub(crate) fn len(&self) -> usize {
        self.data.len()
    }

    fn wrap_index(&self, index: isize) -> Option<usize> {
        let len = self.data.len() as isize;
        if len == 0 {
            None
        } else {
            Some(((index % len + len) % len) as usize)
        }
    }

    pub fn first(&self) -> &T {
        if self.len() == 0 {
            return &self.default;
        }
        self.index(0)
    }

    pub fn last(&self) -> &T {
        if self.len() == 0 {
            return &self.default;
        }
        self.index((self.len() - 1) as isize)
    }
}

impl<T: Default> Index<isize> for RingVec<T> {
    type Output = T;

    fn index(&self, index: isize) -> &Self::Output {
        match self.wrap_index(index) {
            Some(i) => &self.data[i],
            None => &self.default,
        }
    }
}

impl<T: Default> IndexMut<isize> for RingVec<T> {
    fn index_mut(&mut self, index: isize) -> &mut Self::Output {
        match self.wrap_index(index) {
            Some(i) => &mut self.data[i],
            None => &mut self.default,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_vec() {
        let mut vec = RingVec::new(3);
        vec.push(1);
        vec.push(2);
        vec.push(3);

        // Test positive indices
        assert_eq!(vec[0], 1);
        assert_eq!(vec[1], 2);
        assert_eq!(vec[2], 3);

        // Test wrapping around with positive indices
        assert_eq!(vec[3], 1);
        assert_eq!(vec[4], 2);
        assert_eq!(vec[5], 3);

        // Test negative indices
        assert_eq!(vec[-1], 3);
        assert_eq!(vec[-2], 2);
        assert_eq!(vec[-3], 1);

        // Test wrapping around with negative indices
        assert_eq!(vec[-4], 3);
        assert_eq!(vec[-5], 2);
        assert_eq!(vec[-6], 1);

        // Test mutating elements
        vec[0] = 4;
        vec[1] = 5;
        vec[2] = 6;
        assert_eq!(vec[0], 4);
        assert_eq!(vec[1], 5);
        assert_eq!(vec[2], 6);

        // Test length
        assert_eq!(vec.len(), 3);
    }
}
