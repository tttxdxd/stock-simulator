use crate::exchange::Exchange;
use crate::trading_strategy::{
    RandomStrategy, TradingAction, TradingStrategy, TradingStrategyType,
};
use crate::types::*;
use crate::user::User;
use std::collections::HashMap;

pub struct TradingBot {
    #[allow(dead_code)]
    user_id: UserId,
    strategy: TradingStrategyType,
}

impl TradingBot {
    pub fn new(user_id: UserId, strategy: TradingStrategyType) -> Self {
        TradingBot { user_id, strategy }
    }

    pub fn execute_strategy(&self, user: &User, exchange: &Exchange) -> TradingAction {
        match self.strategy {
            TradingStrategyType::Random => {
                let strategy = RandomStrategy;
                strategy.decide(user, exchange)
            }
        }
    }
}

pub struct TradingBotManager {
    bots: HashMap<UserId, TradingBot>,
}

impl TradingBotManager {
    pub fn new() -> Self {
        TradingBotManager {
            bots: HashMap::new(),
        }
    }

    pub fn add_bot(&mut self, user_id: UserId, strategy: TradingStrategyType) {
        let bot = TradingBot::new(user_id, strategy);
        self.bots.insert(user_id, bot);
    }

    pub fn remove_bot(&mut self, user_id: UserId) -> Option<TradingBot> {
        self.bots.remove(&user_id)
    }

    pub fn execute_strategy(&self, exchange: &Exchange) -> Vec<(UserId, TradingAction)> {
        let mut actions = Vec::new();
        for (user_id, bot) in self.bots.iter() {
            let user = exchange.user_manager.get_user(*user_id).unwrap();
            let action = bot.execute_strategy(&user, exchange);
            actions.push((*user_id, action));
        }
        actions
    }
}
