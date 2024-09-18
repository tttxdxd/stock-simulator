use exchange::config::ExchangeConfig;
use exchange::exchange::Exchange;
use exchange::order::OrderType;

#[test]
fn test_add_stock() {
    let config = ExchangeConfig::new();
    let mut exchange = Exchange::new(config);
    let _ = exchange.add_stock("000001", "平安银行", 15000);

    let stock = exchange.get_stock_info("000001").unwrap();
    assert_eq!(stock.code, "000001");
    assert_eq!(stock.name, "平安银行");
    assert_eq!(stock.current_price, 15000);
}

#[test]
fn test_submit_order() {
    let config = ExchangeConfig::new();
    let mut exchange = Exchange::new(config);
    exchange.next_timestamp("09:30:00");

    let _ = exchange.add_stock("000002", "平安银行", 15000);
    let user_id = exchange.add_user("user1", 100000000).unwrap();

    let result = exchange.submit_order(user_id, "000002".to_string(), OrderType::Buy, 14900, 100);

    println!("{:?}", result);
    assert!(result.is_ok());
}

#[test]
fn test_cancel_order() {
    let config = ExchangeConfig::new();
    let mut exchange = Exchange::new(config);
    exchange.next_timestamp("09:30:00");

    let _ = exchange.add_stock("000002", "平安银行", 15000);
    let user_id = exchange.add_user("user1", 100000000).unwrap();
    let order_id = exchange
        .submit_order(user_id, "000002".to_string(), OrderType::Buy, 14900, 100)
        .unwrap();
    let result = exchange.cancel_order(order_id);
    println!("{:?}", result);
    assert!(result.is_ok());
}

#[test]
fn test_order_matching() {
    let config = ExchangeConfig::new();
    let mut exchange = Exchange::new(config);
    exchange.next_timestamp("09:30:00");

    let _ = exchange.add_stock("000002", "平安银行", 15000);
    let user_id = exchange.add_user("user1", 100000000).unwrap();
    exchange
        .submit_order(user_id, "000002".to_string(), OrderType::Buy, 14900, 100)
        .unwrap();
    exchange
        .submit_order(user_id, "000002".to_string(), OrderType::Sell, 15100, 100)
        .unwrap();
    exchange.next_timestamp("09:30:01");

    let (trade_log, _) = exchange.get_trade_logs("000002", 1, 10);

    assert_eq!(trade_log.len(), 0);
}
