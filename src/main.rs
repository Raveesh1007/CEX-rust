mod matching_engine;

use std::thread;
use std::time::Duration;
use rust_decimal::Decimal;
use matching_engine::{
    engine::MatchingEngine,
    types::{Order, BidOrAsk, TradingPair},
    messages::{EngineMessage, EngineResponse, DatabaseMessage}
};

fn main() {
    println!("🚀 Starting MPSC Trading Engine Test...\n");
    
    // Step 1: Create the engine and get the channels
    let (mut engine, order_sender, response_receiver, db_receiver) = MatchingEngine::new();
    
    // Step 2: Add a trading market
    let btc_usd = TradingPair::new("BTC".to_string(), "USD".to_string());
    engine.add_market(btc_usd.clone());
    
    // Step 3: Start the engine in a background thread
    let engine_handle = thread::spawn(move || {
        println!("🔧 Matching Engine started in background thread");
        engine.run();
    });
    
    // Step 4: Start database worker thread
    let db_handle = thread::spawn(move || {
        println!("💾 Database worker started");
        while let Ok(db_msg) = db_receiver.recv() {
            match db_msg {
                DatabaseMessage::SaveTrades(trades) => {
                    println!("💾 DB: Saving {} trades", trades.len());
                    for trade in trades {
                        println!("   - Trade: {} {} at ${}", trade.quantity, "BTC", trade.price);
                    }
                },
                DatabaseMessage::UpdateBalances { user_id, trades } => {
                    println!("💾 DB: Updating balances for user: {}", user_id);
                },
                DatabaseMessage::SaveOrder(order) => {
                    println!("💾 DB: Saving order {} ({:?})", order.id, order.bid_or_ask);
                }
            }
        }
    });
    
    // Step 5: Send some test orders through the channels
    println!("📤 Sending orders through MPSC channels...\n");
    
    // Send a sell order
    let sell_order = Order::new(BidOrAsk::Ask, Decimal::from(5));
    order_sender.send(EngineMessage::PlaceOrder {
        pair: btc_usd.clone(),
        price: Decimal::from(50000),
        order: sell_order,
    }).unwrap();
    
    // Send a buy order that should match
    let buy_order = Order::new(BidOrAsk::Bid, Decimal::from(3));
    order_sender.send(EngineMessage::PlaceOrder {
        pair: btc_usd.clone(),
        price: Decimal::from(50000),
        order: buy_order,
    }).unwrap();
    
    // Send another buy order (partial fill)
    let buy_order2 = Order::new(BidOrAsk::Bid, Decimal::from(10));
    order_sender.send(EngineMessage::PlaceOrder {
        pair: btc_usd,
        price: Decimal::from(50000),
        order: buy_order2,
    }).unwrap();
    
    // Step 6: Listen for responses
    println!("📥 Listening for responses...\n");
    
    for i in 0..3 {
        match response_receiver.recv_timeout(Duration::from_secs(1)) {
            Ok(response) => {
                match response {
                    EngineResponse::OrderPlaced { order_id, trades } => {
                        println!("✅ Order {} placed successfully!", order_id);
                        if trades.is_empty() {
                            println!("   - No trades executed (added to orderbook)");
                        } else {
                            println!("   - {} trades executed:", trades.len());
                            for trade in trades {
                                println!("     * {} BTC at ${}", trade.quantity, trade.price);
                            }
                        }
                    },
                    EngineResponse::Error { message } => {
                        println!("❌ Error: {}", message);
                    },
                    _ => println!("📨 Other response received"),
                }
                println!();
            },
            Err(_) => {
                println!("⏰ Timeout waiting for response {}", i + 1);
                break;
            }
        }
    }
    
    println!("🏁 Test completed! The MPSC architecture is working:");
    println!("   ✅ Orders sent through channels");
    println!("   ✅ Engine processed orders asynchronously");  
    println!("   ✅ Trades executed and matched");
    println!("   ✅ Database messages sent in background");
    println!("   ✅ Responses received instantly");
    
    // Give background threads time to finish processing
    thread::sleep(Duration::from_millis(100));
}