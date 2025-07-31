use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use chrono::{DateTime, Utc};
use crate::matching_engine::types::{Trade, TradingPair};


#[derive(Debug,Clone, Serialize, Deserialize)]
#[serde(tag = "type")]

pub enum MarketDataEvent{

    #[serde(rename = "trade")]
    Trade{
        pair: String,
        price: Decimal,
        quantity: Decimal,
        timestamp: DateTime<Utc>,
        trade_id: String,
    },


    #[serde(rename = "depth")]
    Depth{
        pair: String,
        bids: Vec<PriceLevel>,
        asks: Vec<PriceLevel>,
        timestamp: DateTime<Utc>,
    },


    #[serde(rename = "ticker")]
    Ticker{
        pair: String,
        last_price: Decimal,
        volume_24h: Decimal,
        high_24h: Decimal,
        price_change_24h: Decimal,
        low_24h: Decimal,
        timestamp: DateTime<Utc>,
    },

    #[serde(rename = "order_update")]
    OrderUpdate{
        order_id: String,
        user_id: String,
        status: OrderStatus,
        filled_quantity: Decimal,
        remaining_quantity: Decimal,
        timestamp: DateTime<Utc>,
    },
}



#[derive(Debug,Clone, Serialize, Deserialize)]
pub struct PriceLevel{
    pub price: Decimal,
    pub quantity: Decimal,
}

#[derive(Debug,Clone, Serialize, Deserialize)]
pub enum OrderStatus{
    #[serde(rename = "placed")]
    Placed,
    #[serde(rename = "filled" )]
    Filled,
    #[serde(rename = "partial_filled")]
    PartialFilled,
    #[serde(rename = "cancelled")]
    Cancelled,
}



impl MarketDataEvent {
    pub fn from_trade(trade: &Trade, pair: &TradingPair) -> Self{
        MarketDataEvent::Trade{
            pair: format!("{}/{}", pair.base, pair.quote),
            price: trade.price,
            quantity: trade.quantity,
            timestamp: trade.timestamp,
            trade_id: trade.id.to_string(),
        }
    }

    pub fn to_json(&self) -> String{
        serde_json::to_string(self).unwrap_or_else(|_|"{}".to_string())
    }
}

