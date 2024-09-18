use exchange::utils::{self, PriceSelectionStrategy};
use std::collections::BTreeMap;

#[test]
fn test_calculate_max_volume_price() {
    // 创建测试用的价格-成交量映射
    let mut price_volume = BTreeMap::new();
    price_volume.insert(100, (50, 0)); // 价格100，买量50，卖量0
    price_volume.insert(101, (40, 10)); // 价格101，买量40，卖量10
    price_volume.insert(102, (30, 20)); // 价格102，买量30，卖量20
    price_volume.insert(103, (20, 30)); // 价格103，买量20，卖量30
    price_volume.insert(104, (10, 40)); // 价格104，买量10，卖量40

    // 调用函数计算最大成交量价格
    let (max_volume_price, max_volume) =
        utils::calculate_max_volume_price(&price_volume, PriceSelectionStrategy::Middle);

    // 验证结果
    assert_eq!(max_volume_price, 103);
    assert_eq!(max_volume, 30);
}

#[test]
fn test_calculate_max_volume_price_empty() {
    let price_volume = BTreeMap::new();
    let (max_volume_price, max_volume) =
        utils::calculate_max_volume_price(&price_volume, PriceSelectionStrategy::Middle);
    assert_eq!(max_volume_price, 0);
    assert_eq!(max_volume, 0);
}

#[test]
fn test_calculate_max_volume_price_single_entry() {
    let mut price_volume = BTreeMap::new();
    price_volume.insert(100, (50, 50));
    let (max_volume_price, max_volume) =
        utils::calculate_max_volume_price(&price_volume, PriceSelectionStrategy::Middle);
    assert_eq!(max_volume_price, 100);
    assert_eq!(max_volume, 50);
}

#[test]
fn test_calculate_max_volume_price_with_strategy() {
    let mut price_volume = BTreeMap::new();
    price_volume.insert(100, (50, 0));
    price_volume.insert(101, (40, 10));
    price_volume.insert(102, (30, 20));
    price_volume.insert(103, (20, 30));
    price_volume.insert(104, (10, 40));

    // 测试中间价格策略
    let (max_volume_price, max_volume) =
        utils::calculate_max_volume_price(&price_volume, PriceSelectionStrategy::Middle);
    assert_eq!(max_volume_price, 103);
    assert_eq!(max_volume, 30);

    // 测试最接近给定价格的策略
    let (max_volume_price, max_volume) =
        utils::calculate_max_volume_price(&price_volume, PriceSelectionStrategy::Nearest(101));
    assert_eq!(max_volume_price, 102);
    assert_eq!(max_volume, 30);

    let (max_volume_price, max_volume) =
        utils::calculate_max_volume_price(&price_volume, PriceSelectionStrategy::Nearest(104));
    assert_eq!(max_volume_price, 103);
    assert_eq!(max_volume, 30);
}

#[test]
fn test_calculate_max_volume_price_multiple_max_volumes() {
    let mut price_volume = BTreeMap::new();
    price_volume.insert(100, (30, 0));
    price_volume.insert(101, (30, 30));
    price_volume.insert(102, (30, 30));
    price_volume.insert(103, (30, 30));
    price_volume.insert(104, (30, 40));

    // 测试中间价格策略
    let (max_volume_price, max_volume) =
        utils::calculate_max_volume_price(&price_volume, PriceSelectionStrategy::Middle);
    assert_eq!(max_volume_price, 103);
    assert_eq!(max_volume, 60);

    // 测试最接近给定价格的策略
    let (max_volume_price, max_volume) =
        utils::calculate_max_volume_price(&price_volume, PriceSelectionStrategy::Nearest(100));
    assert_eq!(max_volume_price, 102);
    assert_eq!(max_volume, 60);

    let (max_volume_price, max_volume) =
        utils::calculate_max_volume_price(&price_volume, PriceSelectionStrategy::Nearest(104));
    assert_eq!(max_volume_price, 103);
    assert_eq!(max_volume, 60);
}
