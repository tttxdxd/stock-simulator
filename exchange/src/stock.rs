use crate::exchange_error::{ExchangeError, ExchangeResult};
use crate::order::{Order, OrderType};
use crate::types::*;
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};

// 股票结构体
#[derive(Clone, Debug)]
pub struct Stock {
    // 基本信息
    pub code: StockCode,
    pub name: String,

    // 价格信息
    pub start_price: Price,
    pub current_price: Price,
    pub price_limit: PriceLimit,

    // 日交易信息
    pub daily_info: DailyTradeInfo,

    // 价格历史
    pub price_history: Vec<PriceHistoryInfo>,

    // 订单管理结构
    pub buy_orders: BTreeMap<u32, Vec<OrderId>>,
    pub buy_quantity: BTreeMap<u32, u64>,
    pub sell_orders: BTreeMap<u32, Vec<OrderId>>,
    pub sell_quantity: BTreeMap<u32, u64>,

    // 买卖队列
    pub order_queue: (Vec<(Price, u64)>, Vec<(Price, u64)>),
}

#[derive(Clone, Debug, Serialize)]
pub struct PriceHistoryInfo {
    /** 时间戳 */
    pub timestamp: Timestamp,
    /** 价格 */
    pub price: Price,
    /** 成交量 */
    pub volume: Quantity,
    /** 成交额 */
    pub amount: u64,
    /** 均价 */
    pub average_price: u64,
    /** 涨跌额 */
    pub price_change: i64,
    /** 涨跌幅 */
    pub price_change_rate: f64,
    /** 最低价 */
    pub min_price: Price,
    /** 最高价 */
    pub max_price: Price,
}

#[derive(Clone, Debug, Serialize)]
pub struct StockInfo {
    pub code: StockCode,
    pub name: String,
    pub start_price: Price,
    pub current_price: Price,
    pub price_limit: PriceLimit,
    pub daily_info: DailyTradeInfo,
}

// 修改：日交易数据结构体
#[derive(Clone, Debug, Serialize)]
pub struct DailyTradeInfo {
    pub opening_price: Option<Price>,
    pub closing_price: Option<Price>,
    pub highest_price: Price,
    pub lowest_price: Price,
    pub price_amplitude: f64,
    pub total_volume: u64,
    pub total_value: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct PriceLimit {
    pub upper: Price,
    pub lower: Price,
}

impl Stock {
    // 可以添加一些方法来管理订单和更新价格等
    pub fn new(code: String, name: String, start_price: Price) -> Self {
        Stock {
            code,
            name,
            start_price,
            current_price: start_price,
            price_limit: PriceLimit { upper: 0, lower: 0 },
            daily_info: DailyTradeInfo {
                opening_price: None,
                closing_price: None,
                highest_price: 0,
                lowest_price: 0,
                price_amplitude: 0.0,
                total_volume: 0,
                total_value: 0,
            },
            price_history: Vec::new(),
            buy_orders: BTreeMap::new(),
            buy_quantity: BTreeMap::new(),
            sell_orders: BTreeMap::new(),
            sell_quantity: BTreeMap::new(),
            order_queue: (Vec::new(), Vec::new()),
        }
    }

    pub fn update_price_limit(&mut self, price_limit_start: Price, price_limit_end: Price) {
        if self.start_price > 0 {
            self.price_limit = PriceLimit {
                upper: price_limit_end,
                lower: price_limit_start,
            };
        }
    }

    // 更新日交易信息
    pub fn update_daily_info(&mut self) {
        let daily_info = &mut self.daily_info;

        if daily_info.opening_price.is_none() {
            daily_info.opening_price = Some(self.current_price);
        }

        daily_info.highest_price = daily_info.highest_price.max(self.current_price);
        daily_info.lowest_price = if daily_info.lowest_price == 0 {
            self.current_price
        } else {
            daily_info.lowest_price.min(self.current_price)
        };

        if daily_info.highest_price > 0 && daily_info.lowest_price > 0 {
            daily_info.price_amplitude = (daily_info.highest_price as f64
                - daily_info.lowest_price as f64)
                / daily_info.lowest_price as f64
                * 100.0;
        }
    }

    // 更新买卖队列
    pub fn update_order_queue(&mut self) {
        let buy_queue = self
            .buy_orders
            .iter()
            .rev()
            .map(|(price, _)| (*price, self.buy_quantity.get(price).unwrap_or(&0).clone()))
            .collect();
        let sell_queue = self
            .sell_orders
            .iter()
            .map(|(price, _)| (*price, self.sell_quantity.get(price).unwrap_or(&0).clone()))
            .collect();

        self.order_queue = (buy_queue, sell_queue);
    }

    // 设置开盘价
    pub fn set_opening_price(&mut self, price: Price) {
        self.daily_info.opening_price = Some(price);
        self.current_price = price;
    }

    // 设置收��价
    pub fn set_closing_price(&mut self) {
        self.daily_info.closing_price = Some(self.current_price);
    }

    // 设置当前价格
    pub fn set_current_price(&mut self, price: Price) {
        self.current_price = price;
    }

    // 重置日交易信息
    pub fn reset_daily_info(&mut self) {
        self.daily_info = DailyTradeInfo {
            opening_price: None,
            closing_price: None,
            highest_price: 0,
            lowest_price: 0,
            price_amplitude: 0.0,
            total_volume: 0,
            total_value: 0,
        };
    }

