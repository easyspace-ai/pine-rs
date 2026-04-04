//! Series buffer implementation for Pine Script
//!
//! Series are the core data structure in Pine Script - they're time-series buffers
//! that store historical values. This module provides `SeriesBuf<T>` which is a
//! circular buffer implementation optimized for:
//!
//! - O(1) push (amortized)
//! - O(1) indexed access (by offset from current bar)
//! - Automatic eviction when max_bars_back is exceeded
//!
//! # Indexing Conventions
//!
//! - `series[0]` = current bar (most recent)
//! - `series[1]` = previous bar
//! - `series[n]` = n bars ago
//!
//! Accessing beyond the available history returns `None` (or `na` at the Value level).

use std::ops::Index;

// SIMD lane size for f64 (AVX2 supports 256-bit = 4 x f64)
const F64_LANE_SIZE: usize = 4;

/// Sum a slice of f64 values using SIMD acceleration
#[inline]
fn simd_sum(values: &[f64]) -> f64 {
    // Process 4 elements at a time using SIMD
    let chunks = values.chunks_exact(F64_LANE_SIZE);
    let remainder = chunks.remainder();

    // Sum each chunk
    let mut sum = [0.0; F64_LANE_SIZE];
    for chunk in chunks {
        for i in 0..F64_LANE_SIZE {
            sum[i] += chunk[i];
        }
    }

    // Horizontal sum of SIMD lanes
    let mut total: f64 = sum.iter().sum();

    // Add remainder
    total += remainder.iter().sum::<f64>();

    total
}

/// Find maximum in a slice of f64 values using SIMD
#[inline]
fn simd_max(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }

    let chunks = values.chunks_exact(F64_LANE_SIZE);
    let remainder = chunks.remainder();

    // Track max for each lane
    let mut max_vals = [f64::NEG_INFINITY; F64_LANE_SIZE];

    for chunk in chunks {
        for i in 0..F64_LANE_SIZE {
            max_vals[i] = max_vals[i].max(chunk[i]);
        }
    }

    // Horizontal max
    let mut max_val = max_vals[0];
    for &val in max_vals.iter().take(F64_LANE_SIZE).skip(1) {
        max_val = max_val.max(val);
    }

    // Process remainder
    for &v in remainder {
        max_val = max_val.max(v);
    }

    Some(max_val)
}

/// Find minimum in a slice of f64 values using SIMD
#[inline]
fn simd_min(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }

    let chunks = values.chunks_exact(F64_LANE_SIZE);
    let remainder = chunks.remainder();

    // Track min for each lane
    let mut min_vals = [f64::INFINITY; F64_LANE_SIZE];

    for chunk in chunks {
        for i in 0..F64_LANE_SIZE {
            min_vals[i] = min_vals[i].min(chunk[i]);
        }
    }

    // Horizontal min
    let mut min_val = min_vals[0];
    for &val in min_vals.iter().take(F64_LANE_SIZE).skip(1) {
        min_val = min_val.min(val);
    }

    // Process remainder
    for &v in remainder {
        min_val = min_val.min(v);
    }

    Some(min_val)
}

/// A specialized series buffer for f64 values
///
/// This is optimized for financial data (OHLCV) and eliminates the overhead
/// of generic type handling. Uses contiguous memory for cache efficiency.
#[derive(Debug, Clone)]
pub struct SeriesBufF64 {
    /// The circular buffer storage
    buffer: Vec<f64>,
    /// Current write position in the buffer
    head: usize,
    /// Number of valid elements in the buffer
    len: usize,
    /// Maximum capacity (from RuntimeConfig.max_bars_back)
    capacity: usize,
}

