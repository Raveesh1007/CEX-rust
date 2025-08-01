mod matching_engine;
mod balance;
mod websocket; 
mod redis;
mod api;

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
use api::ApiService;

#[tokio::main] 
async fn main() {
    let (mut engine, order_sender, response_receiver, db_receiver, ws_receiver) = MatchingEngine::new();
    
    setup_markets_and_users(&mut engine);
    
    start_websocket_service(ws_receiver).await;
    start_redis_service(order_sender.clone(), response_receiver).await;
    start_api_service().await;
    
    start_matching_engine(engine);
    start_database_worker(db_receiver);
    
    send_initial_order(order_sender).await;
    
    tokio::time::sleep(Duration::from_secs(3600)).await;
}

fn setup_markets_and_users(engine: &mut MatchingEngine) {
    let btc_usd = TradingPair::new("BTC".to_string(), "USD".to_string());
    engine.add_market(btc_usd);
    
    let mut user1_balances = HashMap::new();
    user1_balances.insert("BTC".to_string(), Decimal::from(10));
    user1_balances.insert("USD".to_string(), Decimal::from(100000));
    engine.add_user("user123".to_string(), user1_balances);
    
    let mut user2_balances = HashMap::new();
    user2_balances.insert("BTC".to_string(), Decimal::from(5));
    user2_balances.insert("USD".to_string(), Decimal::from(50000));
    engine.add_user("user456".to_string(), user2_balances);
}

async fn start_websocket_service(ws_receiver: crossbeam::channel::Receiver<websocket::MarketDataEvent>) {
    let ws_server = WebSocketServer::new(ws_receiver);
    tokio::spawn(async move {
        let _ = ws_server.start("127.0.0.1:8080").await;
    });
}

async fn start_redis_service(
    order_sender: crossbeam::channel::Sender<EngineMessage>,
    response_receiver: crossbeam::channel::Receiver<EngineResponse>
) {
    let redis_service = RedisService::new(
        "redis://127.0.0.1:6379/",
        order_sender,
        response_receiver,
    ).expect("Failed to create Redis service");
    
    tokio::spawn(async move {
        let _ = redis_service.start().await;
    });
}

async fn start_api_service() {
    let api_service = ApiService::new("redis://127.0.0.1:6379/")
        .expect("Failed to create API service");
    
    tokio::task::spawn_blocking(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            let _ = api_service.start("127.0.0.1:8000").await;
        });
    });
}

fn start_matching_engine(engine: MatchingEngine) {
    let _engine_handle = thread::spawn(move || {
        let mut engine = engine;  // Move the mut here
        engine.run();
    });
}

fn start_database_worker(db_receiver: crossbeam::channel::Receiver<DatabaseMessage>) {
    let _db_handle = thread::spawn(move || {
        while let Ok(_) = db_receiver.recv() {
        }
    });
}

async fn send_initial_order(order_sender: crossbeam::channel::Sender<EngineMessage>) {
    let btc_usd = TradingPair::new("BTC".to_string(), "USD".to_string());
    let sell_order = Order::new(BidOrAsk::Ask, Decimal::from(2));
    
    let _ = order_sender.send(EngineMessage::PlaceOrder {
        pair: btc_usd,
        price: Decimal::from(50000),
        order: sell_order,
    });
}