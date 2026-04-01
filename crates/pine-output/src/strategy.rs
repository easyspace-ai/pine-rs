//! Strategy functions (strategy.*)

use crate::{Direction, Result, TradeSignal};

/// Strategy configuration
#[derive(Debug, Clone)]
pub struct StrategyConfig {
    /// Strategy name
    pub name: String,
    /// Pyramiding (number of entries in the same direction)
    pub pyramiding: u32,
    /// Commission (as percentage)
    pub commission: f64,
    /// Slippage (number of ticks)
    pub slippage: u32,
    /// Initial capital
    pub initial_capital: f64,
    /// Default quantity type
    pub default_qty_type: QtyType,
    /// Default quantity value
    pub default_qty_value: f64,
    /// Whether to close entries by reversing
    pub close_entries_rule: CloseEntriesRule,
}

impl Default for StrategyConfig {
    fn default() -> Self {
        Self {
            name: "Strategy".to_string(),
            pyramiding: 0,
            commission: 0.0,
            slippage: 0,
            initial_capital: 100000.0,
            default_qty_type: QtyType::PercentOfEquity,
            default_qty_value: 100.0,
            close_entries_rule: CloseEntriesRule::Immediately,
        }
    }
}

/// Quantity type for strategy entries
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum QtyType {
    /// Fixed number of contracts/shares
    Contracts,
    /// Percentage of available equity
    PercentOfEquity,
    /// Currency amount
    Currency,
}

/// Rule for closing entries
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CloseEntriesRule {
    /// Close immediately on opposite signal
    Immediately,
    /// Close by FIFO (First In First Out)
    FIFO,
    /// Close by LIFO (Last In First Out)
    LIFO,
}

/// Strategy state
#[derive(Debug, Clone)]
pub struct Strategy {
    /// Configuration
    pub config: StrategyConfig,
    /// Current position size (positive = long, negative = short)
    pub position_size: f64,
    /// Entry signals history
    pub entries: Vec<TradeSignal>,
    /// Exit signals history
    pub exits: Vec<TradeSignal>,
    /// Current equity
    pub equity: f64,
    /// Open trades (for pyramiding)
    pub open_trades: Vec<Trade>,
}

/// Individual trade
#[derive(Debug, Clone)]
pub struct Trade {
    /// Entry bar index
    pub entry_bar: i64,
    /// Entry price
    pub entry_price: f64,
    /// Quantity
    pub qty: f64,
    /// Direction
    pub direction: Direction,
    /// Entry comment
    pub comment: Option<String>,
}

impl Strategy {
    /// Create a new strategy with the given configuration
    pub fn new(config: StrategyConfig) -> Self {
        let equity = config.initial_capital;
        Self {
            config,
            position_size: 0.0,
            entries: Vec::new(),
            exits: Vec::new(),
            equity,
            open_trades: Vec::new(),
        }
    }

    /// Create a default strategy
    pub fn default_with_name(name: impl Into<String>) -> Self {
        let config = StrategyConfig {
            name: name.into(),
            ..Default::default()
        };
        Self::new(config)
    }

    /// Enter a long position
    pub fn entry_long(
        &mut self,
        bar_index: i64,
        qty: Option<f64>,
        price: Option<f64>,
        comment: Option<String>,
    ) -> Result<()> {
        let qty = self.resolve_qty(qty);
        if qty <= 0.0 {
            return Ok(());
        }

        // Check pyramiding limit
        if self.config.pyramiding > 0 {
            let same_direction_trades = self
                .open_trades
                .iter()
                .filter(|t| t.direction == Direction::Long)
                .count();
            if same_direction_trades >= self.config.pyramiding as usize {
                return Ok(());
            }
        }

        // Close short positions if any
        if self.position_size < 0.0 {
            self.close(bar_index, price, None)?;
        }

        let signal = TradeSignal {
            bar_index,
            direction: Direction::Long,
            qty,
            price,
            comment: comment.clone(),
        };
        self.entries.push(signal);

        let trade = Trade {
            entry_bar: bar_index,
            entry_price: price.unwrap_or(0.0),
            qty,
            direction: Direction::Long,
            comment,
        };
        self.open_trades.push(trade);
        self.position_size += qty;

        Ok(())
    }

