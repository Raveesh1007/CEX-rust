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

impl RedisService{
    pub fn new(redis_url: &str,
        order_sender: Sender<EngineMessage>,
        response_receiver: Receiver<EngineResponse>,
    ) -> Result<Self, redis::RedisError>{
        let client = Client::open(redis_url)?;

        Ok(RedisService{
            client, 
            order_sender,
            response_receiver,
        })
    }


    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>>{
        println!("Starting Redis service...");

        let mut con = self.client.get_async_connection().await?;

        let response_receiver = self.response_receiver.clone();
        let client_clone = self.client.clone();

        tokio::spawn(async move{
            let mut response_con = client_clone.get_async_connection().await.unwrap();

            while let Ok(response) = response_receiver.recv(){
                match response{
                    EngineResponse::OrderPlaced{order_id, trades} => {
                        let redis_response = RedisOrderResponse{
                            request_id: order_id,
                            success: true,
                            order_id: Some(order_id),
                            trades: trades.iter().map(RedisTradeInfo::from).collect(),
                            error: None,
                        };

                        let json_response = serde_json::to_string(&redis_response).unwrap();
                        let _: () = response_con.publish("order_response", json_response).await.unwrap();


                        for trade in trades{
                            let market_update = RedisMarketUpdate{
                                market: "BTC_USD".to_string(),
                                data: serde_json::to_value(&RedisTradeInfo::from(&trade)).unwrap(),
                                update_type: "trade".to_string(),
                                timestamp: chrono::Utc::now(),
                            };

                            let json = serde_json::to_string(&market_update).unwrap();
                            let _: () = response_con.publish("market_updates", json).await.unwrap();
                        }
                    }

                    EngineResponse::Error{message} => {
                        let redis_response = RedisOrderResponse{
                            request_id: uuid::Uuid::new_v4(),
                            success: false,
                            order_id: None,
                            trades: vec![],
                            error: Some(message),
                        };

                        let json = serde_json::to_string(&redis_response).unwrap();
                        let _: () =  response_con.publish("order_response", json).await.unwrap();
                    }
                    _ => {}
                }
            }
        });


        loop{
            let result: Vec<String> = con.blpop("order_queue", 1.0).await?;

            if result.len() >= 2{
                let json_data = &result[1];


                if let Ok(order_request) = serde_json::from_str::<RedisOrderRequest>(json_data) {
                    println!("Recieved order request: {:?}", order_request);

                    match order_request.to_engine_message(){
                        Ok((pair, price, order)) => {
                            let engine_message = EngineMessage::PlaceOrder{pair, price, order};

                            if let Err(e) = self.order_sender.send(engine_message){
                                println!("Error sending order to engine: {:?}", e);
                            }
                        }

                        Err(e) => {
                            println!("Error parsing order request: {:?}", e);
                        }
                    }
                }else{
                    println!("Invalid order request format");
                }
            }
        }
    }
}