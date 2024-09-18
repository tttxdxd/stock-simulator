use crate::log::{PriceDirection, TradeLog};
use crate::types::{OrderId, Price, Quantity, UserId};
use std::collections::BTreeMap;
use std::collections::VecDeque;

/** 价格选择策略 */
pub enum PriceSelectionStrategy {
    /** 中间价 偶数个价格取中间两个中靠右的那个 */
    Middle,
    /** 最近价 */
    Nearest(Price),
}

/** 计算最大成交量价格 */
pub fn calculate_max_volume_price(
    price_volume: &BTreeMap<Price, (Quantity, Quantity)>,
    strategy: PriceSelectionStrategy,
) -> (Price, Quantity) {
    if price_volume.is_empty() {
        return (0, 0);
    }

    let mut accumulated_buy = 0;
    let mut current_sell = 0;
    let mut best_prices = Vec::new();
    let mut best_volume = 0;

    for (_, &(buy_volume, _)) in price_volume.iter() {
        accumulated_buy += buy_volume;
    }

    for (&price, &(buy_volume, sell_volume)) in price_volume.iter() {
        current_sell += sell_volume;
        let volume = accumulated_buy.min(current_sell);
        if volume > best_volume {
            best_prices.clear();
            best_prices.push(price);
            best_volume = volume;
        } else if volume == best_volume {
            best_prices.push(price);
        }
        accumulated_buy -= buy_volume;
    }

    let selected_price = match strategy {
        PriceSelectionStrategy::Middle => {
            let mid_index = best_prices.len() / 2;
            println!("mid_index: {}", mid_index);
            println!("best_prices: {:?}", best_prices);

            best_prices[mid_index]
        }
        PriceSelectionStrategy::Nearest(reference_price) => *best_prices
            .iter()
            .min_by_key(|&&p| (p as i64 - reference_price as i64).abs())
            .unwrap_or(&0),
    };

    (selected_price, best_volume)
}

/**
 * 匹配买卖订单
 *
 * 具体规则
 * 1. 同一个用户委托的订单不能与自己委托的订单撮合
 * 2. 买入委托价格 >= 卖出委托价格
 *
 */
pub fn match_orders(
    // 买入委托单：价格从高到低排序
    buy_orders: &mut VecDeque<(Price, VecDeque<(OrderId, UserId, Quantity)>)>,
    // 卖出委托单：价格从低到高排序
    sell_orders: &mut VecDeque<(Price, VecDeque<(OrderId, UserId, Quantity)>)>,
) -> Vec<TradeLog> {
    let mut trade_logs = Vec::new();
    let mut new_buy_orders = VecDeque::new();

    while !buy_orders.is_empty() && !sell_orders.is_empty() {
        let (buy_price, mut buy_list) = buy_orders.pop_front().unwrap();
        let mut del_index_list = Vec::new();
        for i in 0..buy_list.len() {
            let (buy_order_id, buy_user_id, mut buy_quantity) = buy_list[i];
            let (remain_buy_quantity, logs) = match_order(
                buy_order_id,
                buy_user_id,
                buy_price,
                buy_quantity,
                sell_orders,
            );
            buy_quantity = remain_buy_quantity;
            buy_list[i] = (buy_order_id, buy_user_id, buy_quantity);
            trade_logs.extend(logs);
            if buy_quantity == 0 {
                del_index_list.push(i);
            }
            if sell_orders.is_empty() {
                break;
            }
        }

        for index in del_index_list.iter().rev() {
            buy_list.remove(*index);
        }
        if !buy_list.is_empty() {
            new_buy_orders.push_back((buy_price, buy_list));
        }
        if sell_orders.is_empty() {
            break;
        }
    }
    buy_orders.append(&mut new_buy_orders);
    trade_logs
}

pub fn match_order(
    buy_order_id: OrderId,
    buyer_id: UserId,
    buy_price: Price,
    buy_quantity: Quantity,
    sell_orders: &mut VecDeque<(Price, VecDeque<(OrderId, UserId, Quantity)>)>,
) -> (Quantity, Vec<TradeLog>) {
    let mut buy_quantity = buy_quantity;
    let mut trade_logs = Vec::new();
    let mut del_order_index_list = Vec::new();
    for i in 0..sell_orders.len() {
        let (sell_price, sell_list) = &mut sell_orders[i];
        // 买入价格 < 卖出价格
        if buy_price < *sell_price {
            break;
        }
        let mut del_index_list = Vec::new();

        for index in 0..sell_list.len() {
            let (sell_order_id, seller_id, mut sell_quantity) = sell_list[index];
            if buyer_id == seller_id {
                continue;
            }
            let matched_quantity = sell_quantity.min(buy_quantity);
            let matched_price = *sell_price;
            let direction = if matched_price > buy_price {
                PriceDirection::Up
            } else if matched_price < buy_price {
                PriceDirection::Down
            } else {
                PriceDirection::Flat
            };

            trade_logs.push(TradeLog::new(
                buyer_id,
                seller_id,
                matched_price,
                matched_quantity,
                buy_order_id,
                sell_order_id,
                direction,
            ));

            buy_quantity -= matched_quantity;
            sell_quantity -= matched_quantity;
            sell_list[index] = (sell_order_id, seller_id, sell_quantity);

            if sell_quantity == 0 {
                // 卖出委托单完全成交
                del_index_list.push(index);
            }
            if buy_quantity == 0 {
                // 买入委托单完全成交
                break;
            }
        }

        for index in del_index_list.iter().rev() {
            sell_list.remove(*index);
        }
        if sell_list.is_empty() {
            del_order_index_list.push(i);
        }
        if buy_quantity == 0 {
            break;
        }
    }

    for index in del_order_index_list.iter().rev() {
        sell_orders.remove(*index);
    }

    (buy_quantity, trade_logs)
}
