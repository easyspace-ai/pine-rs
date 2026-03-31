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
}
