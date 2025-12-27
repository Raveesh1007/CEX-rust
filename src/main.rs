mod matching_engine;
mod balance;
mod websocket; 
mod redis;
mod api;
mod database;

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
use database::Database;

#[tokio::main] 
async fn main() {
    dotenvy::dotenv().expect("Failed to load .env file");
    
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env file");
    
    let redis_url = std::env::var("REDIS_URL")
        .expect("REDIS_URL must be set in .env file");
    
    let api_host = std::env::var("API_HOST")
        .unwrap_or_else(|_| "127.0.0.1".to_string());
    
    let api_port = std::env::var("API_PORT")
        .unwrap_or_else(|_| "8000".to_string());
    
    let websocket_host = std::env::var("WEBSOCKET_HOST")
        .unwrap_or_else(|_| "127.0.0.1".to_string());
    
    let websocket_port = std::env::var("WEBSOCKET_PORT")
        .unwrap_or_else(|_| "8080".to_string());
    
    match Database::new(&database_url).await {
        Ok(_db) => {
            println!("Database connected successfully!");
        }
        Err(e) => {
            println!("Failed to connect to database: {}", e);
            return;
        }
    }
    
    let (mut engine, order_sender, response_receiver, db_receiver, ws_receiver) = MatchingEngine::new();
    
    setup_markets_and_users(&mut engine);
    
    start_websocket_service(ws_receiver, &websocket_host, &websocket_port).await;
    start_redis_service(order_sender.clone(), response_receiver, &redis_url).await;
    start_api_service(&redis_url, &api_host, &api_port).await;
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

async fn start_websocket_service(
    ws_receiver: crossbeam::channel::Receiver<websocket::MarketDataEvent>,
    host: &str,
    port: &str
) {
    let addr = format!("{}:{}", host, port);
    let ws_server = WebSocketServer::new(ws_receiver);
    tokio::spawn(async move {
        let _ = ws_server.start(&addr).await;
    });
}

async fn start_redis_service(
    order_sender: crossbeam::channel::Sender<EngineMessage>,
    response_receiver: crossbeam::channel::Receiver<EngineResponse>,
    redis_url: &str
) {
    let redis_service = RedisService::new(
        redis_url,
        order_sender,
        response_receiver,
    ).expect("Failed to create Redis service");
    
    tokio::spawn(async move {
        let _ = redis_service.start().await;
    });
}

async fn start_api_service(redis_url: &str, host: &str, port: &str) {
    let api_service = ApiService::new(redis_url)
        .expect("Failed to create API service");
    
    let bind_addr = format!("{}:{}", host, port);
    
    tokio::task::spawn_blocking(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            let _ = api_service.start(&bind_addr).await;
        });
    });
}

fn start_matching_engine(engine: MatchingEngine) {
    let _engine_handle = thread::spawn(move || {
        let mut engine = engine;
        engine.run();
    });
}

fn start_database_worker(db_receiver: crossbeam::channel::Receiver<DatabaseMessage>) {
    let _db_handle = thread::spawn(move || {
        while let Ok(_) = db_receiver.recv() {
            // Process database operations silently
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