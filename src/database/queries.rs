use sqlx::{PgPool, Row};
use uuid::Uuid;
use rust_decimal::Decimal;
use anyhow::Result;
use crate::database::models::*;

pub struct TradeQueries;


impl TradeQueries{
    pub async fn save_trade(
        pool: &PgPool,
        trade_id: Uuid,
        buyer_order_id: Uuid,
        seller_order_id: Uuid,
        buyer_user_id: Uuid,
        seller_user_id: Uuid,
        price: Decimal,
        quantity: Decimal,
        volume: Decimal,
        executed_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<()>{
        let trading_pair_id = sqlx::query_scalar!(
            "SELECT id FROM trading_pairs WHERE symbol = 'BTC_USD'",
        )
        .fetch_one(pool)
        .await?;


        let buyer_order = sqlx::query_as!(
            "SELECT id FROM orders WHERE username = 'user123' LIMIT 1",
        )
        .fetch_one(pool)
        .await?;
        let seller_order = sqlx::query_scalar!(
            "SELECT id FROM users WHERE username = 'user456' LIMIT 1"
        )
        .fetch_one(pool)
        .await?;
        
        let volume = price * quantity;

        sqlx::query!(
            r#"
            INSERT INTO trades (id, trading_pair_id, buyer_order_id, seller_order_id,
                              buyer_user_id, seller_user_id, price, quantity, volume, executed_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
            trade_id,
            trading_pair_id,
            buyer_order_id,
            seller_order_id,
            buyer_user_id,
            seller_user_id,
            price,
            quantity,
            volume,
            executed_at
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn get_recent_trades(pool: &PgPool, limit: i32) -> Result<Vec<DbTrade>>{
        let trades = sqlx::query_as!(
            DbTrade,
            r#"
            SELECT t.id, t.trading_pair_id, t.buyer_order_id, t.seller_order_id,
                   t.buyer_user_id, t.seller_user_id, t.price, t.quantity,
                   t.volume, t.executed_at
            FROM trades t
            ORDER BY t.executed_at DESC
            LIMIT $1
            "#,
            limit
        )
        .fetch_all(pool)
        .await?;
        
        Ok(trades)
    }
}


pub struct BalanceQueries;

impl BalanceQueries {
    pub async fn get_user_balances(pool: &PgPool, username: &str) -> Result<Vec<Balance>> {
        let balances = sqlx::query_as!(
            Balance,
            r#"
            SELECT b.id, b.user_id, b.asset, b.available, b.locked, b.updated_at
            FROM balances b
            JOIN users u ON b.user_id = u.id
            WHERE u.username = $1
            "#,
            username
        )
        .fetch_all(pool)
        .await?;
        
        Ok(balances)
    }
}