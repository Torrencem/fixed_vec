
use std::marker::PhantomData;
use std::ops::Deref;

#[macro_use]
extern crate derivative;

// TODO: There should be a way to make a temporary name with &mut, since
// we still have exclusive access to a value, even if we don't have ownership
pub struct Named<T, Name> {
    inner: T,
    _phantom: PhantomData<Name>,
}

/// Safety:
/// Must make sure Name is not used as the name for any other
/// value of type Named<T, Name>
pub unsafe fn name<Name, T>(val: T) -> Named<T, Name> {
    Named {
        inner: val,
        _phantom: PhantomData,
    }
}

impl<T, Name> Named<T, Name> {
    pub fn unname(self) -> T {
        self.inner
    }

    pub fn unname_ref(&self) -> &T {
        &self.inner
    }
    
    /// Safety:
    /// Must uphold whatever invariants the Named protects
    pub unsafe fn unname_ref_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

#[macro_export]
macro_rules! name {
    ($val:expr) => {{
        struct UniqueName {};

        unsafe {
            // Nothing else is named $name because we just
            // defined $name!
            name::<UniqueName, _>($val)
        }
    }}
}

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

#[derive(Derivative)]
#[derivative(Clone(bound=""), Copy(bound=""))]
pub struct Index<Name> {
    index: usize,
    _phantom: PhantomData<Name>,
}

impl<A, Name> FixedVec<A, Name> {
    pub fn fix(val: Named<Vec<A>, Name>) -> Self {
        FixedVec {
            inner: val,
            _phantom: PhantomData,
        }
    }

    pub fn check_index(&self, index: usize) -> Option<Index<Name>> {
        if self.inner.unname_ref().len() <= index {
            None
        } else {
            Some(Index {
                index,
                _phantom: PhantomData
            })
        }
    }

    pub fn get(&self, index: Index<Name>) -> &A {
        unsafe {
            self.inner.unname_ref().get_unchecked(index.index)
        }
    }

    pub fn get_mut(&mut self, index: Index<Name>) -> &mut A {
        unsafe {
            // We can take unname_ref_mut since
            // changing a single index will not
            // violate the length invariant
            self.inner.unname_ref_mut().get_unchecked_mut(index.index)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let v = vec![1, 2, 3];

        let v = name!(v);

        let v = FixedVec::fix(v);

        let index = v.check_index(1).unwrap();

        println!("{}", v.get(index));

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

        println!("{:?}", v.deref());
    }
}
