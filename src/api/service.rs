use actix_web::{web, App, HttpServer, HttpResponse, Result, middleware::Logger};
use actix_cors::Cors;
use redis::{AsyncCommands, Client};
use serde_json;
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;

use crate::api::types::*;
use crate::redis::message::RedisOrderRequest;


pub struct ApiService{
    redis_client: Arc<Client>,
}


impl ApiService{
    pub fn new(redis_url: &str) -> Result<Self, redis::RedisError>{
        let client = Client::open(redis_url)?;

        Ok(ApiService{
            redis_client: Arc::new(client),
        })
    } 

    pub async fn start(&self, bind_address: &str) -> std::io::Result<()> {
        println!("Starting REST API server on http://{}", bind_address);

        let redis_client = Arc::clone(&self.redis_client);

        HttpServer::new(move||{
            App::new()
            .app_data(web::Data::new(redis_client.clone()))
            .wrap(Logger::default())
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header()
            )
            .service(web::scope("/api/v1")
                .route("/order", web::post().to(place_order))
                .route("/depth/{market}", web::get().to(get_depth))
                .route("/trades/{market}", web::get().to(get_recent_trades))
                .route("/balance/{user_id}", web::get().to(get_balance))
                .route("/tickers", web::get().to(get_tickers))
                .route("/tickers/{market}", web::get().to(get_ticker))
                .route("/health", web::get().to(health_check))
            )
        })
        .bind(bind_address)?
        .run()
        .await
    }
}


async fn place_order(
    redis_client: web::Data<Arc<Client>>,
    order_req: web::Json<PlaceOrderRequest>,
) -> Result<HttpResponse>{
    println!("Received order request: {:?}", order_req);

    let order_id = Uuid::new_v4().to_string();


    let redis_order = RedisOrderRequest {
        id: order_id.clone(),
        user_id: order_req.user_id.clone(),
        market: order_req.market.clone(),
        side: order_req.side.clone(),
        order_type: order_req.order_type.clone(),
        price: order_req.price,
        quantity: order_req.quantity,
        timestamp: Utc::now(),
    };

    match redis_client.get_async_connection().await{
        Ok(mut con) => {
            let json = serde_json::to_string(&redis_order).unwrap();

            match con.lpush::<_, _, ()>("order_queue", json).await {
                Ok(_) => {
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

                    let response = PlaceOrderResponse{
                        success: true,
                        order_id: Some(order_id),
                        trades: vec![],
                        error: None,
                    };

                    Ok(HttpResponse::Ok().json(response))
                }

                Err(e) => {
                    let error = ApiError::new(format!("Failed to queue order: {}", e), 500);
                    Ok(HttpResponse::InternalServerError().json(error))
                }
            }
        }

        Err(e) => {
            let error = ApiError::new(format!("Redis connection error: {}", e), 500);
            Ok(HttpResponse::InternalServerError().json(error))
        }
    }
}

async fn get_depth(
    _redis_client: web::Data<Arc<Client>>,
    path: web::Path<String>,
) -> Result<HttpResponse>{
    let market = path.into_inner();
    println!("Getting depth for market: {}", market);

    let response = DepthResponse {
        market: market.clone(),
        bids: vec![
            PriceLevel { price: rust_decimal::Decimal::from(49500), quantity: rust_decimal::Decimal::from(2) },
            PriceLevel { price: rust_decimal::Decimal::from(49400), quantity: rust_decimal::Decimal::from(1) },
        ],
        asks: vec![
            PriceLevel { price: rust_decimal::Decimal::from(50100), quantity: rust_decimal::Decimal::from(1) },
            PriceLevel { price: rust_decimal::Decimal::from(50200), quantity: rust_decimal::Decimal::from(3) },
        ],
        timestamp: Utc::now(),
    };
    Ok(HttpResponse::Ok().json(response))
}



async fn get_recent_trades(
    _redis_client: web::Data<Arc<Client>>,
    path: web::Path<String>,
) -> Result<HttpResponse>{
    let market = path.into_inner();
    println!("Getting recent trades for market: {}", market);

    let response = RecentTradesResponse{
        market: market.clone(),
        trades: vec![
            TradeInfo{
                trade_id: Uuid::new_v4().to_string(),
                price: rust_decimal::Decimal::from(50000),
                quantity: rust_decimal::Decimal::from(1),
                timestamp: Utc::now(),
            }
        ],
    };

    Ok(HttpResponse::Ok().json(response))
}


async fn get_balance(
    _redis_client: web::Data<Arc<Client>>,
    path: web::Path<String>,
) -> Result<HttpResponse>{
    let user_id = path.into_inner();
    println!("Getting balance for user: {}", user_id);

    let mut balances = std::collections:: HashMap::new();
    balances.insert("BTC".to_string(), BalanceInfo{
        available: rust_decimal::Decimal::from(8),
        locked: rust_decimal::Decimal::from(2),
        total: rust_decimal::Decimal::from(10),
    });

    balances.insert("USD".to_string(), BalanceInfo {
        available: rust_decimal::Decimal::from(75000),
        locked: rust_decimal::Decimal::from(25000),
        total: rust_decimal::Decimal::from(100000),
    });

    let response = BalanceResponse{
        user_id: user_id.clone(),
        balances,
    };

    Ok(HttpResponse::Ok().json(response))
}


async fn get_tickers(
    _redis_client: web::Data<Arc<Client>>,
) -> Result<HttpResponse>{
    println!("Getting tickers");

    let tickers = vec![
        TickerResponse{
            market: "BTCUSD".to_string(),
            last_price: rust_decimal::Decimal::from(50000),
            volume_24h: rust_decimal::Decimal::from(100),
            price_change_24h: rust_decimal::Decimal::from(1000),
            high_24h: rust_decimal::Decimal::from(51000),
            low_24h: rust_decimal::Decimal::from(49000),
            timestamp: Utc::now(),
        }
    ];
    Ok(HttpResponse::Ok().json(tickers))
}



async fn get_ticker(
    _redis_client: web::Data<Arc<Client>>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let market = path.into_inner();
    println!("Getting ticker for market: {}", market);
    
    let ticker = TickerResponse {
        market: market.clone(),
        last_price: rust_decimal::Decimal::from(50000),
        volume_24h: rust_decimal::Decimal::from(100),
        price_change_24h: rust_decimal::Decimal::from(1000),
        high_24h: rust_decimal::Decimal::from(51000),
        low_24h: rust_decimal::Decimal::from(49000),
        timestamp: Utc::now(),
    };
    
    Ok(HttpResponse::Ok().json(ticker))
}


async fn health_check() -> Result<HttpResponse>{
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "timestamp": Utc::now(),
        "service": "trading-engine-api"
    })))
}
