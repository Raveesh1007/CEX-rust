use std::collections::HashMap;
use crossbeam::channel::{Receiver, Sender, unbounded};
use rust_decimal::Decimal;
use crate::matching_engine::{
    orderbook::OrderBook,
    types::{TradingPair, Order},
    messages::{EngineMessage, EngineResponse, DatabaseMessage}
};


pub struct MatchingEngine{
    pub orderbooks: HashMap<TradingPair, OrderBook>,
    pub message_receiver: Receiver<EngineMessage>,
    pub message_sender: Sender<EngineResponse>,
    pub database_sender: Sender<DatabaseMessage>,
}

impl MatchingEngine {
    pub fn new() -> (
        Self,
        Sender<EngineMessage>,    // For sending orders to engine
        Receiver<EngineResponse>, // For receiving responses
        Receiver<DatabaseMessage> // For database worker
    ){
        let (msg_tx, msg_rx) = unbounded();
        let (resp_tx, resp_rx) = unbounded();
        let (db_tx, db_rx) = unbounded();


        let engine = Self{
            orderbooks: HashMap::new(),
            message_receiver: msg_rx,
            message_sender: resp_tx,
            database_sender: db_tx,
        };

        (engine, msg_tx, resp_rx, db_rx)
    }

    pub fn add_market(&mut self, pair: TradingPair) {
        self.orderbooks.insert(pair, OrderBook::new());
    }

    pub fn run(mut self){
        while let Ok(msg) = self.message_receiver.recv(){
            match msg{
                EngineMessage::PlaceOrder{pair, order, price} => {
                    self.handle_place_order(pair, order, price);
                },

                EngineMessage::CancelOrder{order_id} => {
                    self.handle_cancel_order(order_id);
                }
            }
        }
    }

    fn handle_place_order(&mut self, pair: TradingPair, order: Order, price: Decimal){
        let order_id = order.id;

        if let Some(orderbook) = self.orderbooks.get_mut(&pair){
            let trades = orderbook.add_order(price, order.clone());

            let response = EngineResponse::OrderPlaced{
                order_id,
                trades: trades.clone(),
            };

            let _ = self.message_sender.send(response);

            if !trades.is_empty(){
                let _ = self.database_sender.send(DatabaseMessage::SaveTrades(trades.clone()));
                let _ = self.database_sender.send(DatabaseMessage::UpdateBalances{
                    user_id: order.user_id.clone(),
                    trades,
                });
            }

            let _ = self.database_sender.send(DatabaseMessage::SaveOrder(order));
        } else {
            let response = EngineResponse::Error{
                message: format!("Market not found for pair: {:?}", pair),
            };

            let _ = self.message_sender.send(response);
        }
    }

    fn handle_cancel_order(&mut self, order_id: uuid::Uuid){
        let response = EngineResponse::OrderCanceled{order_id};
        let _ = self.message_sender.send(response);
    }
}


