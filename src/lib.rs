
use std::marker::PhantomData;
use std::ops::Deref;
use std::ops::Range;

#[macro_use]
extern crate derivative;

extern crate type_name_value;

use type_name_value::Named;

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
    pub fn fix(val: Named<Vec<A>, Name>) -> Self {
        FixedVec {
            inner: val,
            _phantom: PhantomData,
        }
    }

    pub fn unfix(self) -> Vec<A> {
        self.inner.unname()
    }

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
    
    #[inline(always)]
    pub fn get(&self, index: Index<Name>) -> &A {
        unsafe {
            self.inner.unname_ref().get_unchecked(index.index)
        }
    }

    #[inline(always)]
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
    use type_name_value::name;

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
