use std::collections::BTreeMap;
use rust_decimal::Decimal;
use crate::matching_engine::types::{Order, Limit, BidOrAsk, Trade};


#[derive(Debug)]
pub struct OrderBook{
    pub bids: BTreeMap<Decimal, Limit>,
    pub asks: BTreeMap<Decimal, Limit>,
}


impl OrderBook{
    pub fn new() -> OrderBook{
        OrderBook{
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }


    pub fn add_order(&mut self, price: Decimal, mut order: Order) -> Vec<Trade> {
        let mut trades = Vec::new();
        
        match order.bid_or_ask {
            BidOrAsk::Bid => {
                // Actually call the matching logic!
                trades.extend(self.try_match_buy_order(&mut order, price));
                
                // Only add if there's remaining quantity
                if order.size > Decimal::ZERO {
                    let limit = self.bids.entry(price).or_insert_with(|| Limit::new(price));
                    limit.add_order(order);
                }
            },
            BidOrAsk::Ask => {
                trades.extend(self.try_match_sell_order(&mut order, price));
                
                if order.size > Decimal::ZERO {
                    let limit = self.asks.entry(price).or_insert_with(|| Limit::new(price));
                    limit.add_order(order);
                }
            }
        }
        
        trades
    }

    fn try_match_buy_order(&mut self, buy_order: &mut Order, buy_price: Decimal) -> Vec<Trade> {
        let mut trades = Vec::new();
        let mut prices_to_remove = Vec::new();
        
        // BTreeMap already iterates in sorted order (lowest to highest)
        for (ask_price, limit) in self.asks.iter_mut() {
            if buy_price < *ask_price || buy_order.size == Decimal::ZERO {
                break;
            }
            
            let trade_result = OrderBook::match_orders_at_price(buy_order, limit, *ask_price);
            trades.extend(trade_result);
            
            if limit.orders.is_empty() {
                prices_to_remove.push(*ask_price);
            }
        }
        
        for price in prices_to_remove {
            self.asks.remove(&price);
        }
        
        trades
    }

    fn try_match_sell_order(&mut self, sell_order: &mut Order, sell_price: Decimal) -> Vec<Trade> {
        let mut trades = Vec::new();
        let mut prices_to_remove = Vec::new();
        
        let mut bid_prices: Vec<Decimal> = self.bids.keys().cloned().collect();
        bid_prices.sort_by(|a, b| b.cmp(a));
        
        for bid_price in bid_prices {
            if sell_price > bid_price || sell_order.size == Decimal::ZERO {
                break;
            }
            
            if let Some(limit) = self.bids.get_mut(&bid_price) {
                let trade_result = OrderBook::match_orders_at_price(sell_order, limit, bid_price);
                trades.extend(trade_result);
                
                if limit.orders.is_empty() {
                    prices_to_remove.push(bid_price);
                }
            }
        }

        for price in prices_to_remove {
            self.bids.remove(&price);
        }
        
        trades
    }



    fn match_orders_at_price(incoming_order: &mut Order, limit: &mut Limit, price: Decimal) -> Vec<Trade>{
        let mut trades = Vec::new();
        let mut orders_to_remove = Vec::new();

        for (i, existing_order) in limit.orders.iter_mut().enumerate() {
            if incoming_order.size == Decimal::ZERO{
                break;
            }

            let trade_quantity = incoming_order.size.min(existing_order.size);

            let (buyer_id, seller_id) = match incoming_order.bid_or_ask {
                BidOrAsk::Bid => (incoming_order.id, existing_order.id),
                BidOrAsk::Ask => (existing_order.id, incoming_order.id),
            };

            trades.push(Trade::new(buyer_id, seller_id, price, trade_quantity ));

            incoming_order.size -= trade_quantity;
            existing_order.size -= trade_quantity;

            if existing_order.size == Decimal::ZERO{
                orders_to_remove.push(i);
            }
        }

        for &index in orders_to_remove.iter().rev(){
            limit.orders.remove(index);
        }

        trades
    }
}

