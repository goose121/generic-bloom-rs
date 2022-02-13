# generic-bloom

This crate provides a `BloomFilter` trait which can be parameterized
by different types of storage (via the `BloomSet` trait) to obtain
traditional binary Bloom filters, counting Bloom filters, and spectral
Bloom filters. For more information, see the
[documentation](https://docs.rs/generic-bloom/latest/generic-bloom).