impl SeriesBufF64 {
    /// Create a new f64 series buffer with given maximum length
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
            head: 0,
            len: 0,
            capacity,
        }
    }

    /// Create a new series buffer with a default value filled to capacity
    pub fn with_default(capacity: usize, default: f64) -> Self {
        Self {
            buffer: vec![default; capacity],
            head: 0,
            len: capacity,
            capacity,
        }
    }

    /// Get the number of valid elements in the series
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Check if the series is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Get the maximum capacity of the series
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Clear all elements from the series
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.head = 0;
        self.len = 0;
    }

    /// Push a new value to the series (adds current bar)
    #[inline]
    pub fn push(&mut self, value: f64) {
        if self.buffer.len() < self.capacity {
            // Still filling the initial capacity
            self.buffer.push(value);
            self.head = (self.head + 1) % self.capacity;
            self.len += 1;
        } else {
            // Circular buffer - overwrite oldest
            self.buffer[self.head] = value;
            self.head = (self.head + 1) % self.capacity;
        }
    }

    /// Push multiple values at once (batch optimization)
    pub fn extend(&mut self, values: &[f64]) {
        for &v in values {
            self.push(v);
        }
    }

    /// Overwrite the most recent element without deepening history.
    pub fn update_current(&mut self, value: f64) {
        if self.len == 0 {
            self.push(value);
            return;
        }

        let idx = if self.buffer.len() < self.capacity {
            self.head.wrapping_sub(1)
        } else {
            (self.head + self.capacity - 1) % self.capacity
        };

        if let Some(slot) = self.buffer.get_mut(idx) {
            *slot = value;
        }
    }

    /// Get the value at the given offset from current
    ///
    /// - `get(0)` returns the most recent value (current bar)
    /// - `get(1)` returns the previous bar's value
    /// - Returns `None` if offset >= len
    #[inline]
    pub fn get(&self, offset: usize) -> Option<f64> {
        if offset >= self.len {
            return None;
        }

        // Calculate index: head - 1 - offset (wrapped around)
        let idx = if self.head > offset {
            self.head - 1 - offset
        } else {
            self.capacity - (offset - self.head) - 1
        };

        self.buffer.get(idx).copied()
    }

    /// Get a reference to the value at the given offset
    #[inline]
    pub fn get_ref(&self, offset: usize) -> Option<&f64> {
        if offset >= self.len {
            return None;
        }

        let idx = if self.head > offset {
            self.head - 1 - offset
        } else {
            self.capacity - (offset - self.head) - 1
        };

        self.buffer.get(idx)
    }

    /// Get the most recent value (current bar)
    #[inline]
    pub fn current(&self) -> Option<f64> {
        self.get(0)
    }

    /// Get the previous value
    #[inline]
    pub fn previous(&self) -> Option<f64> {
        self.get(1)
    }

    /// Get the oldest value in the series
    pub fn oldest(&self) -> Option<f64> {
        if self.is_empty() {
            return None;
        }
        self.get(self.len - 1)
    }

    /// Get all values as a Vec (newest to oldest)
    pub fn to_vec(&self) -> Vec<f64> {
        self.iter().collect()
    }

    /// Get all values as a Vec (oldest to newest)
    pub fn to_vec_oldest_first(&self) -> Vec<f64> {
        self.iter_oldest_first().collect()
    }

    /// Iterate from newest to oldest (offset 0 to len-1)
    pub fn iter(&self) -> SeriesBufF64Iter<'_> {
        SeriesBufF64Iter {
            series: self,
            offset: 0,
        }
    }

    /// Iterate from oldest to newest
    pub fn iter_oldest_first(&self) -> SeriesBufF64IterOldestFirst<'_> {
        SeriesBufF64IterOldestFirst {
            series: self,
            index: 0,
        }
    }

    /// Calculate simple moving average for the last n bars using SIMD
    ///
    /// Returns None if n > len or n == 0
    #[allow(clippy::needless_range_loop)]
    pub fn sma(&self, n: usize) -> Option<f64> {
        if n == 0 || n > self.len {
            return None;
        }

        // Collect values into a temporary buffer for SIMD processing
        // This is more cache-friendly than random access
        let mut temp = vec![0.0; n];
        for i in 0..n {
            temp[i] = self.get(i)?;
        }

        let sum = simd_sum(&temp);
        Some(sum / n as f64)
    }

    /// Calculate the sum of last n bars using SIMD
    #[allow(clippy::needless_range_loop)]
    pub fn sum(&self, n: usize) -> Option<f64> {
        if n == 0 || n > self.len {
            return None;
        }

        // Collect values for SIMD processing
        let mut temp = vec![0.0; n];
        for i in 0..n {
            temp[i] = self.get(i)?;
        }

        Some(simd_sum(&temp))
    }

    /// Get the maximum value in the last n bars using SIMD
    #[allow(clippy::needless_range_loop)]
    pub fn max(&self, n: usize) -> Option<f64> {
        if n == 0 || n > self.len {
            return None;
        }

        // Collect values for SIMD processing
        let mut temp = vec![0.0; n];
        for i in 0..n {
            temp[i] = self.get(i)?;
        }

        simd_max(&temp)
    }

    /// Get the minimum value in the last n bars using SIMD
    #[allow(clippy::needless_range_loop)]
    pub fn min(&self, n: usize) -> Option<f64> {
        if n == 0 || n > self.len {
            return None;
        }

        // Collect values for SIMD processing
        let mut temp = vec![0.0; n];
        for i in 0..n {
            temp[i] = self.get(i)?;
        }

        simd_min(&temp)
    }

    /// Fast path for SMA when buffer is contiguous (no wrap-around)
    /// This avoids the temporary buffer allocation
    pub fn sma_fast(&self, n: usize) -> Option<f64> {
        if n == 0 || n > self.len {
            return None;
        }

        // Check if the last n elements are contiguous in memory
        if self.head >= n && self.buffer.len() == self.capacity {
            // Data is contiguous: [head-n, head-1]
            let start = self.head - n;
            let slice = &self.buffer[start..self.head];
            Some(simd_sum(slice) / n as f64)
        } else {
            // Fall back to regular SMA
            self.sma(n)
        }
    }
}

