pub mod message;

use redis::{AsyncCommands, Client};
use serde_json;
use crossbeam::channel::{Receiver, Sender};
use crate::matching_engine::messages::{EngineMessage, EngineResponse};
use self::message::{RedisOrderRequest, RedisOrderResponse, RedisMarketUpdate, RedisTradeInfo};

pub struct RedisService {
    client: Client,
    order_sender: Sender<EngineMessage>,
    response_receiver: Receiver<EngineResponse>,
}

impl RedisService {
    pub fn new(
        redis_url: &str,
        order_sender: Sender<EngineMessage>,
        response_receiver: Receiver<EngineResponse>,
    ) -> Result<Self, redis::RedisError> {
        let client = Client::open(redis_url)?;

        Ok(RedisService {
            client, 
            order_sender,
            response_receiver,
        })
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut con = self.client.get_async_connection().await?;
        let response_receiver = self.response_receiver.clone();
        let client_clone = self.client.clone();

        tokio::spawn(async move {
            match client_clone.get_async_connection().await {
                Ok(mut response_con) => {
                    while let Ok(response) = response_receiver.recv() {
                        match response {
                            EngineResponse::OrderPlaced { order_id, trades } => {
                                let redis_response = RedisOrderResponse {
                                    request_id: order_id,
                                    success: true,
                                    order_id: Some(order_id),
                                    trades: trades.iter().map(RedisTradeInfo::from).collect(),
                                    error: None,
                                };

                                if let Ok(json_response) = serde_json::to_string(&redis_response) {
                                    let _: Result<(), _> = response_con.publish("order_response", json_response).await;
                                }

                                for trade in trades {
                                    let market_update = RedisMarketUpdate {
                                        market: "BTC_USD".to_string(),
                                        data: serde_json::to_value(&RedisTradeInfo::from(&trade)).unwrap(),
                                        update_type: "trade".to_string(),
                                        timestamp: chrono::Utc::now(),
                                    };

                                    if let Ok(json) = serde_json::to_string(&market_update) {
                                        let _: Result<(), _> = response_con.publish("market_updates", json).await;
                                    }
                                }
                            }

                            EngineResponse::Error { message } => {
                                let redis_response = RedisOrderResponse {
                                    request_id: uuid::Uuid::new_v4(),
                                    success: false,
                                    order_id: None,
                                    trades: vec![],
                                    error: Some(message),
                                };

                                if let Ok(json) = serde_json::to_string(&redis_response) {
                                    let _: Result<(), _> = response_con.publish("order_response", json).await;
                                }
                            }
                            
                            EngineResponse::OrderCancelled { .. } => {
                            }
                        }
                    }
                }
                Err(_) => {
                }
            }
        });

        loop {
            match con.blpop::<_, Vec<String>>("order_queue", 0.0).await {
                Ok(result) => {
                    if result.len() >= 2 {
                        let json_data = &result[1];

                        if let Ok(order_request) = serde_json::from_str::<RedisOrderRequest>(json_data) {
                            if let Ok((pair, price, order)) = order_request.to_engine_message() {
                                let engine_message = EngineMessage::PlaceOrder { 
                                    pair, 
                                    price, 
                                    order 
                                };

                                let _ = self.order_sender.send(engine_message);
                            }
                        }
                    }
                }
                Err(_) => {
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    
                    if let Ok(new_con) = self.client.get_async_connection().await {
                        con = new_con;
                    } else {
                        return Err("Failed to reconnect to Redis".into());
                    }
                }
            }
        }
    }
}