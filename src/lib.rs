
use std::marker::PhantomData;
use std::ops::Deref;
use std::ops::Range;

#[macro_use]
extern crate derivative;

extern crate type_name_value;

pub use type_name_value::{Named, name};

/// A wrapper around a ``Vec`` that ensures that any valid indices will always remain valid. In
/// practice, this means a ``FixedVec`` will never shrink in size (it can, however, grow in size).
#[derive(Derivative)]
#[derivative(Debug(bound="A: std::fmt::Debug"), PartialEq(bound="A: PartialEq"), Eq(bound="A: Eq"), Hash(bound="A: std::hash::Hash"), PartialOrd(bound="A: PartialOrd"), Ord(bound="A: Ord"))]
pub struct FixedVec<A, Name> {
    inner: Named<Vec<A>, Name>,
    _phantom: PhantomData<Name>,
}

impl<A, Name> Deref for FixedVec<A, Name> {
    type Target = Vec<A>;

    fn deref(&self) -> &Self::Target {
        self.inner.unname_ref()
    }
}

/// A valid index into a ``FixedVec`` with name ``Name``. This cannot be created directly except
/// through the ``check_index`` method of the same ``FixedVec``.
#[derive(Derivative)]
#[derivative(Clone(bound=""), Copy(bound=""), Debug(bound=""), PartialEq(bound=""), Eq(bound=""), Hash(bound=""), PartialOrd(bound=""), Ord(bound=""))]
pub struct Index<Name> {
    index: usize,
    _phantom: PhantomData<Name>,
}

impl<Name> Deref for Index<Name> {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.index
    }
}

/// A range of valid indices into a ``FixedVec`` with name ``Name``. This cannot be created except
/// through the ``check_range`` method of a ``FixedVec``.
#[derive(Derivative)]
#[derivative(Clone(bound=""))]
pub struct CheckedRange<Name> {
    range: Range<usize>,
    _phantom: PhantomData<Name>,
}

impl<Name> Iterator for CheckedRange<Name> {
    type Item = Index<Name>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.range.start >= self.range.end {
            None
        } else {
            let tmp = Index {
                index: self.range.start,
                _phantom: PhantomData,
            };
            self.range.start += 1;
            Some(tmp)
        }
    }
}

impl<A, Name> FixedVec<A, Name> {
    /// Create a ``FixedVec`` from a named ``Vec``. To use this method, first assign a name to a
    /// ``Vec`` using ``name!()``.
    pub fn fix(val: Named<Vec<A>, Name>) -> Self {
        FixedVec {
            inner: val,
            _phantom: PhantomData,
        }
    }
    
    /// Unwrap's the inner ``Vec`` so that it can be changed again, including its length. Since
    /// this takes ownership of the ``FixedVec``, it indirectly invalidates all ``Index``'s with
    /// the same ``Name``.
    pub fn unfix(self) -> Vec<A> {
        self.inner.unname()
    }
    
    /// Perform an index bounds check. This Is the only way to directly create an ``Index``. The created
    /// ``Index`` will share the same ``Name`` as the ``FixedVec``, so that it can later be used
    /// with the ``get`` and ``get_mut`` methods.
    ///
    /// # Example
    ///
    /// ```
    /// # use fixed_vec::*;
    /// let v = vec![1, 2, 3];
    /// let v = name!(v);
    /// let v = FixedVec::fix(v);
    ///
    /// let index = v.check_index(1).unwrap();
    ///
    /// assert_eq!(v.get(index), &2);
    /// ```
    pub fn check_index(&self, index: usize) -> Option<Index<Name>> {
        if self.len() <= index {
            None
        } else {
            Some(Index {
                index,
                _phantom: PhantomData
            })
        }
    }
    
    /// Perform an index bounds check on a whole range of indices. This is the only way to create a
    /// ``CheckedRange``, which will share the same ``Name`` as the ``FixedVec``. The created
    /// ``CheckedRange`` can be used to create valid ``Index``'s for the ``FixedVec``.
    ///
    /// # Example
    ///
    /// ```
    /// # use fixed_vec::*;
    /// let v = vec![0u32; 50];
    /// let v = name!(v);
    /// let mut v = FixedVec::fix(v);
    ///
    /// let range = 0usize..20;
    /// let range = v.check_range(range).unwrap();
    ///
    /// for i in range {
    ///     *v.get_mut(i) += 1;
    /// }
    /// ```
    pub fn check_range(&self, range: Range<usize>) -> Option<CheckedRange<Name>> {
        if range.end >= self.len() {
            None
        } else {
            Some(CheckedRange {
                range,
                _phantom: PhantomData,
            })
        }
    }
    