impl Index<usize> for SeriesBufF64 {
    type Output = f64;

    fn index(&self, index: usize) -> &Self::Output {
        self.get_ref(index).expect("series index out of bounds")
    }
}

/// Iterator from newest to oldest for SeriesBufF64
pub struct SeriesBufF64Iter<'a> {
    series: &'a SeriesBufF64,
    offset: usize,
}

impl<'a> Iterator for SeriesBufF64Iter<'a> {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.series.get(self.offset);
        self.offset += 1;
        result
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.series.len().saturating_sub(self.offset);
        (remaining, Some(remaining))
    }
}

impl<'a> ExactSizeIterator for SeriesBufF64Iter<'a> {}

/// Iterator from oldest to newest for SeriesBufF64
pub struct SeriesBufF64IterOldestFirst<'a> {
    series: &'a SeriesBufF64,
    index: usize,
}

impl<'a> Iterator for SeriesBufF64IterOldestFirst<'a> {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.series.len() {
            return None;
        }
        let offset = self.series.len() - 1 - self.index;
        self.index += 1;
        self.series.get(offset)
    }
}

/// A circular buffer for time-series data
///
/// This is the core data structure for Pine Script series. It maintains a
/// sliding window of values with O(1) access by historical offset.
#[derive(Debug, Clone)]
pub struct SeriesBuf<T> {
    /// The circular buffer storage
    buffer: Vec<T>,
    /// Current write position in the buffer
    head: usize,
    /// Number of valid elements in the buffer
    len: usize,
    /// Maximum capacity (from RuntimeConfig.max_bars_back)
    capacity: usize,
}

