// Copyright 2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

mod bucket;
mod counter;
mod error;
mod histograms;
mod indexing;

pub use bucket::*;
pub use counter::*;
pub use error::*;
pub use histograms::*;
pub use indexing::*;

pub use rustcommon_atomics::{Atomic, AtomicU16, AtomicU32, AtomicU64, AtomicU8};

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn build() {
        let h = Histogram::<u8, u8>::new(255, 3);
        assert_eq!(h.percentile(0.0), Err(HistogramError::Empty));

        let mut h = Histogram::<u16, u8>::new(10000, 3);
        assert_eq!(h.percentile(0.0), Err(HistogramError::Empty));
        h.increment(1, 1);
        assert_eq!(h.percentile(0.0), Ok(1));
        assert_eq!(h.percentile(100.0), Ok(1));
        h.increment(65535, 1);
        assert_eq!(h.percentile(0.0), Ok(1));
        assert_eq!(h.percentile(100.0), Err(HistogramError::OutOfRange));
    }
}
