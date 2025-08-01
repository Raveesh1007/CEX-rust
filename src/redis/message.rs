use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use uuid::Uuid;
use crate::matching_engine::types::{Trade, TradingPair, Order, BidOrAsk};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisOrderRequest{
    pub id: String,
    pub user_id: String,
    pub market: String,
    pub side: String,
    pub order_type: String,
    pub price: Option<Decimal>,
    pub quantity: Decimal,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisOrderResponse {
    pub request_id: Uuid,
    pub success: bool,
    pub order_id: Option<Uuid>,
    pub trades: Vec<RedisTradeInfo>,
    pub error: Option<String>,
}



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisTradeInfo {
    pub trade_id: String,
    pub price: Decimal,
    pub quantity: Decimal,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub buyer_order_id: String,
    pub seller_order_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisMarketUpdate{
    pub market: String,
    pub update_type: String,
    pub data: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl RedisOrderRequest{
    pub fn to_engine_message(&self) -> Result<(TradingPair, Decimal,Order), String>{
        let parts: Vec<&str> = self.market.split('_').collect();
        if parts.len() != 2{
            return Err("Invalid market format".to_string());
        }

        let pair = TradingPair::new(parts[0].to_string(), parts[1].to_string());

        let side = match self.side.as_str(){
            "buy" => BidOrAsk::Bid,
            "sell" => BidOrAsk::Ask,
            _ => return Err("Invalid side".to_string()),
        };

        let price = match self.order_type.as_str(){
            "limit" => self.price.ok_or("Price is required for limit orders")?,
            "market" => Decimal::from(0),
            _ => return Err("Invalid order type".to_string()),
        };


        let mut order = Order::new(side, self.quantity);
        order.user_id = self.user_id.clone();

        Ok((pair, price, order))
    }
}


impl From<&Trade> for RedisTradeInfo{
    fn from(trade: &Trade) -> Self{
        RedisTradeInfo{
            trade_id: trade.id.to_string(),
            price: trade.price,
            quantity: trade.quantity,
            timestamp: trade.timestamp,
            buyer_order_id: trade.buyer_order_id.to_string(),
            seller_order_id: trade.seller_order_id.to_string(),
        }
    }
}


