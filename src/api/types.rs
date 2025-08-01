use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct PlaceOrderRequest {
    pub user_id: String,
    pub order_type: String,
    pub market: String,
    pub side: String,
    pub price: Option<Decimal>,
    pub quantity: Decimal,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlaceOrderResponse{
    pub success: bool,
    pub order_id: Option<String>,
    pub trades: Vec<TradeInfo>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BalanceResponse {
    pub user_id: String,
    pub balances: std::collections::HashMap<String, BalanceInfo>,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct TradeInfo{
    pub trade_id: String,
    pub price: Decimal,
    pub quantity: Decimal,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DepthResponse {
    pub market: String,
    pub bids: Vec<PriceLevel>,
    pub asks: Vec<PriceLevel>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PriceLevel {
    pub price: Decimal,
    pub quantity: Decimal,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct RecentTradesResponse{
    pub market: String,
    pub trades: Vec<TradeInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BalanceInfo{
    pub available: Decimal,
    pub locked: Decimal,
    pub total: Decimal,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TickerResponse{
    pub market: String,
    pub last_price: Decimal,
    pub volume_24h: Decimal,
    pub price_change_24h: Decimal,
    pub high_24h: Decimal,
    pub low_24h: Decimal,
    pub timestamp: DateTime<Utc>,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    pub error: String,
    pub code: u16,
    pub timestamp: DateTime<Utc>,
}


impl ApiError{
    pub fn new(error: String, code: u16) -> Self{
        ApiError{
            error: error.to_string(),
            code,
            timestamp: Utc::now(),
        }
    }
}