impl<T> SeriesBuf<T> {
    /// Create a new series buffer with given maximum length
    ///
    /// # Examples
    /// ```
    /// use pine_runtime::series::SeriesBuf;
    ///
    /// let series: SeriesBuf<i64> = SeriesBuf::new(100);
    /// assert_eq!(series.len(), 0);
    /// assert_eq!(series.capacity(), 100);
    /// ```
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
            head: 0,
            len: 0,
            capacity,
        }
    }

    /// Create a new series buffer with a default value filled to capacity
    ///
    /// This is useful for creating series that start with `na` values.
    pub fn with_default(capacity: usize, default: T) -> Self
    where
        T: Clone,
    {
        Self {
            buffer: vec![default; capacity],
            head: 0,
            len: capacity,
            capacity,
        }
    }

    /// Get the number of valid elements in the series
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Check if the series is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Get the maximum capacity of the series
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Clear all elements from the series
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.head = 0;
        self.len = 0;
    }

    /// Push a new value to the series (adds current bar)
    ///
    /// If the series is at capacity, the oldest value is overwritten.
    pub fn push(&mut self, value: T) {
        if self.buffer.len() < self.capacity {
            // Still filling the initial capacity
            self.buffer.push(value);
            self.head = (self.head + 1) % self.capacity;
            self.len += 1;
        } else {
            // Circular buffer - overwrite oldest
            self.buffer[self.head] = value;
            self.head = (self.head + 1) % self.capacity;
        }
    }

    /// Overwrite the most recent element without deepening history (same Pine bar).
    ///
    /// Used when a `var` series receives multiple updates in one bar; only the final value
    /// should advance history on the next bar.
    pub fn update_current(&mut self, value: T) {
        if self.len == 0 {
            self.push(value);
            return;
        }
        let idx = if self.buffer.len() < self.capacity {
            self.head.wrapping_sub(1)
        } else {
            (self.head + self.capacity - 1) % self.capacity
        };
        if let Some(slot) = self.buffer.get_mut(idx) {
            *slot = value;
        }
    }

    /// Get the value at the given offset from current
    ///
    /// - `get(0)` returns the most recent value (current bar)
    /// - `get(1)` returns the previous bar's value
    /// - Returns `None` if offset >= len
    ///
    /// # Examples
    /// ```
    /// use pine_runtime::series::SeriesBuf;
    ///
    /// let mut series = SeriesBuf::new(10);
    /// series.push(10);
    /// series.push(20);
    /// series.push(30);
    ///
    /// assert_eq!(series.get(0), Some(&30)); // current
    /// assert_eq!(series.get(1), Some(&20)); // previous
    /// assert_eq!(series.get(2), Some(&10)); // two bars ago
    /// assert_eq!(series.get(3), None);      // out of range
    /// ```
    pub fn get(&self, offset: usize) -> Option<&T> {
        if offset >= self.len {
            return None;
        }

        // Calculate index: head - 1 - offset (wrapped around)
        let idx = if self.head > offset {
            self.head - 1 - offset
        } else {
            self.capacity - (offset - self.head) - 1
        };

        self.buffer.get(idx)
    }

    /// Get a mutable reference to the value at the given offset
    pub fn get_mut(&mut self, offset: usize) -> Option<&mut T> {
        if offset >= self.len {
            return None;
        }

        let idx = if self.head > offset {
            self.head - 1 - offset
        } else {
            self.capacity - (offset - self.head) - 1
        };

        self.buffer.get_mut(idx)
    }

    /// Get the most recent value (current bar)
    ///
    /// Equivalent to `get(0)`.
    #[inline]
    pub fn current(&self) -> Option<&T> {
        self.get(0)
    }

    /// Get the previous value
    ///
    /// Equivalent to `get(1)`.
    #[inline]
    pub fn previous(&self) -> Option<&T> {
        self.get(1)
    }

    /// Get the oldest value in the series
    pub fn oldest(&self) -> Option<&T> {
        if self.is_empty() {
            return None;
        }
        self.get(self.len - 1)
    }

    /// Iterate from newest to oldest (offset 0 to len-1)
    pub fn iter(&self) -> SeriesIter<'_, T> {
        SeriesIter {
            series: self,
            offset: 0,
        }
    }

    /// Iterate from oldest to newest
    pub fn iter_oldest_first(&self) -> SeriesIterOldestFirst<'_, T> {
        SeriesIterOldestFirst {
            series: self,
            index: 0,
        }
    }
}

