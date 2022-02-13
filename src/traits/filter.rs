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

//! Traits for types which act as Bloom filters.

use std::hash::{Hash, BuildHasher};
use crate::traits::set::*;

/// Supertrait for all types which act as Bloom filters.
pub trait BloomFilter {
    type Set: BloomSet;
    type Hasher: BuildHasher;

    /// Gets a reference to the underlying counters used by this
    /// `BloomFilter`.
    fn counters(&self) -> &Self::Set;

    /// Inserts `val` into the set.
    fn insert<T: Hash>(&mut self, val: &T);

    /// Checks whether the set contains `val`.
    fn contains<T: Hash>(&self, val: &T) -> bool;

    /// Clears all values from the set.
    fn clear(&mut self);
}

/// Trait for types which act as Bloom filters and support deletion.
pub trait BloomFilterDelete: BloomFilter
where
    Self::Set: BloomSetDelete,
{
    /// Removes `val` from the set. **If `val` was not previously
    /// added to the set, this may cause false negatives in future
    /// queries.**
    ///
    /// # Example
    /// ```
    /// use generic_bloom::{BloomFilter, BloomFilterDelete, SimpleBloomFilter};
    ///
    /// let mut f: SimpleBloomFilter<Box<[u8]>> = SimpleBloomFilter::new(10, 20);
    /// for x in 0..30 {
    ///     f.insert(&x);
    /// }
    ///
    /// let contains_30 = f.contains(&30);
    /// f.insert(&30);
    ///
    /// assert!(f.contains(&30));
    /// f.remove(&30);
    ///
    /// // Only check if the result is the same as it was
    /// // before, in case it was a false positive.
    /// assert!(f.contains(&30) == contains_30);
    fn remove<T: Hash>(&mut self, val: &T);
}

/// Trait for types which act as Bloom filters and support set
/// operations.
pub trait BinaryBloomFilter: BloomFilter
where
    Self::Set: BinaryBloomSet,
{
    /// Inserts all values from `other` into `self`. **`other` and
    /// `self` must have the same [`BuildHasher`]s for this to work,
    /// and this cannot be checked in general** (for instance,
    /// [`RandomState`](std::collections::hash_map::RandomState) does
    /// not implement [`PartialEq`]).
    ///
    /// # Example
    /// ```
    /// use generic_bloom::{BloomFilter, BinaryBloomFilter, SimpleBloomFilter};
    /// use bitvec::prelude::*;
    ///
    /// let mut f1: SimpleBloomFilter<BitBox<usize, Lsb0>> = SimpleBloomFilter::new(10, 20);
    /// f1.insert(&48);
    /// f1.insert(&32);
    ///
    /// let mut f2: SimpleBloomFilter<BitBox<usize, Lsb0>> =
    ///     SimpleBloomFilter::with_hashers(f1.hashers().clone(), 20);
    /// f2.insert(&39);
    ///
    /// assert!(f1.contains(&48));
    /// assert!(f1.contains(&32));
    /// // May fail if 39 is a false positive
    /// assert!(!f1.contains(&39));
    /// assert!(f2.contains(&39));
    ///
    /// f1.union(&f2);
    ///
    /// assert!(f1.contains(&48));
    /// assert!(f1.contains(&32));
    /// assert!(f1.contains(&39));
    /// ```
    fn union<Other>(&mut self, other: &Other)
        where Other: BinaryBloomFilter<Set = Self::Set, Hasher = Self::Hasher>;

    /// Keeps only values in `self` which are also in
    /// `other`. **`other` and `self` must have the same
    /// [`BuildHasher`]s for this to work, and this cannot be checked
    /// in general** (for instance,
    /// [`RandomState`](std::collections::hash_map::RandomState) does
    /// not implement [`PartialEq`]).
    ///
    /// # Example
    /// ```
    /// use generic_bloom::{BloomFilter, BinaryBloomFilter, SimpleBloomFilter};
    /// use bitvec::prelude::*;
    ///
    /// let mut f1: SimpleBloomFilter<BitBox<usize, Lsb0>> = SimpleBloomFilter::new(10, 20);
    /// f1.insert(&48);
    /// f1.insert(&32);
    ///
    /// let mut f2: SimpleBloomFilter<BitBox<usize, Lsb0>> =
    ///     SimpleBloomFilter::with_hashers(f1.hashers().clone(), 20);
    /// f2.insert(&32);
    /// f2.insert(&39);
    ///
    /// assert!(f1.contains(&48));
    /// assert!(f1.contains(&32));
    /// // May fail if 39 is a false positive
    /// assert!(!f1.contains(&39));
    /// assert!(f2.contains(&39));
    ///
    /// f1.intersect(&f2);
    ///
    /// // May fail if 48 is a false positive
    /// assert!(!f1.contains(&48));
    /// assert!(f1.contains(&32));
    /// // May fail if 39 is a false positive
    /// assert!(!f1.contains(&39));
    /// ```
    fn intersect<Other>(&mut self, other: &Other)
    where
        Other: BinaryBloomFilter<Set = Self::Set, Hasher = Self::Hasher>;
}

/// Trait for types which act as Bloom filters and support
/// count-based queries.
pub trait SpectralBloomFilter: BloomFilter
where
    Self::Set: SpectralBloomSet,
    <<Self as BloomFilter>::Set as SpectralBloomSet>::Count: Ord,
{
    /// Tests whether the set contains `val` more than `count` times.
    fn contains_more_than<T: Hash>(
        &self,
        val: &T,
        count: &<<Self as BloomFilter>::Set as SpectralBloomSet>::Count,
    ) -> bool;

    /// Returns an estimate of the number of times the set contains `val`.
    fn find_count<T: Hash>(&self, val: &T) -> &<<Self as BloomFilter>::Set as SpectralBloomSet>::Count;
}
