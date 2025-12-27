use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DbTrade {
    pub id: Uuid,
    pub trading_pair_id: Uuid,
    pub buyer_order_id: Uuid,
    pub seller_order_id: Uuid,
    pub buyer_user_id: Uuid,
    pub seller_user_id: Uuid,
    pub price: Decimal,
    pub quantity: Decimal,
    pub volume: Decimal,
    pub executed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Balance {
    pub id: Uuid,
    pub user_id: Uuid,
    pub asset: String,
    pub available: Decimal,
    pub locked: Decimal,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DbOrder {
    pub id: Uuid,
    pub user_id: Uuid,
    pub trading_pair_id: Uuid,
    pub order_type: String,
    pub side: String,
    pub quantity: Decimal,
    pub price: Option<Decimal>,
    pub filled_quantity: Decimal,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

