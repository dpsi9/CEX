use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Market {
    pub pair: String,
    pub base_asset: String,
    pub quote_asset: String,
    pub tick_size: String,
    pub lot_size: String,
}
