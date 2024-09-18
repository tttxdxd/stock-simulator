use crate::exchange::Exchange;
use crate::trading_strategy::{TradingAction, TradingStrategyDecide};
use crate::user::User;
use rand::Rng;

// 修改后的随机交易策略
pub struct RandomStrategy;

impl TradingStrategyDecide for RandomStrategy {
    fn decide(&self, user: &User, exchange: &Exchange) -> TradingAction {
        let mut rng = rand::thread_rng();

        // 20% 概率进行交易，80% 概率保持不动
        if rng.gen_bool(0.2) {
            let stock_codes = exchange.stock_manager.get_stock_codes();
            if stock_codes.is_empty() {
                return TradingAction::Hold;
            }

            let stock_code = stock_codes[rng.gen_range(0..stock_codes.len())].clone();
            let stock = exchange.stock_manager.get_stock(&stock_code).unwrap();
            let limit = stock.price_limit.clone();
            let current_price = stock.current_price;

            // 在当前价格的基础上，以较小的幅度随机生成价格（±0.01%）
            let price_range = ((current_price as f64 * 0.0001) as u32).max(1);
            let min_price = current_price.saturating_sub(price_range).max(limit.lower);
            let max_price = (current_price + price_range).min(limit.upper);
            let price = rng.gen_range(min_price..=max_price);

            // 随机决定买入或卖出
            if rng.gen_bool(0.5) {
                // 买入
                let max_quantity = (user.balance / price as u64) as u32;
                if max_quantity >= 100 {
                    let quantity = (rng.gen_range(1..=5) * 100).min(max_quantity);
                    TradingAction::Buy {
                        stock_code: stock.code.clone(),
                        price,
                        quantity,
                    }
                } else {
                    TradingAction::Hold
                }
            } else {
                // 卖出
                if let Some(holding) = user.holdings.get(&stock.code) {
                    if holding.available_quantity >= 100 {
                        let quantity = (rng.gen_range(1..=5) * 100).min(holding.available_quantity);
                        TradingAction::Sell {
                            stock_code: stock.code.clone(),
                            price,
                            quantity: quantity as u32,
                        }
                    } else {
                        TradingAction::Hold
                    }
                } else {
                    TradingAction::Hold
                }
            }
        } else {
            TradingAction::Hold
        }
    }
}
