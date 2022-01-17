use std::{
    fmt::Debug,
    iter::Zip,
    ops::{Index, IndexMut, Range},
};

use smallvec::{smallvec, SmallVec};

use crate::backend::register_allocation::RegisterInterface;

#[derive(Clone, Debug)]
pub struct LiveRange<R: RegisterInterface> {
    pub vregs: SmallVec<[u32; 4]>,
    pub spill_cost: f32,
    pub precolor: Option<R>,
}

impl<R: RegisterInterface> LiveRange<R> {
    pub fn new(vreg: u32) -> LiveRange<R> {
        LiveRange {
            vregs: smallvec![vreg],
            spill_cost: 0.,
            precolor: None,
        }
    }
    pub fn reg(reg: R) -> LiveRange<R> {
        LiveRange {
            vregs: SmallVec::new(),
            spill_cost: f32::MAX,
            precolor: Some(reg),
        }
    }

    pub fn _is_vreg(&self) -> bool {
        !self.vregs.is_empty()
    }
}

#[derive(Clone, Debug)]
pub struct IntervalVector<T, U>
where
    T: Copy + Debug,
    U: Clone + Debug,
{
    pub range: SmallVec<[Range<T>; 2]>,
    pub item: SmallVec<[U; 4]>,
}

impl<T, U> IntervalVector<T, U>
where
    T: Copy + Debug + PartialOrd,
    U: Clone + Debug,
{
    pub fn new(range: Range<T>, item: U) -> IntervalVector<T, U> {
        IntervalVector {
            range: smallvec![range],
            item: smallvec![item],
        }
    }

    pub fn empty() -> IntervalVector<T, U> {
        IntervalVector {
            range: SmallVec::new(),
            item: SmallVec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        assert_eq!(self.range.is_empty(), self.item.is_empty());
        self.range.is_empty()
    }

    pub fn insert(&mut self, range: Range<T>, item: U) {
        let mut split = None;
        for (ran, it) in self.range.iter_mut().zip(self.item.iter()) {
            // The behaviour where the new range full covers the old range is covered by the normal cases for semi-overlap
            // The old range will functionaly be deleted, because the range will contain no items
            if ran.contains(&range.start) || ran.contains(&range.end) {
                split = Some((range.end..range.end, it.clone()))
            } else {
                if ran.contains(&range.start) {
                    ran.end = range.start;
                } else if range.contains(&ran.start) {
                    ran.start = range.end;
                }
                if ran.contains(&range.end) {
                    ran.start = range.end;
                } else if range.contains(&ran.end) {
                    ran.end = range.start;
                }
            }
        }
        if let Some((range, item)) = split {
            self.range.push(range);
            self.item.push(item);
        }
        self.range.push(range);
        self.item.push(item);
    }

    pub fn get(&self, point: T) -> Option<U> {
        for (range, item) in self.range.iter().zip(self.item.iter()) {
            if range.contains(&point) {
                return Some(item.clone());
            }
        }
        None
    }
}

impl<T, U> Index<T> for IntervalVector<T, U>
where
    T: Copy + Debug + PartialOrd,
    U: Clone + Debug,
{
    type Output = U;
    fn index<'a>(&'a self, index: T) -> &'a Self::Output {
        for (range, item) in self.range.iter().zip(self.item.iter()) {
            if range.contains(&index) {
                return item;
            }
        }
        unreachable!()
    }
}

impl<T, U> IndexMut<T> for IntervalVector<T, U>
where
    T: Copy + Debug + PartialOrd,
    U: Clone + Debug,
{
    fn index_mut(&mut self, index: T) -> &mut Self::Output {
        for (range, item) in self.range.iter().zip(self.item.iter_mut()) {
            if range.contains(&index) {
                return item;
            }
        }
        unreachable!()
    }
}

impl<'a, T, U> IntoIterator for &'a IntervalVector<T, U>
where
    T: 'a + Copy + Debug + PartialOrd,
    U: 'a + Clone + Debug,
{
    type IntoIter = Zip<std::slice::Iter<'a, Range<T>>, std::slice::Iter<'a, U>>;
    type Item = <<&'a IntervalVector<T, U> as IntoIterator>::IntoIter as Iterator>::Item;
    fn into_iter(self) -> Self::IntoIter {
        let it = self.range.iter().zip(self.item.iter());
        it
    }
}