impl<T: Clone> SeriesBuf<T> {
    /// Get a cloned value at the given offset
    pub fn get_cloned(&self, offset: usize) -> Option<T> {
        self.get(offset).cloned()
    }

    /// Get all values as a Vec (newest to oldest)
    pub fn to_vec(&self) -> Vec<T> {
        self.iter().cloned().collect()
    }
}

/// Index implementation for SeriesBuf
///
/// # Panics
///
/// Panics if index >= len (just like Vec)
impl<T> Index<usize> for SeriesBuf<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).expect("series index out of bounds")
    }
}

/// Iterator from newest to oldest
pub struct SeriesIter<'a, T> {
    series: &'a SeriesBuf<T>,
    offset: usize,
}

impl<'a, T> Iterator for SeriesIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.series.get(self.offset);
        self.offset += 1;
        result
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.series.len().saturating_sub(self.offset);
        (remaining, Some(remaining))
    }
}

impl<'a, T> ExactSizeIterator for SeriesIter<'a, T> {}

/// Iterator from oldest to newest
pub struct SeriesIterOldestFirst<'a, T> {
    series: &'a SeriesBuf<T>,
    index: usize,
}

impl<'a, T> Iterator for SeriesIterOldestFirst<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.series.len() {
            return None;
        }
        // Calculate offset from oldest: offset = len - 1 - index
        let offset = self.series.len() - 1 - self.index;
        self.index += 1;
        self.series.get(offset)
    }
}

/// A typed series ID for use in ExecutionContext
///
/// This is a type-erased identifier for series stored in the context.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SeriesId(pub usize);

/// Types that can be stored in a series
///
/// This trait is implemented for types that can be used in Pine Script series.
/// Currently supported: f64 (close, high, etc.), i64 (volume), Value.
pub trait SeriesValue: Clone + 'static {}

impl SeriesValue for f64 {}
impl SeriesValue for i64 {}
impl SeriesValue for i32 {}
impl SeriesValue for bool {}
impl SeriesValue for crate::value::Value {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operations() {
        let mut series = SeriesBuf::new(10);
        assert!(series.is_empty());
        assert_eq!(series.len(), 0);

        series.push(1.0);
        assert_eq!(series.len(), 1);
        assert_eq!(series.get(0), Some(&1.0));

        series.push(2.0);
        series.push(3.0);
        assert_eq!(series.len(), 3);
        assert_eq!(series.get(0), Some(&3.0)); // current
        assert_eq!(series.get(1), Some(&2.0)); // previous
        assert_eq!(series.get(2), Some(&1.0)); // oldest
    }

    #[test]
    fn test_circular_behavior() {
        let mut series = SeriesBuf::new(3);

        // Fill to capacity
        series.push(1.0);
        series.push(2.0);
        series.push(3.0);
        assert_eq!(series.len(), 3);
        assert_eq!(series.get(0), Some(&3.0));
        assert_eq!(series.get(2), Some(&1.0));

        // Overwrite oldest (1.0)
        series.push(4.0);
        assert_eq!(series.len(), 3);
        assert_eq!(series.get(0), Some(&4.0)); // current
        assert_eq!(series.get(1), Some(&3.0)); // previous
        assert_eq!(series.get(2), Some(&2.0)); // 2 bars ago (was 1.0)
        assert_eq!(series.get(3), None); // out of range
    }

    #[test]
    fn test_index_operator() {
        let mut series = SeriesBuf::new(10);
        series.push(10);
        series.push(20);
        series.push(30);

        assert_eq!(series[0], 30);
        assert_eq!(series[1], 20);
        assert_eq!(series[2], 10);
    }

    #[test]
    #[should_panic(expected = "series index out of bounds")]
    fn test_index_out_of_bounds() {
        let series: SeriesBuf<i32> = SeriesBuf::new(10);
        let _ = series[0]; // Should panic
    }

