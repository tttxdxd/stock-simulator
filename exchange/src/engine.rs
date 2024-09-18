use crate::exchange::Exchange;
use crate::log::TradeLog;
use crate::types::*;
use crate::utils;

use std::collections::BTreeMap;
use std::collections::VecDeque;

// MatchingEngine 结构体
pub struct MatchingEngine {}

impl MatchingEngine {
    pub fn new() -> Self {
        MatchingEngine {}
    }

    /** 试撮合交易 */
    pub fn simulate_match_trades(&self, exchange: &Exchange) -> Vec<(StockCode, Price, Quantity)> {
        let stock_codes = exchange.stock_manager.get_stock_codes();
        let mut trades = Vec::new();

        for stock_code in stock_codes {
            let stock = exchange.stock_manager.get_stock(&stock_code);
            if let Some(stock) = stock {
                let buy_orders = &stock.buy_orders;
                let sell_orders = &stock.sell_orders;

                if buy_orders.is_empty() || sell_orders.is_empty() {
                    continue;
                }

                let mut price_volume: BTreeMap<Price, (Quantity, Quantity)> = BTreeMap::new();

                // 统计每个价格的买卖委托量
                for (price, order_ids) in buy_orders {
                    for order_id in order_ids {
                        let order = exchange.order_manager.get_order(*order_id);
                        if let Some(order) = order {
                            if order.remaining_quantity > 0 {
                                price_volume.entry(*price).or_insert((0, 0)).0 +=
                                    order.remaining_quantity;
                            }
                        }
                    }
                }
                for (price, order_ids) in sell_orders {
                    for order_id in order_ids {
                        let order = exchange.order_manager.get_order(*order_id);
                        if let Some(order) = order {
                            if order.remaining_quantity > 0 {
                                price_volume.entry(*price).or_insert((0, 0)).1 +=
                                    order.remaining_quantity;
                            }
                        }
                    }
                }

                let (best_price, best_volume) = utils::calculate_max_volume_price(
                    &price_volume,
                    utils::PriceSelectionStrategy::Middle,
                );

                trades.push((stock_code, best_price, best_volume));
            }
        }

        trades
    }

    /** 正式撮合交易 */
    pub fn execute_match_trades(&self, _: &Exchange) -> () {}

    /** 实时连续竞价交易 */
    pub fn continuous_trading(&self, exchange: &Exchange) -> Vec<TradeLog> {
        let mut trade_logs = Vec::new();

        for stock_code in exchange.stock_manager.get_stock_codes() {
            let stock = exchange.stock_manager.get_stock(&stock_code).unwrap();
            let buy_orders: &BTreeMap<u32, Vec<u64>> = &stock.buy_orders;
            let sell_orders: &BTreeMap<u32, Vec<u64>> = &stock.sell_orders;

            // 买入委托单：价格从高到低排序
            let mut buy_orders = buy_orders
                .iter()
                .rev() // 反转迭代器，使价格从高到低
                .map(|(price, order_ids)| {
                    (
                        *price,
                        order_ids
                            .iter()
                            .map(|id| exchange.order_manager.get_order(*id).unwrap())
                            .map(|order| (order.id, order.user_id, order.quantity))
                            .filter(|(_, _, quantity)| *quantity > 0)
                            .collect::<VecDeque<(OrderId, UserId, Quantity)>>(),
                    )
                })
                .collect::<VecDeque<(Price, VecDeque<(OrderId, UserId, Quantity)>)>>();

            // 卖出委托单：价格从低到高排序
            let mut sell_orders = sell_orders
                .iter() // 已经是从低到高排序
                .map(|(price, order_ids)| {
                    (
                        *price,
                        order_ids
                            .iter()
                            .map(|id| exchange.order_manager.get_order(*id).unwrap())
                            .map(|order| (order.id, order.user_id, order.quantity))
                            .filter(|(_, _, quantity)| *quantity > 0)
                            .collect::<VecDeque<(OrderId, UserId, Quantity)>>(),
                    )
                })
                .collect::<VecDeque<(Price, VecDeque<(OrderId, UserId, Quantity)>)>>();

            trade_logs.extend(
                utils::match_orders(&mut buy_orders, &mut sell_orders)
                    .into_iter()
                    .map(|mut order| {
                        order.stock_code = stock_code.clone();
                        order
                    }),
            );

            // 删除已成交的委托单
        }
        trade_logs
    }
}
