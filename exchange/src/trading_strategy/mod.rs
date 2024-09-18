use crate::exchange::Exchange;
use crate::types::*;
use crate::user::User;

mod simple_random;
mod trade_random;

use self::simple_random::RandomStrategy;
use self::trade_random::TradeRandomStrategy;

#[derive(Debug, Clone)]
pub enum TradingStrategy {
    /** 简单随机策略 */
    SimpleRandom,
    /** 交易随机策略 根据买卖盘口随机交易 */
    TradeRandom(u8),
}

pub fn get_trading_strategy(strategy: TradingStrategy) -> Box<dyn TradingStrategyDecide> {
    match strategy {
        TradingStrategy::SimpleRandom => Box::new(RandomStrategy),
        TradingStrategy::TradeRandom(n) => Box::new(TradeRandomStrategy(n)),
    }
}

pub trait TradingStrategyDecide {
    fn decide(&self, user: &User, exchange: &Exchange) -> TradingAction;
}

pub enum TradingAction {
    Buy {
        stock_code: StockCode,
        price: Price,
        quantity: Quantity,
    },
    Sell {
        stock_code: StockCode,
        price: Price,
        quantity: Quantity,
    },
    Hold,
}
