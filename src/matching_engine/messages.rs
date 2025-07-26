use rust_decimal::Decimal;
use crate::matching_engine::types::{Order, Trade, TradingPair};


#[derive(Debug, Clone)]
pub enum EngineMessage{
    PlaceOrder{
        pair: TradingPair,
        order: Order,
        price: Decimal,
    },
    CancelOrder{
        order_id: uuid:: Uuid,
    },    
}


#[derive(Debug, Clone)]
pub enum EngineResponse{
    OrderPlaced{
        order_id: uuid:: Uuid,
        trades: Vec<Trade>,
    },
    OrderCanceled{
        order_id: uuid:: Uuid,
    },

    Error{
        message: String,
    },    
}

#[derive(Debug, Clone)]
pub enum DatabaseMessage {
    SaveTrades(Vec<Trade>),
    UpdateBalances {
        user_id: String,
        trades: Vec<Trade>,
    },
    SaveOrder(Order),
}