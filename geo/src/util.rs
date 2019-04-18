//! This module contains an implementation of
//! [QuickSelect](https://en.wikipedia.org/wiki/Quickselect) to provide O(n)
//! median search. It's an in place selection algorithm that also has the nice
//! side effect of partitioning the data according to the kth element.

use std;
use std::cmp::Ordering;

/// Sort the given slice until it finds the kth smallest element. Return None if
/// k is out of bounds.
pub fn ksmallest<T: Ord + std::fmt::Debug>(elems: &mut [T], k: usize) -> Option<&T> {
    ksmallest_by(elems, k, |l, r| l.cmp(r))
}

/// Sort the given slice until it finds the kth smallest element according to
/// the given key function. Return None if k is out of bounds.
pub fn ksmallest_by_key<T, F, K>(elems: &mut [T], k: usize, mut f: F) -> Option<&T>
where
    F: FnMut(&T) -> K,
    K: Ord,
{
    ksmallest_by(elems, k, |l, r| f(l).cmp(&f(r)))
}

/// Sort the given slice until it finds the kth smallest element according to
/// the function that returns the ordering between elements. Return None if k is
/// out of bounds.
pub fn ksmallest_by<T, F>(elems: &mut [T], k: usize, mut f: F) -> Option<&T>
where
    F: FnMut(&T, &T) -> Ordering,
{
    if k >= elems.len() {
        return None;
    }

    let mut left = 0;
    let mut right = elems.len() - 1;

    loop {
        let pivot = partition_by(elems, &mut f, left, right, right);

        match pivot.cmp(&k) {
            Ordering::Equal => return Some(&elems[pivot]),
            Ordering::Less => left = pivot + 1,
            Ordering::Greater => right = pivot - 1,
        };
    }
}

fn partition_by<T, F>(elems: &mut [T], f: &mut F, left: usize, right: usize, pivot: usize) -> usize
where
    F: FnMut(&T, &T) -> Ordering,
{
    elems.swap(right, pivot);
    let pivot = right;

    let mut store_index = left;

    for i in left..right {
        if let Ordering::Less = f(&elems[i], &elems[pivot]) {
            elems.swap(store_index, i);
            store_index += 1;
        }
    }

    elems.swap(store_index, pivot);

    store_index
}

#[cfg(test)]
mod test {
    use super::{ksmallest, ksmallest_by};

    use proptest::proptest;

    proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(100))]
        #[test]
        fn prop_k_smallest_is_the_same_as_kth_elem_of_sorted_array(
            v in proptest::collection::vec(0_u8..255, 0..100)
        ) {
            let mut sorted = v.clone();
            sorted.sort();

            for k in 0..v.len() {
                assert_eq!(ksmallest(&mut v.clone(), k), Some(&sorted[k]));
            }
        }
    }

    proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(100))]
        #[test]
        fn prop_k_smallest_sorts_the_input_if_called_with_k_up_to_len(
            mut v in proptest::collection::vec(0_u8..255, 1..100)
        ) {
            let mut sorted = v.clone();
            sorted.sort();
            sorted.reverse();

            for k in 0..sorted.len() {
                ksmallest_by(&mut v, k, |l, r| r.cmp(l)).unwrap();
            }

            assert_eq!(sorted, v);
        }
    }
}