    /// Enter a short position
    pub fn entry_short(
        &mut self,
        bar_index: i64,
        qty: Option<f64>,
        price: Option<f64>,
        comment: Option<String>,
    ) -> Result<()> {
        let qty = self.resolve_qty(qty);
        if qty <= 0.0 {
            return Ok(());
        }

        // Check pyramiding limit
        if self.config.pyramiding > 0 {
            let same_direction_trades = self
                .open_trades
                .iter()
                .filter(|t| t.direction == Direction::Short)
                .count();
            if same_direction_trades >= self.config.pyramiding as usize {
                return Ok(());
            }
        }

        // Close long positions if any
        if self.position_size > 0.0 {
            self.close(bar_index, price, None)?;
        }

        let signal = TradeSignal {
            bar_index,
            direction: Direction::Short,
            qty: -qty, // Negative for short
            price,
            comment: comment.clone(),
        };
        self.entries.push(signal);

        let trade = Trade {
            entry_bar: bar_index,
            entry_price: price.unwrap_or(0.0),
            qty,
            direction: Direction::Short,
            comment,
        };
        self.open_trades.push(trade);
        self.position_size -= qty;

        Ok(())
    }

    /// Exit a specific position or all positions
    pub fn exit(
        &mut self,
        bar_index: i64,
        qty: Option<f64>,
        price: Option<f64>,
        comment: Option<String>,
    ) -> Result<()> {
        if self.position_size == 0.0 {
            return Ok(());
        }

        let exit_qty = qty.unwrap_or(self.position_size.abs());
        let exit_qty = exit_qty.min(self.position_size.abs());

        let signal = TradeSignal {
            bar_index,
            direction: Direction::Close,
            qty: exit_qty,
            price,
            comment: comment.clone(),
        };
        self.exits.push(signal);

        // Update position
        if self.position_size > 0.0 {
            self.position_size -= exit_qty;
        } else {
            self.position_size += exit_qty;
        }

        // Remove closed trades
        self.close_trades(exit_qty);

        Ok(())
    }

    /// Close all positions
    pub fn close(
        &mut self,
        bar_index: i64,
        price: Option<f64>,
        comment: Option<String>,
    ) -> Result<()> {
        if self.position_size == 0.0 {
            return Ok(());
        }

        let qty = self.position_size.abs();
        let signal = TradeSignal {
            bar_index,
            direction: Direction::Close,
            qty,
            price,
            comment: comment.clone(),
        };
        self.exits.push(signal);

        self.position_size = 0.0;
        self.open_trades.clear();

        Ok(())
    }

    /// Resolve quantity based on configuration
    fn resolve_qty(&self, qty: Option<f64>) -> f64 {
        match qty {
            Some(q) => q,
            None => match self.config.default_qty_type {
                QtyType::Contracts => self.config.default_qty_value,
                QtyType::PercentOfEquity => self.equity * self.config.default_qty_value / 100.0,
                QtyType::Currency => self.config.default_qty_value,
            },
        }
    }

    /// Close trades up to the specified quantity
    fn close_trades(&mut self, qty: f64) {
        let mut remaining = qty;
        self.open_trades.retain(|trade| {
            if remaining <= 0.0 {
                return true;
            }
            if trade.qty <= remaining {
                remaining -= trade.qty;
                false
            } else {
                remaining = 0.0;
                true
            }
        });
    }

    /// Get net profit (simplified calculation)
    pub fn net_profit(&self) -> f64 {
        // Simplified calculation - in reality would need price history
        self.equity - self.config.initial_capital
    }

    /// Get current position direction
    pub fn position_direction(&self) -> Direction {
        if self.position_size > 0.0 {
            Direction::Long
        } else if self.position_size < 0.0 {
            Direction::Short
        } else {
            Direction::None
        }
    }
}

/// Enter a long position (convenience function)
pub fn entry_long(
    strategy: &mut Strategy,
    bar_index: i64,
    qty: Option<f64>,
    price: Option<f64>,
    comment: Option<String>,
) -> Result<()> {
    strategy.entry_long(bar_index, qty, price, comment)
}

