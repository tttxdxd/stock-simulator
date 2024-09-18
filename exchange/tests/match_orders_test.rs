use exchange::types::{OrderId, Price, Quantity, UserId};
use exchange::utils;
use std::collections::VecDeque;

#[test]
fn test_match_order() {
    let buy_order_id = 1 as OrderId;
    let buyer_id = 1 as UserId;
    let buy_price = 100 as Price;
    let buy_quantity = 100 as Quantity;

    let mut sell_orders = VecDeque::new();
    sell_orders.push_back((
        100 as Price,
        VecDeque::from(vec![(2 as OrderId, 2 as UserId, 100 as Quantity)]),
    ));

    let (remain_buy_quantity, trade_logs) = utils::match_order(
        buy_order_id,
        buyer_id,
        buy_price,
        buy_quantity,
        &mut sell_orders,
    );

    assert_eq!(remain_buy_quantity, 0);
    assert_eq!(trade_logs.len(), 1);

    if let Some(trade) = trade_logs.first() {
        assert_eq!(trade.buyer_id, 1);
        assert_eq!(trade.seller_id, 2);
        assert_eq!(trade.price, 100);
        assert_eq!(trade.quantity, 100);
        assert_eq!(trade.buy_order_id, 1);
        assert_eq!(trade.sell_order_id, 2);
    } else {
        panic!("No trades were matched");
    }

    // 验证卖单队列是否为空
    assert!(sell_orders.is_empty());
}

#[test]
fn test_match_order_no_match() {
    let buy_order_id = 1 as OrderId;
    let buyer_id = 1 as UserId;
    let buy_price = 100 as Price;
    let buy_quantity = 100 as Quantity;

    let mut sell_orders = VecDeque::new();
    sell_orders.push_back((
        100 as Price,
        VecDeque::from(vec![(2 as OrderId, 2 as UserId, 100 as Quantity)]),
    ));

    let (remain_buy_quantity, trade_logs) = utils::match_order(
        buy_order_id,
        buyer_id,
        buy_price,
        buy_quantity,
        &mut sell_orders,
    );

    assert_eq!(remain_buy_quantity, 0);
    assert_eq!(trade_logs.len(), 1);

    // 验证卖单队列是否为空
    assert_eq!(sell_orders.len(), 0);
}

#[test]
fn test_match_order_partial_fill() {
    let mut sell_orders = VecDeque::new();
    sell_orders.push_back((
        100 as Price,
        VecDeque::from(vec![(2 as OrderId, 2 as UserId, 100 as Quantity)]),
    ));

    let (remain_buy_quantity, trade_logs) = utils::match_order(
        1 as OrderId,
        1 as UserId,
        100 as Price,
        150 as Quantity,
        &mut sell_orders,
    );

    assert_eq!(remain_buy_quantity, 50);
    assert_eq!(trade_logs.len(), 1);

    if let Some(trade) = trade_logs.first() {
        assert_eq!(trade.buyer_id, 1);
        assert_eq!(trade.seller_id, 2);
        assert_eq!(trade.price, 100);
        assert_eq!(trade.quantity, 100);
        assert_eq!(trade.buy_order_id, 1);
        assert_eq!(trade.sell_order_id, 2);
    } else {
        panic!("No trades were matched");
    }

    // 验证卖单队列是否为空
    assert_eq!(sell_orders.len(), 0);
}

#[test]
fn test_match_orders() {
    // 创建买单和卖单
    let mut buy_orders = VecDeque::new();
    let mut sell_orders = VecDeque::new();

    // 添加买单 (价格从高到低)
    buy_orders.push_back((105, VecDeque::from(vec![(1, 1, 100), (2, 2, 50)])));
    buy_orders.push_back((100, VecDeque::from(vec![(3, 3, 200)])));

    // 添加卖单 (价格从低到高)
    sell_orders.push_back((95, VecDeque::from(vec![(4, 4, 150)])));
    sell_orders.push_back((100, VecDeque::from(vec![(5, 5, 100), (6, 6, 50)])));

    // 执行撮合
    let trade_logs = utils::match_orders(&mut buy_orders, &mut sell_orders);
    println!("buy_orders: {:?}", buy_orders);
    println!("sell_orders: {:?}", sell_orders);
    println!("trade_logs: {:?}", trade_logs);
    // 验证结果
    assert_eq!(buy_orders.len(), 1);
    assert_eq!(sell_orders.len(), 0);

    let (remaining_buy_price, remaining_buy_orders) = buy_orders.pop_front().unwrap();
    assert_eq!(remaining_buy_price, 100);
    assert_eq!(remaining_buy_orders.len(), 1);
    assert_eq!(remaining_buy_orders[0], (3, 3, 50));
}

#[test]
fn test_match_orders_partial_fill() {
    let mut buy_orders = VecDeque::new();
    let mut sell_orders = VecDeque::new();

    // 添加买单
    buy_orders.push_back((100, VecDeque::from(vec![(1, 1, 200)])));

    // 添加卖单
    sell_orders.push_back((100, VecDeque::from(vec![(2, 2, 150)])));

    // 执行撮合
    utils::match_orders(&mut buy_orders, &mut sell_orders);

    // 验证结果
    assert_eq!(buy_orders.len(), 1);
    assert_eq!(sell_orders.len(), 0);

    let (remaining_buy_price, remaining_buy_orders) = buy_orders.pop_front().unwrap();
    assert_eq!(remaining_buy_price, 100);
    assert_eq!(remaining_buy_orders.len(), 1);
    assert_eq!(remaining_buy_orders[0], (1, 1, 50));
}

#[test]
fn test_match_orders_no_match() {
    let mut buy_orders = VecDeque::new();
    let mut sell_orders = VecDeque::new();

    // 添加买单
    buy_orders.push_back((90, VecDeque::from(vec![(1, 1, 100)])));

    // 添加卖单
    sell_orders.push_back((100, VecDeque::from(vec![(2, 2, 100)])));

    // 执行撮合
    utils::match_orders(&mut buy_orders, &mut sell_orders);

    // 验证结果
    assert_eq!(buy_orders.len(), 1);
    assert_eq!(sell_orders.len(), 1);

    let (buy_price, buy_order) = buy_orders.pop_front().unwrap();
    assert_eq!(buy_price, 90);
    assert_eq!(buy_order[0], (1, 1, 100));

    let (sell_price, sell_order) = sell_orders.pop_front().unwrap();
    assert_eq!(sell_price, 100);
    assert_eq!(sell_order[0], (2, 2, 100));
}