    #[test]
    fn test_iterator() {
        let mut series = SeriesBuf::new(10);
        series.push(1);
        series.push(2);
        series.push(3);

        let values: Vec<_> = series.iter().copied().collect();
        assert_eq!(values, vec![3, 2, 1]);
    }

    #[test]
    fn test_iterator_oldest_first() {
        let mut series = SeriesBuf::new(10);
        series.push(1);
        series.push(2);
        series.push(3);

        let values: Vec<_> = series.iter_oldest_first().copied().collect();
        assert_eq!(values, vec![1, 2, 3]);
    }

    #[test]
    fn test_with_default() {
        let series = SeriesBuf::with_default(5, 0.0);
        assert_eq!(series.len(), 5);
        assert_eq!(series[0], 0.0);
        assert_eq!(series[4], 0.0);
    }

    #[test]
    fn test_clear() {
        let mut series = SeriesBuf::new(10);
        series.push(1);
        series.push(2);
        assert_eq!(series.len(), 2);

        series.clear();
        assert!(series.is_empty());
        assert_eq!(series.get(0), None);
    }

    #[test]
    fn test_current_and_previous() {
        let mut series = SeriesBuf::new(10);
        assert_eq!(series.current(), None);
        assert_eq!(series.previous(), None);

        series.push(100);
        assert_eq!(series.current(), Some(&100));
        assert_eq!(series.previous(), None);

        series.push(200);
        assert_eq!(series.current(), Some(&200));
        assert_eq!(series.previous(), Some(&100));
    }

    #[test]
    fn test_get_mut() {
        let mut series = SeriesBuf::new(10);
        series.push(1);
        series.push(2);
        series.push(3);

        if let Some(val) = series.get_mut(0) {
            *val = 300;
        }
        assert_eq!(series.get(0), Some(&300));
    }

    #[test]
    fn test_oldest() {
        let mut series = SeriesBuf::new(3);
        assert_eq!(series.oldest(), None);

        series.push(1);
        series.push(2);
        assert_eq!(series.oldest(), Some(&1));

        series.push(3);
        assert_eq!(series.oldest(), Some(&1));

        series.push(4); // Overwrites 1
        assert_eq!(series.oldest(), Some(&2));
    }

    #[test]
    fn test_to_vec() {
        let mut series = SeriesBuf::new(10);
        series.push(1);
        series.push(2);
        series.push(3);

        assert_eq!(series.to_vec(), vec![3, 2, 1]);
    }

    // SeriesBufF64 tests
    #[test]
    fn test_f64_basic_operations() {
        let mut series = SeriesBufF64::new(10);
        assert!(series.is_empty());
        assert_eq!(series.len(), 0);

        series.push(1.0);
        assert_eq!(series.len(), 1);
        assert_eq!(series.get(0), Some(1.0));

        series.push(2.0);
        series.push(3.0);
        assert_eq!(series.len(), 3);
        assert_eq!(series.get(0), Some(3.0)); // current
        assert_eq!(series.get(1), Some(2.0)); // previous
        assert_eq!(series.get(2), Some(1.0)); // oldest
    }

    #[test]
    fn test_f64_circular_behavior() {
        let mut series = SeriesBufF64::new(3);

        // Fill to capacity
        series.push(1.0);
        series.push(2.0);
        series.push(3.0);
        assert_eq!(series.len(), 3);
        assert_eq!(series.get(0), Some(3.0));
        assert_eq!(series.get(2), Some(1.0));

        // Overwrite oldest (1.0)
        series.push(4.0);
        assert_eq!(series.len(), 3);
        assert_eq!(series.get(0), Some(4.0)); // current
        assert_eq!(series.get(1), Some(3.0)); // previous
        assert_eq!(series.get(2), Some(2.0)); // 2 bars ago (was 1.0)
        assert_eq!(series.get(3), None); // out of range
    }

