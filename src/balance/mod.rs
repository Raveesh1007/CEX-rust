use std::collections::HashMap;
use rust_decimal::Decimal;
use uuid::Uuid;


#[derive(Debug, Clone)]
pub struct UserBalance {
    pub available: Decimal,  
    pub locked: Decimal,     
}


impl UserBalance{
    pub fn new(available: Decimal, locked: Decimal) -> Self{
            UserBalance{available, 
            locked: Decimal::ZERO,
        }
    }
    
    pub fn total(&self) -> Decimal{
        self.available + self.locked
    }
}


#[derive(Debug)]
pub struct UserBalances{
    balances: HashMap<String, HashMap<String, UserBalance>>,

    locked_funds: HashMap<Uuid, (String, String, Decimal)>,
}


impl BalanceManager{
    pub fn new() -> Self{
        BalanceManager{
            balances: HashMap::new(),
            locked_funds: HashMap::new(),
        }
    }

    pub fn
}
