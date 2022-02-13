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

//! Traits for [`BloomFilter`](crate::BloomFilter) storage.
//!
//! These traits describe different features of various types of
//! backing storage for
//! [`BloomFilter`](crate::BloomFilter)s. Implementations are provided
//! for [`BitVec`]s, providing a binary Bloom filter, and for
//! `Box<[T]>` where `T` is a numeric type, providing a spectral Bloom
//! filter which supports deletions.
use bitvec::{boxed::BitBox, order::BitOrder, store::BitStore, vec::BitVec};
use num_traits::{Bounded, One, SaturatingAdd, Zero};
use std::ops::SubAssign;

/// A trait for types which can serve as the underlying storage for a
/// [`BloomFilter`](crate::BloomFilter).
pub trait BloomSet {
    /// Creates a new set with `count` counters.
    fn new(count: usize) -> Self;

    /// Returns the number of counters in the storage.
    fn size(&self) -> usize;

    /// Increments the counter with index `index`.
    fn increment(&mut self, index: usize);

    /// Clears all counters.
    fn clear(&mut self);

    /// Queries whether a counter indicates presence.
    fn query(&self, index: usize) -> bool;
}

/// A trait for types which can serve as the underlying storage for a
/// [`BloomFilter`](crate::BloomFilter) and perform deletions.
pub trait BloomSetDelete: BloomSet {
    /// Decrements the counter with index `index`.
    fn decrement(&mut self, index: usize);
}

/// A trait for types which can serve as the underlying storage for a
/// [`BloomFilter`](crate::BloomFilter) and perform threshold-based
/// lookups.
pub trait SpectralBloomSet: BloomSet {
    type Count;

    /// Returns the count at `index`.
    fn query_count(&self, index: usize) -> &Self::Count;
}

/// A trait for types which can serve as the underlying storage for a
/// [`BloomFilter`](crate::BloomFilter) and perform unions and
/// intersections.
pub trait BinaryBloomSet: BloomSet {
    /// Inserts all values from `other` into `self`.
    fn union(&mut self, other: &Self);

    /// Keeps only values in `self` which are also in `other`.
    fn intersect(&mut self, other: &Self);
}

impl<T, O> BloomSet for BitBox<T, O>
where
    T: BitStore,
    O: BitOrder,
{
    fn new(count: usize) -> Self {
        BitVec::repeat(false, count).into_boxed_bitslice()
    }

    fn size(&self) -> usize {
        self.len()
    }

    fn increment(&mut self, index: usize) {
        self.set(index, true);
    }

    fn clear(&mut self) {
        self.fill(false);
    }

    fn query(&self, index: usize) -> bool {
        self[index]
    }
}

impl<T, O> BinaryBloomSet for BitBox<T, O>
where
    T: BitStore,
    O: BitOrder,
{
    fn union(&mut self, other: &Self) {
        *self |= other;
    }

    fn intersect(&mut self, other: &Self) {
        *self &= other;
    }
}

impl<T> BloomSet for Box<[T]>
where
    T: SaturatingAdd + One + Zero + Ord,
{
    fn new(count: usize) -> Self {
        std::iter::repeat_with(T::zero)
            .take(count)
            .collect::<Vec<T>>()
            .into_boxed_slice()
    }

    fn size(&self) -> usize {
        self.len()
    }

    fn increment(&mut self, index: usize) {
        self[index] = self[index].saturating_add(&T::one());
    }

    fn clear(&mut self) {
        self.fill_with(T::zero);
    }

    fn query(&self, index: usize) -> bool {
        self.query_count(index) > &T::zero()
    }
}

impl<T> BloomSetDelete for Box<[T]>
where
    T: SaturatingAdd + SubAssign + One + Zero + Ord + Bounded,
{
    fn decrement(&mut self, index: usize) {
        if self[index] != T::max_value() {
            self[index] -= T::one();
        }
    }
}

impl<T> SpectralBloomSet for Box<[T]>
where
    T: SaturatingAdd + One + Zero + Ord,
{
    type Count = T;

    fn query_count(&self, index: usize) -> &Self::Count {
        &self[index]
    }
}
