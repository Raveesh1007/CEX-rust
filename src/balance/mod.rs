use std::collections::HashMap;
use rust_decimal::Decimal;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct UserBalance {
    pub available: Decimal,  // Free to use
    pub locked: Decimal,     // Reserved for open orders
}

impl UserBalance {
    pub fn new(available: Decimal) -> Self {
        UserBalance {
            available,
            locked: Decimal::ZERO,
        }
    }
    
    pub fn total(&self) -> Decimal {
        self.available + self.locked
    }
}

#[derive(Debug)]
pub struct BalanceManager {
    // user_id -> asset -> balance
    balances: HashMap<String, HashMap<String, UserBalance>>,
    // order_id -> (user_id, asset, locked_amount) for unlocking on cancel
    locked_funds: HashMap<Uuid, (String, String, Decimal)>,
}

impl BalanceManager {
    pub fn new() -> Self {
        BalanceManager {
            balances: HashMap::new(),
            locked_funds: HashMap::new(),
        }
    }
    
    // Initialize user with starting balances
    pub fn add_user(&mut self, user_id: String, initial_balances: HashMap<String, Decimal>) {
        let mut user_balances = HashMap::new();
        for (asset, amount) in initial_balances {
            user_balances.insert(asset, UserBalance::new(amount));
        }
        self.balances.insert(user_id, user_balances);
    }
    
    // Check if user has enough balance for an order
    pub fn can_place_order(&self, user_id: &str, asset: &str, required_amount: Decimal) -> bool {
        if let Some(user_balances) = self.balances.get(user_id) {
            if let Some(balance) = user_balances.get(asset) {
                return balance.available >= required_amount;
            }
        }
        false
    }
    
    // Lock funds for an order (reserve them)
    pub fn lock_funds(&mut self, order_id: Uuid, user_id: &str, asset: &str, amount: Decimal) -> Result<(), String> {
        if !self.can_place_order(user_id, asset, amount) {
            return Err(format!("Insufficient {} balance for user {}", asset, user_id));
        }
        
        if let Some(user_balances) = self.balances.get_mut(user_id) {
            if let Some(balance) = user_balances.get_mut(asset) {
                balance.available -= amount;
                balance.locked += amount;
                
                // Track locked funds for potential unlocking
                self.locked_funds.insert(order_id, (user_id.to_string(), asset.to_string(), amount));
                
                return Ok(());
            }
        }
        
        Err(format!("User {} or asset {} not found", user_id, asset))
    }
    
    // Unlock funds (when order is cancelled)
    pub fn unlock_funds(&mut self, order_id: Uuid) -> Result<(), String> {
        if let Some((user_id, asset, amount)) = self.locked_funds.remove(&order_id) {
            if let Some(user_balances) = self.balances.get_mut(&user_id) {
                if let Some(balance) = user_balances.get_mut(&asset) {
                    balance.locked -= amount;
                    balance.available += amount;
                    return Ok(());
                }
            }
        }
        Err(format!("Order {} not found in locked funds", order_id))
    }
    
    // Execute trade - transfer balances between users
    pub fn execute_trade(&mut self, buyer_id: &str, seller_id: &str, 
                        base_asset: &str, quote_asset: &str, 
                        quantity: Decimal, price: Decimal) -> Result<(), String> {
        let quote_amount = quantity * price;
        
        // Buyer: lose quote asset, gain base asset
        self.transfer_asset(buyer_id, quote_asset, seller_id, quote_amount)?;
        
        // Seller: lose base asset, gain quote asset  
        self.transfer_asset(seller_id, base_asset, buyer_id, quantity)?;
        
        Ok(())
    }
    
    // Helper: Transfer asset from one user to another
    fn transfer_asset(&mut self, from_user: &str, asset: &str, to_user: &str, amount: Decimal) -> Result<(), String> {
        // Remove from sender's locked funds
        if let Some(from_balances) = self.balances.get_mut(from_user) {
            if let Some(from_balance) = from_balances.get_mut(asset) {
                if from_balance.locked >= amount {
                    from_balance.locked -= amount;
                } else {
                    return Err(format!("Insufficient locked {} for user {}", asset, from_user));
                }
            } else {
                return Err(format!("Asset {} not found for user {}", asset, from_user));
            }
        } else {
            return Err(format!("User {} not found", from_user));
        }
        
        // Add to receiver's available funds
        if let Some(to_balances) = self.balances.get_mut(to_user) {
            if let Some(to_balance) = to_balances.get_mut(asset) {
                to_balance.available += amount;
            } else {
                // Create new asset balance if it doesn't exist
                to_balances.insert(asset.to_string(), UserBalance::new(amount));
            }
        } else {
            return Err(format!("User {} not found", to_user));
        }
        
        Ok(())
    }
    
    // Get user's balance for an asset
    pub fn get_balance(&self, user_id: &str, asset: &str) -> Option<&UserBalance> {
        self.balances.get(user_id)?.get(asset)
    }
    
    // Get all balances for a user
    pub fn get_user_balances(&self, user_id: &str) -> Option<&HashMap<String, UserBalance>> {
        self.balances.get(user_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_balance_management() {
        let mut bm = BalanceManager::new();
        
        // Add user with initial balances
        let mut initial = HashMap::new();
        initial.insert("BTC".to_string(), Decimal::from(10));
        initial.insert("USD".to_string(), Decimal::from(50000));
        bm.add_user("user1".to_string(), initial);
        
        // Test locking funds
        let order_id = Uuid::new_v4();
        assert!(bm.lock_funds(order_id, "user1", "USD", Decimal::from(25000)).is_ok());
        
        let balance = bm.get_balance("user1", "USD").unwrap();
        assert_eq!(balance.available, Decimal::from(25000));
        assert_eq!(balance.locked, Decimal::from(25000));
        
        // Test insufficient funds
        assert!(bm.lock_funds(Uuid::new_v4(), "user1", "USD", Decimal::from(30000)).is_err());
    }
}