//! Real-time data manager
//! Combines historical data with live WebSocket updates.

use std::collections::VecDeque;
use std::sync::Arc;

use tokio::sync::{broadcast, RwLock};

use crate::data::{BarUpdate, BinanceClient, BinanceWsClient, OhlcvBar};

/// Real-time data manager state
pub struct RealtimeDataManager {
    symbol: String,
    interval: String,
    max_bars: usize,
    bars: RwLock<VecDeque<OhlcvBar>>,
    tx: broadcast::Sender<RealtimeUpdate>,
}

/// Update event sent to clients
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum RealtimeUpdate {
    /// Full snapshot of all bars
    Snapshot {
        bars: Vec<OhlcvBar>,
    },
    /// Update to the forming bar
    FormingUpdate {
        bar: OhlcvBar,
    },
    /// New bar added (previous closed)
    NewBar {
        closed_bar: OhlcvBar,
        new_bar: OhlcvBar,
    },
}

impl RealtimeDataManager {
    /// Create a new real-time data manager
    pub fn new(symbol: String, interval: String, max_bars: usize) -> Self {
        let (tx, _rx) = broadcast::channel(100);

        Self {
            symbol,
            interval,
            max_bars,
            bars: RwLock::new(VecDeque::with_capacity(max_bars)),
            tx,
        }
    }

    /// Subscribe to real-time updates
    pub fn subscribe(&self) -> broadcast::Receiver<RealtimeUpdate> {
        self.tx.subscribe()
    }

    /// Get current bars snapshot
    pub async fn get_bars(&self) -> Vec<OhlcvBar> {
        self.bars.read().await.iter().cloned().collect()
    }

    /// Initialize with historical data
    pub async fn load_historical(&self, client: &BinanceClient) -> Result<(), Box<dyn std::error::Error>> {
        let bars = client.fetch_klines(&self.symbol, &self.interval, self.max_bars).await?;

        let mut bars_lock = self.bars.write().await;
        bars_lock.clear();
        bars_lock.extend(bars);

        // Send snapshot to subscribers
        let snapshot = RealtimeUpdate::Snapshot {
            bars: bars_lock.iter().cloned().collect(),
        };
        let _ = self.tx.send(snapshot);

        Ok(())
    }

    /// Start processing WebSocket updates
    pub async fn start(self: Arc<Self>, ws_client: Arc<BinanceWsClient>) {
        let mut rx = ws_client.subscribe();

        tokio::spawn(async move {
            while let Ok(update) = rx.recv().await {
                self.handle_update(update).await;
            }
        });
    }

    /// Handle a BarUpdate from WebSocket
    async fn handle_update(&self, update: BarUpdate) {
        match update {
            BarUpdate::Forming { bar, is_new } => {
                let mut bars = self.bars.write().await;

                if is_new {
                    // This is a new forming bar - add it
                    bars.push_back(bar.clone());

                    // Trim if too many bars
                    while bars.len() > self.max_bars {
                        bars.pop_front();
                    }

                    // If we had a previous bar, it's now closed
                    if bars.len() >= 2 {
                        let closed_idx = bars.len() - 2;
                        let closed_bar = bars[closed_idx].clone();
                        let _ = self.tx.send(RealtimeUpdate::NewBar {
                            closed_bar,
                            new_bar: bar.clone(),
                        });
                    }
                } else {
                    // Update the existing forming bar (last one)
                    if let Some(last) = bars.back_mut() {
                        if last.time == bar.time {
                            *last = bar.clone();
                        }
                    }

                    let _ = self.tx.send(RealtimeUpdate::FormingUpdate { bar });
                }
            }
            BarUpdate::Closed { bar } => {
                // This bar is now closed - update it in our list
                let mut bars = self.bars.write().await;

                // Find and update the closed bar
                if let Some(pos) = bars.iter_mut().position(|b| b.time == bar.time) {
                    bars[pos] = bar.clone();
                }

                // Send update
                let _ = self.tx.send(RealtimeUpdate::FormingUpdate { bar });
            }
        }
    }
}
