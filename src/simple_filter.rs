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
use std::iter::FromIterator;
use crate::traits::set::*;
use crate::traits::filter::*;
use std::rc::Rc;
use std::marker::PhantomData;

#[derive(Debug, Clone, PartialEq)]
/// A Bloom filter with underlying set `B` and [`BuildHasher`] type
/// `S`, the `BuildHasher`s being held in a collection of type
/// `V`. The supported operations are based on the traits implemented
/// by `B`.
pub struct SimpleBloomFilter<B, S = RandomState, V = Rc<[S]>>
where
    V: AsRef<[S]>,
{
    hashers: V,
    set: B,
    _phantom: PhantomData<S>
}

impl<B, S, V> SimpleBloomFilter<B, S, V>
where
    B: BloomSet,
    S: BuildHasher,
    V: AsRef<[S]>,
{
    /// Creates a new `SimpleBloomFilter` with a specified number of counters
    /// and [`BuildHasher`]s. The `BuildHasher`s will be initialized by
    /// [`default`](Default::default).
    pub fn new(n_hashers: usize, n_counters: usize) -> Self
    where
        S: Default,
        V: FromIterator<S>,
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
    pub fn with_hashers(hashers: V, n_counters: usize) -> Self {
        debug_assert!(hashers.as_ref().len() > 0);
        SimpleBloomFilter {
            hashers: hashers,
            set: B::new(n_counters),
            _phantom: PhantomData
        }
    }

    /// Returns the hashers and bit set of the filter.
    pub fn into_inner(self) -> (V, B) {
        (self.hashers, self.set)
    }

    pub fn hashers(&self) -> &V {
        &self.hashers
    }

    fn hash_indices<'a, T: Hash>(
        hashers: &'a V,
        set_size: usize,
        val: &'a T,
    ) -> impl Iterator<Item = usize> + 'a
    where S: 'a {
        hashers.as_ref().iter().map(move |b| {
            let mut h = b.build_hasher();
            val.hash(&mut h);
            h.finish() as usize % set_size
        })
    }
}

impl<B, S, V> BloomFilter for SimpleBloomFilter<B, S, V>
where
    B: BloomSet,
    S: BuildHasher,
    V: AsRef<[S]>,
{
    type Set = B;
    type Hasher = S;

    fn counters(&self) -> &B {
        return &self.set;
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

impl<B, S, V> BloomFilterDelete for SimpleBloomFilter<B, S, V>
where
    B: BloomSetDelete,
    S: BuildHasher,
    V: AsRef<[S]>,
{
    fn remove<T: Hash>(&mut self, val: &T) {
        for i in Self::hash_indices(&self.hashers, self.set.size(), val) {
            self.set.decrement(i);
        }
    }
}

impl<B, S, V> BinaryBloomFilter for SimpleBloomFilter<B, S, V>
where
    B: BinaryBloomSet,
    S: BuildHasher,
    V: AsRef<[S]>,
{
    fn union<Other>(&mut self, other: &Other)
    where
        Other: BinaryBloomFilter<Set = Self::Set, Hasher = Self::Hasher>
    {
        self.set.union(&other.counters());
    }

    fn intersect<Other>(&mut self, other: &Other)
    where
        Other: BinaryBloomFilter<Set = Self::Set, Hasher = Self::Hasher>
    {
        self.set.intersect(&other.counters());
    }
}

impl<B, S, V> SpectralBloomFilter for SimpleBloomFilter<B, S, V>
where
    B: SpectralBloomSet,
    B::Count: Ord,
    S: BuildHasher,
    V: AsRef<[S]>,
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
