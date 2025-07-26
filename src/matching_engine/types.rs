use std::collections::HashMap;
use uuid::Uuid;
use rust_decimal::Decimal;


#[derive(Debug, Clone, PartialEq)]
pub enum BidOrAsk{
    Bid,
    Ask,
}


#[derive(Debug, Clone)]
pub struct Order{
    pub id: Uuid,
    pub user_id: String,
    pub bid_or_ask : BidOrAsk,
    pub size: Decimal,
}


impl Order{
    pub fn new(bid_or_ask: BidOrAsk, size: Decimal) -> Order {
        Order{
            id: Uuid::new_v4(),
            user_id: "user123".to_string(),
            bid_or_ask,
            size,
        }
    }
}


impl Limit{
    pub fn new(price: Decimal) -> Limit{
        Limit{
            price,
            orders: Vec::new(),
        }
    }


    pub fn add_order(&mut self, order: Order){
        self.orders.push(order);
    }

    pub fn total_volume(&self) -> Decimal{
        self.orders.iter().map(|o| o.size).sum()
    }
}


#[derive(Debug, Clone)]
pub struct Trade{
    pub id: Uuid,
    pub buyer_order_id: Uuid,
    pub seller_order_id: Uuid,
    pub price: Decimal,
    pub quantity: Decimal,
    pub timestamp: DateTime<Utc>,
}


impl Trade{
    pub fn new(buyer_order_id: Uuid, seller_order_id: Uuid, price: Decimal, quantity: Decimal) -> self{
        Trade{
            id: Uuid::new_v4(),
            buyer_order_id,
            seller_order_id,
            price,
            quantity,
            timestamp: Utc::now(),
        }
    }
}