    #[test]
    fn test_f64_sma() {
        let mut series = SeriesBufF64::new(10);
        series.push(10.0);
        series.push(20.0);
        series.push(30.0);

        // SMA of last 3 bars: (30 + 20 + 10) / 3 = 20.0
        assert_eq!(series.sma(3), Some(20.0));

        // SMA of last 2 bars: (30 + 20) / 2 = 25.0
        assert_eq!(series.sma(2), Some(25.0));

        // SMA of 1 bar = current value
        assert_eq!(series.sma(1), Some(30.0));

        // SMA of 0 bars = None
        assert_eq!(series.sma(0), None);

        // SMA of more bars than available = None
        assert_eq!(series.sma(5), None);
    }

    #[test]
    fn test_f64_sum() {
        let mut series = SeriesBufF64::new(10);
        series.push(10.0);
        series.push(20.0);
        series.push(30.0);

        assert_eq!(series.sum(3), Some(60.0));
        assert_eq!(series.sum(2), Some(50.0));
    }

    #[test]
    fn test_f64_max_min() {
        let mut series = SeriesBufF64::new(10);
        series.push(10.0);
        series.push(30.0);
        series.push(20.0);

        assert_eq!(series.max(3), Some(30.0));
        assert_eq!(series.min(3), Some(10.0));
        assert_eq!(series.max(2), Some(30.0)); // 30 and 20
        assert_eq!(series.min(2), Some(20.0)); // 30 and 20
    }

    #[test]
    fn test_f64_iterators() {
        let mut series = SeriesBufF64::new(10);
        series.push(1.0);
        series.push(2.0);
        series.push(3.0);

        // Newest to oldest
        let values: Vec<_> = series.iter().collect();
        assert_eq!(values, vec![3.0, 2.0, 1.0]);

        // Oldest to newest
        let values: Vec<_> = series.iter_oldest_first().collect();
        assert_eq!(values, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_f64_index() {
        let mut series = SeriesBufF64::new(10);
        series.push(10.0);
        series.push(20.0);
        series.push(30.0);

        assert_eq!(series[0], 30.0);
        assert_eq!(series[1], 20.0);
        assert_eq!(series[2], 10.0);
    }

    #[test]
    fn test_f64_extend() {
        let mut series = SeriesBufF64::new(10);
        series.extend(&[1.0, 2.0, 3.0, 4.0, 5.0]);

        assert_eq!(series.len(), 5);
        assert_eq!(series.get(0), Some(5.0));
        assert_eq!(series.get(4), Some(1.0));
    }

    #[test]
    fn test_f64_sma_fast() {
        let mut series = SeriesBufF64::new(10);
        // Fill the buffer to capacity first
        series.extend(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0]);

        // sma_fast should work when data is contiguous
        assert_eq!(series.sma_fast(3), Some(9.0)); // (10 + 9 + 8) / 3
        assert_eq!(series.sma_fast(5), Some(8.0)); // (10 + 9 + 8 + 7 + 6) / 5

        // sma_fast falls back to sma when data wraps around
        series.push(11.0); // This overwrites 1.0
        assert_eq!(series.sma_fast(3), Some(10.0)); // (11 + 10 + 9) / 3
    }

    #[test]
    fn test_simd_sum_large() {
        // Test SIMD sum with more than 4 elements (SIMD lane size)
        let mut series = SeriesBufF64::new(20);
        let values: Vec<f64> = (1..=16).map(|i| i as f64).collect();
        series.extend(&values);

        // Sum of 1..16 = 16*17/2 = 136
        assert_eq!(series.sum(16), Some(136.0));
    }

    #[test]
    fn test_simd_max_min_large() {
        // Test SIMD max/min with more than 4 elements
        let mut series = SeriesBufF64::new(20);
        let values = [5.0, 2.0, 8.0, 1.0, 9.0, 3.0, 7.0, 4.0];
        series.extend(&values);

        assert_eq!(series.max(8), Some(9.0));
        assert_eq!(series.min(8), Some(1.0));
    }
}
