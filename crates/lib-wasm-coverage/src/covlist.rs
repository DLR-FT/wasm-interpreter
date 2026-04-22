use alloc::{
    collections::{BTreeMap, btree_map::IntoValues},
    vec::Vec,
};
use core::ops::{Range, RangeTo, RangeToInclusive};
use std::collections::btree_map::Values;

/// A helper data structure that keeps an ordered list of disjoint ranges ordered per their starting points
#[derive(Default, Debug)]
pub struct CovList {
    tree: BTreeMap<u64, Range<u64>>,
}

impl CovList {
    pub fn new() -> Self {
        CovList::default()
    }

    pub fn insert(&mut self, range: Range<u64>) {
        if range.is_empty() {
            return;
        }

        // early exit if this range is inserted before
        if let Some(r) = self.tree.get(&range.start)
            && *r == range
        {
            return;
        }

        // let the range inserted be [s_i := range.start, e_i := range.end).
        let mut range_to_insert = range.clone();

        // get the previously inserted range [s_l, e_l) with the largest s_l < s_i
        let left_side = self.tree.range(RangeTo { end: range.start }).next_back();

        // if it exists and e_l >= s_i, set s_i := s_l
        // Now [s_i, e_i) does not overlap from left with any range, that is, for any s_k with s_k < s_i
        // s_k < e_k < s_i holds.
        if let Some((_, Range { start, end })) = left_side
            && *end >= range_to_insert.start
        {
            range_to_insert.start = *start;
        }

        // get the previously inserted range [s_r, e_r) with the largest s_r <= e_i
        let right_side = self.tree.range(RangeTo { end: range.end }).next_back();

        // if it exists and e_r > e_i, set e_i := e_r.
        if let Some((_, Range { start: _, end })) = right_side
            && *end > range_to_insert.end
        {
            range_to_insert.end = *end;
        }

        // remove all ranges [s_k, e_k) where s_i <= s_k < e_i.
        // this will only remove the ranges that are within [s_i e_i).
        // That is it must be that for all s_k with s_i <= s_k < e_i, s_i <= s_k < e_k <= e_i.
        // Since s_i <= s_k < e_i applies (by assumption)
        // And s_i <= s_k < e_k also applies (s_k >= e_k implies an empty range, which is not inserted),
        // the only case this would not be true is when there is some e_k' > e_i.
        // This cannot happen, since if this e_k' existed, it would be the part of the range [s_k',e_k')
        // where s_k' is the largest among all s_k (by the invariant of this covlist holding disjoint ordered ranges).
        // This means [s_k, e_k') = [s_r, e_r) if it exists,
        // and we already extend our range when e_r > e_i by setting e_i := e_r.
        let remove_keys: Vec<_> = self
            .tree
            .range(range_to_insert.clone())
            .map(|(k, _)| *k)
            .collect();
        for k in remove_keys {
            self.tree.remove(&k);
        }
        let start = range_to_insert.start;

        // insert [s_i, e_i).
        // this will not overlap with any range. To see, for all s_k
        // s_k < s_i implies s_k < e_k < s_i by left_side extension, so no overlap.
        // s_i <= s_k < e_i were removed by the operation above.
        // e_i <= s_k already does not overlap.
        self.tree.insert(start, range_to_insert);
    }

    pub fn contains(&self, value: u64) -> bool {
        let range = self.tree.range(RangeToInclusive { end: value }).next_back();
        range.is_some_and(|(_, range)|range.contains(&value))
    }
}

pub struct CovListIterator(IntoValues<u64, Range<u64>>);

impl Iterator for CovListIterator {
    type Item = Range<u64>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

pub struct CovListRefIterator<'a>(Values<'a, u64, Range<u64>>);

impl<'a> Iterator for CovListRefIterator<'a> {
    type Item = Range<u64>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().cloned()
    }
}

impl IntoIterator for CovList {
    type Item = Range<u64>;

    type IntoIter = CovListIterator;

    fn into_iter(self) -> Self::IntoIter {
        CovListIterator(self.tree.into_values())
    }
}

impl<'a> IntoIterator for &'a CovList {
    type Item = Range<u64>;

    type IntoIter = CovListRefIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        CovListRefIterator(self.tree.values())
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;

    #[test]
    fn test1() {
        let mut covlist = CovList::new();
        covlist.insert(2..5);
        let nums: Vec<_> = covlist.into_iter().collect();
        assert_eq!(nums, vec![2..5]);
    }

    #[test]
    fn test2() {
        let mut covlist = CovList::new();
        covlist.insert(2..5);
        covlist.insert(5..7);
        let nums: Vec<_> = covlist.into_iter().collect();
        assert_eq!(nums, vec![2..7]);
    }

    #[test]
    fn test3() {
        let mut covlist = CovList::new();
        covlist.insert(2..5);
        covlist.insert(10..15);
        covlist.insert(13..17);
        let nums: Vec<_> = covlist.into_iter().collect();
        assert_eq!(nums, vec![2..5, 10..17]);
    }

    #[test]
    fn test4() {
        let mut covlist = CovList::new();
        covlist.insert(12..14);
        covlist.insert(10..17);
        let nums: Vec<_> = covlist.into_iter().collect();
        assert_eq!(nums, vec![10..17]);
    }

    #[test]
    fn test5() {
        let mut covlist = CovList::new();
        covlist.insert(10..11);
        covlist.insert(11..12);
        let nums: Vec<_> = covlist.into_iter().collect();
        assert_eq!(nums, vec![10..12]);
    }

    #[test]
    fn test6() {
        let mut covlist = CovList::new();
        covlist.insert(10..10);
        covlist.insert(11..9);
        let nums: Vec<_> = covlist.into_iter().collect();
        assert_eq!(nums, vec![]);
    }

    #[test]
    fn test7() {
        let mut covlist = CovList::new();

        covlist.insert(11..12);
        covlist.insert(11..11);
        let nums: Vec<_> = covlist.into_iter().collect();
        assert_eq!(nums, vec![11..12]);
    }

    #[test]
    fn test8() {
        let mut covlist = CovList::new();

        covlist.insert(11..13);
        covlist.insert(11..13);
        let nums: Vec<_> = covlist.into_iter().collect();
        assert_eq!(nums, vec![11..13]);
    }

    #[test]
    fn test9() {
        let mut covlist = CovList::new();
        covlist.insert(2..5);
        covlist.insert(10..15);
        covlist.insert(13..17);
        assert!(!covlist.contains(1));
        assert!(!covlist.contains(5));
        assert!(!covlist.contains(7));
        assert!(covlist.contains(2));
        assert!(covlist.contains(4));
        assert!(covlist.contains(10));
        assert!(covlist.contains(15));
    }
}
