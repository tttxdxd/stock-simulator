use super::{TradingAction, TradingStrategyDecide};
use crate::exchange::Exchange;
use crate::user::User;
use rand::Rng;

// 修改后的随机交易策略
#[allow(dead_code)]
pub struct TradeRandomStrategy(pub u8);

impl TradingStrategyDecide for TradeRandomStrategy {
    fn decide(&self, user: &User, exchange: &Exchange) -> TradingAction {
        let mut rng = rand::thread_rng();

        // 50% 概率进行交易，50% 概率保持不动
        if rng.gen_bool(0.5) {
            let stock_codes = exchange.stock_manager.get_stock_codes();
            if stock_codes.is_empty() {
                return TradingAction::Hold;
            }

            let stock_code = stock_codes[rng.gen_range(0..stock_codes.len())].clone();
            let stock = exchange.stock_manager.get_stock(&stock_code).unwrap();

            // 决定是买入还是卖出
            let is_buy = rng.gen_bool(0.5);

            // 选择价格
            let price = if is_buy {
                if stock.sell_orders.is_empty() {
                    stock.current_price
                } else if rng.gen_bool(0.8) {
                    // 80% 概率选择卖一价格
                    *stock.sell_orders.keys().next().unwrap()
                } else {
                    // 20% 概率随机选择卖盘中的价格
                    let index = rng.gen_range(0..stock.sell_orders.len());
                    *stock.sell_orders.keys().nth(index).unwrap()
                }
            } else {
                if stock.buy_orders.is_empty() {
                    stock.current_price
                } else if rng.gen_bool(0.8) {
                    // 80% 概率选择买一价格
                    *stock.buy_orders.keys().next_back().unwrap()
                } else {
                    // 20% 概率随机选择买盘中的价格
                    let index = rng.gen_range(0..stock.buy_orders.len());
                    *stock.buy_orders.keys().nth(index).unwrap()
                }
            };

            // 随机决定交易数量
            let quantity = rng.gen_range(1..=10) * 100;

            if is_buy {
                // 买入
                let max_quantity = (user.balance / price as u64) as u32;
                if max_quantity >= 100 {
                    let quantity = quantity.min(max_quantity);
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
                        let quantity = quantity.min(holding.available_quantity as u32);
                        TradingAction::Sell {
                            stock_code: stock.code.clone(),
                            price,
                            quantity,
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
