use std::collections::BTreeMap;

use rust_decimal::Decimal;
use shared::types::DepthLevel;

pub fn aggregate_levels(levels: &BTreeMap<Decimal, Decimal>) -> Vec<DepthLevel> {
    levels
        .iter()
        .map(|(price, qty)| DepthLevel {
            price: *price,
            quantity: *qty,
        })
        .collect()
}
