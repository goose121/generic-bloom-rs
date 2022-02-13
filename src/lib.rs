// This file is part of generic-bloom.
//
// generic-bloom is free software: you can redistribute it and/or
// modify it under the terms of the GNU Affero General Public License
// as published by the Free Software Foundation, either version 3 of
// the License, or (at your option) any later version.
//
// generic-bloom is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// Affero General Public License for more details.  You should have
// received a copy of the GNU Affero General Public License along with
// generic-bloom. If not, see <https://www.gnu.org/licenses/>.

//! This crate provides a [`BloomFilter`] type which can be
//! parameterized by different types of storage to obtain traditional
//! binary Bloom filters, counting Bloom filters, and spectral Bloom
//! filters. For basic usage, see the documentation for
//! [`BloomFilter`].

use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hash, Hasher};
use std::iter::IntoIterator;

pub mod traits;
use traits::*;

#[derive(Debug, Clone, PartialEq)]
/// A bloom filter with underlying set `B` and [`BuildHasher`] type
/// `S`. The supported operations are based on the traits implemented
/// by `B`.
pub struct BloomFilter<B, S = RandomState> {
    hashers: Vec<S>,
    set: B,
}

impl<B, S> BloomFilter<B, S>
where
    B: BloomSet,
    S: BuildHasher,
{
    /// Creates a new `BloomFilter` with a specified number of counters
    /// and [`BuildHasher`]s. The `BuildHasher`s will be initialized by
    /// [`default`](Default::default).
    pub fn new(n_hashers: usize, n_counters: usize) -> BloomFilter<B, S>
    where
        S: Default,
    {
        BloomFilter::with_hashers(
            std::iter::repeat_with(|| S::default())
                .take(n_hashers)
                .collect(),
            n_counters,
        )
    }

    /// Creates a new `BloomFilter` with specified `BuildHasher`s and a
    /// specified number of counters.
    pub fn with_hashers(hashers: Vec<S>, n_counters: usize) -> BloomFilter<B, S> {
        debug_assert!(hashers.len() > 0);
        BloomFilter {
            hashers: hashers.into_iter().collect(),
            set: B::new(n_counters),
        }
    }

    /// Returns the `BuildHasher`s used by this `BloomFilter`.
    pub fn hashers(&self) -> &[S] {
        &*self.hashers
    }

    fn hash_indices<'a, T: Hash>(
        hashers: &'a Vec<S>,
        set_size: usize,
        val: &'a T,
    ) -> impl Iterator<Item = usize> + 'a {
        hashers.iter().map(move |b| {
            let mut h = b.build_hasher();
            val.hash(&mut h);
            h.finish() as usize % set_size
        })
    }

    /// Inserts `val` into the set.
    pub fn insert<T: Hash>(&mut self, val: &T) {
        for i in Self::hash_indices(&self.hashers, self.set.size(), val) {
            self.set.increment(i);
        }
    }

    /// Checks whether the set contains `val`.
    pub fn contains<T: Hash>(&self, val: &T) -> bool {
        for i in Self::hash_indices(&self.hashers, self.set.size(), val) {
            if !self.set.query(i) {
                return false;
            }
        }

        true
    }

    /// Clears all values from the set.
    pub fn clear(&mut self) {
        self.set.clear()
    }
}

impl<B, S> BloomFilter<B, S>
where
    B: BloomSetDelete,
    S: BuildHasher,
{
    /// Removes `val` from the set. **If `val` was not previously
    /// added to the set, this may cause false negatives in future
    /// queries.**
    pub fn remove<T: Hash>(&mut self, val: &T) {
        for i in Self::hash_indices(&self.hashers, self.set.size(), val) {
            self.set.decrement(i);
        }
    }
}

impl<B, S> BloomFilter<B, S>
where
    B: BinaryBloomSet,
    S: BuildHasher,
{
    /// Inserts all values from `other` into `self`.
    pub fn union(&mut self, other: &BloomFilter<B, S>) {
        self.set.union(&other.set);
    }

    /// Keeps only values in `self` which are also in `other`.
    pub fn intersect(&mut self, other: &BloomFilter<B, S>) {
        self.set.intersect(&other.set);
    }
}

impl<B, S> BloomFilter<B, S>
where
    B: SpectralBloomSet,
    B::Count: Ord,
    S: BuildHasher,
{
    /// Tests whether the set contains `val` more than `count` times.
    pub fn contains_more_than<T: Hash>(
        &self,
        val: &T,
        count: &<B as SpectralBloomSet>::Count,
    ) -> bool {
        for i in Self::hash_indices(&self.hashers, self.set.size(), val) {
            if *self.set.query_count(i) <= *count {
                return false;
            }
        }

        true
    }

    /// Returns an estimate of the number of times the set contains `val`.
    pub fn find_count<T: Hash>(&self, val: &T) -> &<B as SpectralBloomSet>::Count {
        Self::hash_indices(&self.hashers, self.set.size(), val)
            .map(|i| self.set.query_count(i))
            .min()
            .unwrap()
    }
}

// TODO: improve checks ensuring elements aren't present (maybe
// statistics?)

#[cfg(test)]
mod tests {
    use crate::*;
    use bitvec::boxed::BitBox;
    use bitvec::order::Lsb0;

    #[test]
    fn insert_contains() {
        let mut f: BloomFilter<BitBox<usize, Lsb0>> = BloomFilter::new(10, 20);
        f.insert(&48);
        f.insert(&32);
        assert!(f.contains(&48));
        assert!(f.contains(&32));
        assert!(!f.contains(&39));
    }

    #[test]
    fn union() {
        let mut f1: BloomFilter<BitBox<usize, Lsb0>> = BloomFilter::new(10, 20);
        f1.insert(&48);
        f1.insert(&32);
        let mut f2: BloomFilter<BitBox<usize, Lsb0>> =
            BloomFilter::with_hashers(f1.hashers().to_vec(), 20);
        f2.insert(&39);
        assert!(f1.contains(&48));
        assert!(f1.contains(&32));
        assert!(!f1.contains(&39));
        assert!(f2.contains(&39));
        f1.union(&f2);
        assert!(f1.contains(&48));
        assert!(f1.contains(&32));
        assert!(f1.contains(&39));
    }

    #[test]
    fn intersect() {
        let mut f1: BloomFilter<BitBox<usize, Lsb0>> = BloomFilter::new(10, 20);
        f1.insert(&48);
        f1.insert(&32);
        let mut f2: BloomFilter<BitBox<usize, Lsb0>> =
            BloomFilter::with_hashers(f1.hashers().to_vec(), 20);
        f2.insert(&32);
        f2.insert(&39);
        assert!(f1.contains(&48));
        assert!(f1.contains(&32));
        assert!(!f1.contains(&39));
        assert!(f2.contains(&39));
        f1.intersect(&f2);
        assert!(!f1.contains(&48));
        assert!(f1.contains(&32));
        assert!(!f1.contains(&39));
    }

    #[test]
    fn delete() {
        let mut f: BloomFilter<Box<[u8]>> = BloomFilter::new(10, 20);
        for x in 0..30 {
            f.insert(&x);
        }
        let contains_30 = f.contains(&30);
        f.insert(&30);
        assert!(f.contains(&30));
        f.remove(&30);
        assert!(f.contains(&30) == contains_30);
    }
}