    /// Get an element of the ``FixedVec`` without bounds checking. This is safe because the
    /// ``Index`` is guaranteed to have been created from one of the methods of this ``FixedVec``,
    /// at which point the index was checked to have been in bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// # use fixed_vec::*;
    /// let v = vec![1, 2, 3];
    /// let v = name!(v);
    /// let v = FixedVec::fix(v);
    ///
    /// let index = v.check_index(1).unwrap();
    ///
    /// assert_eq!(v.get(index), &2);
    /// ```
    /// 
    /// The following examples don't compile, since they use an ``Index`` for the wrong
    /// ``FixedVec``:
    ///
    /// ```compile_fail
    /// # use fixed_vec::*;
    /// let v = vec![1, 2, 3];
    /// let v = name!(v);
    /// let v = FixedVec::fix(v);
    ///
    /// let index = v.check_index(1).unwrap();
    ///
    /// let v2 = vec![];
    /// let v2 = name!(v);
    /// let v2 = FixedVec::fix(v2);
    ///
    /// println!("{}", v2.get(index));
    /// ```
    ///
    /// ```compile_fail
    /// # use fixed_vec::*;
    /// let v = vec![1, 2, 3];
    /// let v = name!(v);
    /// let v = FixedVec::fix(v);
    ///
    /// let index = v.check_index(1).unwrap();
    ///
    /// let mut v = v.unfix();
    /// v = vec![];
    ///
    /// let v = name!(v);
    /// let v = FixedVec::fix(v);
    ///
    /// println!("{}", v.get(index));
    /// ```
    #[inline(always)]
    pub fn get(&self, index: Index<Name>) -> &A {
        unsafe {
            self.inner.unname_ref().get_unchecked(index.index)
        }
    }
    
    /// Get a mutable reference to an element of the ``FixedVec``. This is safe for the same
    /// reasons as ``get()``, in addition to the fact that mutating a single element of a vector
    /// does not change it's length.
    #[inline(always)]
    pub fn get_mut(&mut self, index: Index<Name>) -> &mut A {
        unsafe {
            // We can take unname_ref_mut since
            // changing a single index will not
            // violate the length invariant
            self.inner.unname_ref_mut().get_unchecked_mut(index.index)
        }
    }

    // Implementation of other normal Vec methods that preserve size
    
    /// Reserves capacity for at least ``additional`` more elements to be inserted in the given
    /// ``FixedVec``. See [std docs](std::vec::Vec::reserve) for more information.
    pub fn reserve(&mut self, additional: usize) {
        unsafe {
            self.inner.unname_ref_mut().reserve(additional);
        }
    }
    
    /// Reserves the minimum capacity for exactly ``additional`` more elements to be inserted in
    /// the given ``Vec<T>``. See [std docs](std::vec::Vec::reserve_exact) for more information.
    pub fn reserve_exact(&mut self, additional: usize) {
        unsafe {
            self.inner.unname_ref_mut().reserve_exact(additional);
        }
    }

    /// Shrinks the capacity of the vector as much as possible. See [std
    /// docs](std::vec::Vec::shrink_to_fit) for more information.
    pub fn shrink_to_fit(&mut self) {
        unsafe {
            self.inner.unname_ref_mut().shrink_to_fit();
        }
    }

    /// Extracts a mutable slice of the entire vector. See [std docs](std::vec::Vec::as_mut_slice)
    /// for more information.
    pub fn as_mut_slice(&mut self) -> &mut [A] {
        unsafe {
            self.inner.unname_ref_mut().as_mut_slice()
        }
    }

    /// Returns an unsafe mutable pointer to the vector's buffer. See [std
    /// docs](std::vec::Vec::as_mut_ptr) for more information.
    pub fn as_mut_ptr(&mut self) -> *mut A {
        unsafe {
            self.inner.unname_ref_mut().as_mut_ptr()
        }
    }

    /// Inserts an element at position ``index`` within the vector, shifting all elements after it
    /// to the right. See [std docs](std::vec::Vec::insert) for more information.
    pub fn insert(&mut self, index: usize, element: A) {
        unsafe {
            self.inner.unname_ref_mut().insert(index, element)
        }
    }

    /// Appends an element to the back of a collection. See [std docs](std::vec::Vec::push) for
    /// more information.
    pub fn push(&mut self, value: A) {
        unsafe {
            self.inner.unname_ref_mut().push(value);
        }
    }

    /// Moves all the elements of ``other`` into ``Self``, leaving ``other`` empty. See [std
    /// docs](std::vec::Vec::append) for more information.
    pub fn append(&mut self, other: &mut Vec<A>) {
        unsafe {
            self.inner.unname_ref_mut().append(other);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use type_name_value::name;

    #[test]
    fn it_works() {
        let v = vec![1, 2, 3];

        let v = name!(v);

        let v = FixedVec::fix(v);

        let index = v.check_index(1).unwrap();

        assert_eq!(v.get(index), &2);
        assert_eq!(v.check_index(3), None);

        // Doesn't compile:
        // let v2: Vec<usize> = vec![];
        //
        // let v2 = name!(v2);
        //
        // let v2 = FixedVec::fix(v2);
        //
        // println!("{}", v2.get(index));
    }

    #[test]
    fn loop_iter() {
        let v = vec![1, 2, 3];

        let v = name!(v);

        let mut v = FixedVec::fix(v);

        let index_a = v.check_index(0).unwrap();
        let index_b = v.check_index(1).unwrap();
        let index_c = v.check_index(2).unwrap();

        for _ in 0..10 {
            *v.get_mut(index_a) += 1;
            *v.get_mut(index_b) += 2;
            *v.get_mut(index_c) += 3;
        }

        assert_eq!(v.unfix(), vec![11, 22, 33]);
    }

    #[test]
    fn checked_range() {
        let v = vec![0u32; 50];
        let v = name!(v);
        let mut v = FixedVec::fix(v);

        let range = 0usize..20;
        let range = v.check_range(range).unwrap();

        for _ in 0..10 {
            for i in range.clone() {
                *v.get_mut(i) += 1;
            }
        }

        // The following won't compile:
        // let v2 = vec![];
        // let v2 = name!(v2);
        // let mut v2 = FixedVec::fix(v2);
        // for i in range {
        //     *v2.get_mut(i) += 1;
        // }
    }
}
