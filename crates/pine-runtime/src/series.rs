//! Series buffer implementation

/// Series buffer for storing historical values
#[derive(Debug, Clone)]
pub struct SeriesBuf<T> {
    /// Internal buffer
    data: Vec<T>,
    /// Maximum length (from RuntimeConfig.max_bars_back)
    max_len: usize,
}

impl<T> SeriesBuf<T> {
    /// Create a new series buffer
    pub fn new(max_len: usize) -> Self {
        Self {
            data: Vec::with_capacity(max_len),
            max_len,
        }
    }

    /// Push a value to the series
    pub fn push(&mut self, value: T) {
        if self.data.len() >= self.max_len {
            self.data.remove(0);
        }
        self.data.push(value);
    }

    /// Get value at index (0 = current, 1 = previous, etc.)
    pub fn get(&self, index: usize) -> Option<&T> {
        self.data.iter().rev().nth(index)
    }

    /// Get the length of the series
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the series is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl<T: Clone> SeriesBuf<T> {
    /// Get a copy of the value at index
    pub fn get_cloned(&self, index: usize) -> Option<T> {
        self.get(index).cloned()
    }
}
