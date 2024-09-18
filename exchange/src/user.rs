use crate::exchange_error::{ExchangeError, ExchangeResult};
use crate::types::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct User {
    pub id: UserId,
    pub username: String,
    pub balance: u64,
    pub holdings: HashMap<StockCode, Holding>,
}

#[derive(Debug, Clone)]
pub struct Holding {
    pub quantity: u64,
    pub available_quantity: u64,
}

impl User {
    pub fn new(id: UserId, username: String, initial_balance: u64) -> Self {
        User {
            id,
            username,
            balance: initial_balance,
            holdings: HashMap::new(),
        }
    }

    // 判断用户是否足够
    pub fn has_enough_balance(&self, price: Price, quantity: Quantity) -> bool {
        self.balance >= price as u64 * quantity as u64
    }

    pub fn deposit(&mut self, amount: u64) {
        self.balance += amount;
    }

    pub fn withdraw(&mut self, amount: u64) -> ExchangeResult<()> {
        if self.balance >= amount {
            self.balance -= amount;
            Ok(())
        } else {
            Err(ExchangeError::InsufficientBalance)
        }
    }

    pub fn add_holding(&mut self, stock_code: StockCode, quantity: u64) {
        let holding = self.holdings.entry(stock_code).or_insert(Holding {
            quantity: 0,
            available_quantity: 0,
        });
        holding.quantity += quantity;
        holding.available_quantity += quantity;
    }

    pub fn remove_holding(&mut self, stock_code: StockCode, quantity: u64) {
        let holding = self.holdings.get_mut(&stock_code).unwrap();
        holding.available_quantity -= quantity;
        if holding.available_quantity <= 0 {
            self.holdings.remove(&stock_code);
        }
    }
}

pub struct UserManager {
    users: HashMap<UserId, User>,
    next_user_id: UserId,
}

impl UserManager {
    pub fn new() -> Self {
        UserManager {
            users: HashMap::new(),
            next_user_id: 1,
        }
    }

    pub fn create_user(&mut self, username: String, initial_balance: u64) -> UserId {
        let user_id = self.next_user_id;
        self.next_user_id += 1;
        let user = User::new(user_id, username, initial_balance);
        self.users.insert(user_id, user);
        user_id
    }

    pub fn get_user(&self, user_id: UserId) -> Option<&User> {
        self.users.get(&user_id)
    }

    pub fn get_user_mut(&mut self, user_id: UserId) -> Option<&mut User> {
        self.users.get_mut(&user_id)
    }

    pub fn deposit(&mut self, user_id: UserId, amount: u64) -> ExchangeResult<()> {
        if let Some(user) = self.users.get_mut(&user_id) {
            user.deposit(amount);
            Ok(())
        } else {
            Err(ExchangeError::UserNotFound(user_id))
        }
    }

    pub fn withdraw(&mut self, user_id: UserId, amount: u64) -> ExchangeResult<()> {
        if let Some(user) = self.users.get_mut(&user_id) {
            user.withdraw(amount)
        } else {
            Err(ExchangeError::UserNotFound(user_id))
        }
    }

    // 清算
    pub fn reset_positions(&mut self) {
        for user in self.users.values_mut() {
            for holding in user.holdings.values_mut() {
                holding.available_quantity = holding.quantity;
            }
        }
    }
}
