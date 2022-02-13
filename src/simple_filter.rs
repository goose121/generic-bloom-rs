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

use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hash, Hasher};
use std::iter::IntoIterator;
use crate::traits::set::*;
use crate::traits::filter::*;

#[derive(Debug, Clone, PartialEq)]
/// A Bloom filter with underlying set `B` and [`BuildHasher`] type
/// `S`. The supported operations are based on the traits implemented
/// by `B`.
pub struct SimpleBloomFilter<B, S = RandomState> {
    hashers: Vec<S>,
    set: B,
}

impl<B, S> SimpleBloomFilter<B, S>
where
    B: BloomSet,
    S: BuildHasher,
{
    /// Creates a new `SimpleBloomFilter` with a specified number of counters
    /// and [`BuildHasher`]s. The `BuildHasher`s will be initialized by
    /// [`default`](Default::default).
    pub fn new(n_hashers: usize, n_counters: usize) -> SimpleBloomFilter<B, S>
    where
        S: Default,
    {
        SimpleBloomFilter::with_hashers(
            std::iter::repeat_with(|| S::default())
                .take(n_hashers)
                .collect(),
            n_counters,
        )
    }

    /// Creates a new `SimpleBloomFilter` with specified `BuildHasher`s and a
    /// specified number of counters.
    pub fn with_hashers(hashers: Vec<S>, n_counters: usize) -> SimpleBloomFilter<B, S> {
        debug_assert!(hashers.len() > 0);
        SimpleBloomFilter {
            hashers: hashers.into_iter().collect(),
            set: B::new(n_counters),
        }
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
}

impl<B, S> BloomFilter for SimpleBloomFilter<B, S>
where
    B: BloomSet,
    S: BuildHasher,
{
    type Set = B;
    type Hasher = S;

    fn hashers(&self) -> &[S] {
        &*self.hashers
    }

    fn insert<T: Hash>(&mut self, val: &T) {
        for i in Self::hash_indices(&self.hashers, self.set.size(), val) {
            self.set.increment(i);
        }
    }

    fn contains<T: Hash>(&self, val: &T) -> bool {
        for i in Self::hash_indices(&self.hashers, self.set.size(), val) {
            if !self.set.query(i) {
                return false;
            }
        }

        true
    }

    fn clear(&mut self) {
        self.set.clear()
    }
}

impl<B, S> BloomFilterDelete for SimpleBloomFilter<B, S>
where
    B: BloomSetDelete,
    S: BuildHasher,
{
    fn remove<T: Hash>(&mut self, val: &T) {
        for i in Self::hash_indices(&self.hashers, self.set.size(), val) {
            self.set.decrement(i);
        }
    }
}

impl<B, S> BinaryBloomFilter for SimpleBloomFilter<B, S>
where
    B: BinaryBloomSet,
    S: BuildHasher,
{
    fn union(&mut self, other: &SimpleBloomFilter<B, S>) {
        self.set.union(&other.set);
    }

    fn intersect(&mut self, other: &SimpleBloomFilter<B, S>) {
        self.set.intersect(&other.set);
    }
}

impl<B, S> SpectralBloomFilter for SimpleBloomFilter<B, S>
where
    B: SpectralBloomSet,
    B::Count: Ord,
    S: BuildHasher,
{
    fn contains_more_than<T: Hash>(
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

    fn find_count<T: Hash>(&self, val: &T) -> &<B as SpectralBloomSet>::Count {
        Self::hash_indices(&self.hashers, self.set.size(), val)
            .map(|i| self.set.query_count(i))
            .min()
            .unwrap()
    }
}
