mod matching_engine;
mod balance;
mod websocket; 

use std::thread;
use std::time::Duration;
use std::collections::HashMap;
use rust_decimal::Decimal;
use matching_engine::{
    engine::MatchingEngine,
    types::{Order, BidOrAsk, TradingPair},
    messages::{EngineMessage, EngineResponse, DatabaseMessage}
};
use websocket::WebSocketServer; 

#[tokio::main] 
async fn main() {
    println!("Starting WebSocket-Enabled Trading Engine...\n");
    
    let (mut engine, order_sender, response_receiver, db_receiver, ws_receiver) = MatchingEngine::new();
    
    let btc_usd = TradingPair::new("BTC".to_string(), "USD".to_string());
    engine.add_market(btc_usd.clone());
    
    let mut user1_balances = HashMap::new();
    user1_balances.insert("BTC".to_string(), Decimal::from(10));
    user1_balances.insert("USD".to_string(), Decimal::from(100000));
    engine.add_user("user123".to_string(), user1_balances);
    
    println!("Added user with 10 BTC and $100,000 USD");
    
    let ws_server = WebSocketServer::new(ws_receiver);
    tokio::spawn(async move {
        if let Err(e) = ws_server.start("127.0.0.1:8080").await {
            println!("WebSocket server error: {}", e);
        }
    });
    
    println!("WebSocket server started on ws://127.0.0.1:8080");
    
    let _engine_handle = thread::spawn(move || {
        println!("Matching Engine started with balance validation");
        engine.run();
    });
    
    let _db_handle = thread::spawn(move || {
        println!("Database worker started");
        while let Ok(db_msg) = db_receiver.recv() {
            match db_msg {
                DatabaseMessage::SaveTrades(trades) => {
                    println!("DB: Saving {} trades", trades.len());
                    for trade in trades {
                        println!("   - Trade: {} {} at ${}", trade.quantity, "BTC", trade.price);
                    }
                },
                DatabaseMessage::UpdateBalances { user_id, trades: _ } => {
                    println!("DB: Updating balances for user: {}", user_id);
                },
                DatabaseMessage::SaveOrder(order) => {
                    println!("DB: Saving order {} ({:?})", order.id, order.bid_or_ask);
                }
            }
        }
    });
    
    println!("Testing balance validation with WebSocket broadcasting...\n");
    
    // Test orders
    let sell_order = Order::new(BidOrAsk::Ask, Decimal::from(2));
    order_sender.send(EngineMessage::PlaceOrder {
        pair: btc_usd.clone(),
        price: Decimal::from(50000),
        order: sell_order,
    }).unwrap();
    
    let buy_order = Order::new(BidOrAsk::Bid, Decimal::from(1));
    order_sender.send(EngineMessage::PlaceOrder {
        pair: btc_usd,
        price: Decimal::from(50000),
        order: buy_order,
    }).unwrap();
    
    println!("Listening for responses...\n");
    
    for i in 0..2 {
        match response_receiver.recv_timeout(Duration::from_secs(1)) {
            Ok(response) => {
                match response {
                    EngineResponse::OrderPlaced { order_id, trades } => {
                        println!("Order {} placed successfully!", order_id);
                        if !trades.is_empty() {
                            println!("   - {} trades executed", trades.len());
                            println!("   - Trade events broadcasted via WebSocket!");
                        }
                    },
                    EngineResponse::Error { message } => {
                        println!("Order rejected: {}", message);
                    },
                    _ => {}
                }
            },
            Err(_) => {
                println!("Timeout waiting for response {}", i + 1);
                break;
            }
        }
    }
    
    println!("\n WebSocket-enabled trading engine running!");
    println!("Connect to ws://127.0.0.1:8080 to see live market data");
    println!("   Press Ctrl+C to stop...");
    
    tokio::time::sleep(Duration::from_secs(3600)).await;
}