    // 修改：添加价格到价格历史，一分钟内的数据整合到一条数据内
    pub fn add_price_to_history(&mut self, timestamp: Timestamp, price: Price, volume: Quantity) {
        let minute_timestamp = timestamp - (timestamp % 60); // 将时间戳向下取整到分钟

        if let Some(last_item) = self.price_history.last_mut() {
            if last_item.timestamp == minute_timestamp {
                // 更新同一分钟内的数据
                last_item.volume += volume;
                last_item.amount += price as u64 * volume as u64;
                last_item.average_price = if last_item.volume > 0 {
                    last_item.amount / last_item.volume as u64
                } else {
                    last_item.average_price // 保持原有均价
                };

                // 更新最高价和最低价
                last_item.max_price = last_item.max_price.max(price);
                last_item.min_price = last_item.min_price.min(price);

                // 更新当前价格（使用最新价格）
                last_item.price = price;

                // 更新涨跌额和涨跌幅
                last_item.price_change = price as i64 - self.start_price as i64;
                last_item.price_change_rate = if self.start_price > 0 {
                    (last_item.price_change as f64 / self.start_price as f64) * 100.0
                } else {
                    0.0
                };

                return;
            }
        }

        // 如果是新的一分钟或者是第一条记录，则添加新的记录
        let amount = price as u64 * volume as u64;
        let price_change = price as i64 - self.start_price as i64;
        let price_change_rate = if self.start_price > 0 {
            (price_change as f64 / self.start_price as f64) * 100.0
        } else {
            0.0
        };

        self.price_history.push(PriceHistoryInfo {
            timestamp: minute_timestamp,
            price,
            volume,
            amount,
            average_price: if volume > 0 { price as u64 } else { 0 },
            price_change,
            price_change_rate,
            min_price: price,
            max_price: price,
        });
    }

    // 添加订单到买单或卖单队列
    pub fn add_order(&mut self, order: &Order) {
        let orders = match order.order_type {
            OrderType::Buy => &mut self.buy_orders,
            OrderType::Sell => &mut self.sell_orders,
        };
        let quantities = match order.order_type {
            OrderType::Buy => &mut self.buy_quantity,
            OrderType::Sell => &mut self.sell_quantity,
        };
        orders
            .entry(order.price)
            .or_insert_with(Vec::new)
            .push(order.id);
        *quantities.entry(order.price).or_insert(0) += order.quantity as u64;
    }

    // 获取最优价格
    pub fn best_price(&self, order_type: OrderType) -> Option<Price> {
        match order_type {
            OrderType::Buy => self.buy_orders.keys().next().cloned(),
            OrderType::Sell => self.sell_orders.keys().next().cloned(),
        }
    }

    // 移除订单
    pub fn remove_order(&mut self, order: &Order) {
        let orders = match order.order_type {
            OrderType::Buy => &mut self.buy_orders,
            OrderType::Sell => &mut self.sell_orders,
        };
        let quantities = match order.order_type {
            OrderType::Buy => &mut self.buy_quantity,
            OrderType::Sell => &mut self.sell_quantity,
        };
        if let Some(order_ids) = orders.get_mut(&order.price) {
            order_ids.retain(|&id| id != order.id);
            if order_ids.is_empty() {
                orders.remove(&order.price);
            }
        }
        *quantities.entry(order.price).or_insert(0) -= order.quantity as u64;
    }
}

pub struct StockManager {
    stocks: HashMap<StockCode, Stock>,
}

impl StockManager {
    pub fn new() -> Self {
        StockManager {
            stocks: HashMap::new(),
        }
    }

    pub fn add_stock(
        &mut self,
        code: StockCode,
        name: String,
        start_price: Price,
        price_limit_start: Price,
        price_limit_end: Price,
    ) -> ExchangeResult<()> {
        if self.stocks.contains_key(&code) {
            return Err(ExchangeError::StockAlreadyExists(code));
        }
        let mut stock = Stock::new(code.clone(), name, start_price);
        stock.update_daily_info();
        stock.update_price_limit(price_limit_start, price_limit_end);
        self.stocks.insert(code, stock);
        Ok(())
    }

    pub fn get_stock_codes(&self) -> Vec<StockCode> {
        self.stocks.keys().cloned().collect()
    }

    pub fn get_stock(&self, code: &StockCode) -> Option<&Stock> {
        self.stocks.get(code)
    }

    pub fn get_stock_mut(&mut self, code: &StockCode) -> Option<&mut Stock> {
        self.stocks.get_mut(code)
    }

    pub fn reset_daily_info(&mut self) {
        for stock in self.stocks.values_mut() {
            stock.reset_daily_info();
        }
    }

    pub fn add_order(&mut self, order: &Order) -> ExchangeResult<()> {
        let stock = self
            .stocks
            .get_mut(&order.stock_code)
            .ok_or_else(|| ExchangeError::StockNotFound(order.stock_code.clone()))?;
        stock.add_order(order);
        Ok(())
    }

    pub fn remove_order(&mut self, order: &Order) -> ExchangeResult<()> {
        let stock = self
            .stocks
            .get_mut(&order.stock_code)
            .ok_or_else(|| ExchangeError::StockNotFound(order.stock_code.clone()))?;
        stock.remove_order(order);
        Ok(())
    }

    pub fn remove_buy_order(&mut self, stock: &mut Stock, order: &Order) -> ExchangeResult<()> {
        stock.remove_order(order);
        Ok(())
    }

    pub fn remove_sell_order(&mut self, stock: &mut Stock, order: &Order) -> ExchangeResult<()> {
        stock.remove_order(order);
        Ok(())
    }

    pub fn get_best_price(
        &self,
        stock_code: &StockCode,
        order_type: OrderType,
    ) -> ExchangeResult<Option<u32>> {
        let stock = self
            .stocks
            .get(stock_code)
            .ok_or_else(|| ExchangeError::StockNotFound(stock_code.to_string()))?;
        Ok(stock.best_price(order_type))
    }
}
