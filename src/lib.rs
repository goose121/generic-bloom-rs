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

//! This crate provides a [`BloomFilter`] trait which can be
//! parameterized by different types of storage (via the [`BloomSet`]
//! trait) to obtain traditional binary Bloom filters, counting Bloom
//! filters, and spectral Bloom filters. For basic usage, see the
//! documentation for [`SimpleBloomFilter`].
//!
//! [`BloomSet`] implementations are provided for
//! [`BitBox`](bitvec::boxed::BitBox)es (for traditional bitmap-style
//! Bloom filters) and `Box<[T]>` where `T` is a numeric type (for
//! counting or spectral Bloom filters).
//!
//! # Example
//! Basic usage:
//! ```
//! use generic_bloom::{BloomFilter, SimpleBloomFilter};
//! use bitvec::prelude::*;
//!
//! let mut filter: SimpleBloomFilter<BitBox<usize, Lsb0>> = SimpleBloomFilter::new(10, 20);
//! filter.insert(&48);
//! filter.insert(&32);
//! assert!(filter.contains(&48));
//! assert!(filter.contains(&32));
//! // May fail if 39 happens to be a false positive
//! assert!(!filter.contains(&39));
//! ```
mod simple_filter;
pub use simple_filter::SimpleBloomFilter;

pub mod traits;
pub use traits::filter::*;
pub use traits::set::BloomSet;

// #[cfg(test)]
// mod tests {
//     use crate::*;
//     use bitvec::boxed::BitBox;
//     use bitvec::order::Lsb0;
// }
