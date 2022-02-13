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

//! Traits for Bloom filter functionality.
//!
//! In general, the [`BloomFilter`](filter::BloomFilter) should
//! control which positions in the underlying storage are incremented
//! or decremented for each operation, and should contain a
//! [`BloomSet`](set::BloomSet) which tracks the actual counter values. For instance,
//! counting Bloom filters, which are based solely on how the storage
//! handles increment/decrement operations on each individual counter,
//! should be implemented using [`BloomSet`](set::BloomSet), while the
//! Minimal Increase optimization for spectral bloom filters should be
//! implemented as a [`BloomFilter`](filter::BloomFilter), because it
//! involves optimizing which counters are incremented.

pub mod filter;
pub mod set;