/// Enter a short position (convenience function)
pub fn entry_short(
    strategy: &mut Strategy,
    bar_index: i64,
    qty: Option<f64>,
    price: Option<f64>,
    comment: Option<String>,
) -> Result<()> {
    strategy.entry_short(bar_index, qty, price, comment)
}

/// Exit a position (convenience function)
pub fn exit(
    strategy: &mut Strategy,
    bar_index: i64,
    qty: Option<f64>,
    price: Option<f64>,
    comment: Option<String>,
) -> Result<()> {
    strategy.exit(bar_index, qty, price, comment)
}

/// Close all positions (convenience function)
pub fn close(
    strategy: &mut Strategy,
    bar_index: i64,
    price: Option<f64>,
    comment: Option<String>,
) -> Result<()> {
    strategy.close(bar_index, price, comment)
}

/// Set strategy properties
pub fn set_properties(
    strategy: &mut Strategy,
    pyramiding: Option<u32>,
    commission: Option<f64>,
    slippage: Option<u32>,
) -> Result<()> {
    if let Some(p) = pyramiding {
        strategy.config.pyramiding = p;
    }
    if let Some(c) = commission {
        strategy.config.commission = c;
    }
    if let Some(s) = slippage {
        strategy.config.slippage = s;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strategy_creation() {
        let strategy = Strategy::default_with_name("Test Strategy");
        assert_eq!(strategy.config.name, "Test Strategy");
        assert_eq!(strategy.position_size, 0.0);
        assert_eq!(strategy.equity, 100000.0);
    }

    #[test]
    fn test_entry_long() {
        let mut strategy = Strategy::default_with_name("Test");
        strategy
            .entry_long(0, Some(10.0), Some(100.0), Some("Entry".to_string()))
            .unwrap();

        assert_eq!(strategy.position_size, 10.0);
        assert_eq!(strategy.entries.len(), 1);
        assert_eq!(strategy.position_direction(), Direction::Long);
    }

    #[test]
    fn test_entry_short() {
        let mut strategy = Strategy::default_with_name("Test");
        strategy
            .entry_short(0, Some(10.0), Some(100.0), Some("Short Entry".to_string()))
            .unwrap();

        assert_eq!(strategy.position_size, -10.0);
        assert_eq!(strategy.entries.len(), 1);
        assert_eq!(strategy.position_direction(), Direction::Short);
    }

    #[test]
    fn test_close_position() {
        let mut strategy = Strategy::default_with_name("Test");
        strategy
            .entry_long(0, Some(10.0), Some(100.0), None)
            .unwrap();
        strategy
            .close(1, Some(110.0), Some("Take Profit".to_string()))
            .unwrap();

        assert_eq!(strategy.position_size, 0.0);
        assert_eq!(strategy.exits.len(), 1);
        assert!(strategy.open_trades.is_empty());
    }

    #[test]
    fn test_pyramiding() {
        let mut config = StrategyConfig::default();
        config.pyramiding = 2;
        let mut strategy = Strategy::new(config);

        // Should allow 2 entries
        strategy
            .entry_long(0, Some(1.0), Some(100.0), None)
            .unwrap();
        strategy
            .entry_long(1, Some(1.0), Some(101.0), None)
            .unwrap();
        // Third entry should be ignored due to pyramiding limit
        strategy
            .entry_long(2, Some(1.0), Some(102.0), None)
            .unwrap();

        assert_eq!(strategy.entries.len(), 2);
    }

    #[test]
    fn test_direction_reversal() {
        let mut strategy = Strategy::default_with_name("Test");

        // Enter long
        strategy
            .entry_long(0, Some(10.0), Some(100.0), None)
            .unwrap();
        assert_eq!(strategy.position_direction(), Direction::Long);

        // Enter short should close long first
        strategy
            .entry_short(1, Some(5.0), Some(105.0), None)
            .unwrap();
        assert_eq!(strategy.position_direction(), Direction::Short);
        assert_eq!(strategy.exits.len(), 1); // Long position was closed
    }
}
