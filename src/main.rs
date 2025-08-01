mod matching_engine;
mod balance;
mod websocket; 
mod redis;

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
use redis::RedisService;

#[tokio::main] 
async fn main() {
    println!("Starting Redis-Enabled Trading Engine...\n");
    
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
    
    let redis_service = RedisService::new(
        "redis://127.0.0.1:6379/",
        order_sender.clone(),
        response_receiver,
    ).expect("Failed to create Redis service");
    
    tokio::spawn(async move {
        if let Err(e) = redis_service.start().await {
            println!("Redis service error: {}", e);
        }
    });
    println!("Redis service started - listening for orders on 'order_queue'");
    
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
    
    println!("Sending test order directly to engine...\n");
    
    let sell_order = Order::new(BidOrAsk::Ask, Decimal::from(2));
    if let Err(e) = order_sender.send(EngineMessage::PlaceOrder {
        pair: btc_usd.clone(),
        price: Decimal::from(50000),
        order: sell_order,
    }) {
        println!("Failed to send order: {}", e);
    }
    
    println!("All services started!");
    println!("WebSocket: ws://127.0.0.1:8080");
    println!("Redis: Listening on 'order_queue'");
    println!("Engine: Processing orders");
    println!("Database: Persisting data");
    println!("\n To test via Redis, send JSON to 'order_queue':");
    println!(r#"   redis-cli LPUSH order_queue '{{"id":"test1","user_id":"user123","market":"BTC_USD","side":"buy","order_type":"limit","price":49000,"quantity":1,"timestamp":"2024-01-01T00:00:00Z"}}'"#);
    println!("\n   Press Ctrl+C to stop...");
    

    tokio::time::sleep(Duration::from_secs(3600)).await;
}