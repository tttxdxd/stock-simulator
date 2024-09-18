use std::collections::BTreeMap;

use crate::config::ExchangeConfig;
use crate::config::TradingPeriodType;
use crate::engine::MatchingEngine;
use crate::exchange_error::ExchangeError;
use crate::log::LogManager;
use crate::log::TradeLog;
use crate::order::{OrderManager, OrderType};
use crate::stock::PriceHistoryInfo;
use crate::stock::StockInfo;
use crate::stock::StockManager;
use crate::trade_day::TradingCalendar;
use crate::trading_bot::TradingBotManager;
use crate::trading_strategy::TradingAction;
use crate::trading_strategy::TradingStrategyType;
use crate::types::*;
use crate::user::UserManager;
use chrono::NaiveDate;

// 交易所结构体
pub struct Exchange {
    pub config: ExchangeConfig,
    pub engine: MatchingEngine,
    pub stock_manager: StockManager,
    pub user_manager: UserManager,
    pub bot_manager: TradingBotManager,
    pub order_manager: OrderManager,
    pub log_manager: LogManager,
    pub trade_day_manager: TradingCalendar,
    pub current_trade_day: NaiveDate,
}

impl Exchange {
    pub fn new(config: ExchangeConfig) -> Self {
        let exchange = Self {
            config: config,
            engine: MatchingEngine::new(),
            user_manager: UserManager::new(),
            bot_manager: TradingBotManager::new(),
            order_manager: OrderManager::new(),
            stock_manager: StockManager::new(),
            log_manager: LogManager::new(),
            trade_day_manager: TradingCalendar::new(),
            current_trade_day: NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
        };
        exchange
    }

    pub fn next_timestamp(&mut self, timestamp: &str) {
        let timestamp = string_to_timestamp(timestamp).unwrap();
        if timestamp <= self.config.current_timestamp {
            return;
        }
        self.config.set_current_timestamp(timestamp);
        // 当前配置阶段
        let trading_period = self.config.get_current_period();
        if let Some(period) = trading_period {
            match period.period_type {
                TradingPeriodType::OpeningAuction => {
                    // 开盘竞价
                    let records = self.engine.simulate_match_trades(self);

                    for (stock_code, price, quantity) in records {
                        let stock = self.stock_manager.get_stock_mut(&stock_code).unwrap();
                        stock.set_opening_price(price);
                        stock.add_price_to_history(timestamp, price, quantity);
                        stock.update_daily_info();
                    }
                }
                TradingPeriodType::CallAuctionWithCancel => {
                    // 集合竞价 可以撤单
                    let records = self.engine.simulate_match_trades(self);

                    for (stock_code, price, quantity) in records {
                        let stock = self.stock_manager.get_stock_mut(&stock_code).unwrap();
                        stock.set_opening_price(price);
                        stock.add_price_to_history(timestamp, price, quantity);
                        stock.update_daily_info();
                    }
                }
                TradingPeriodType::CallAuctionNoCancel => {
                    // 集合竞价 不可以撤单
                    self.engine.execute_match_trades(self);
                }
                TradingPeriodType::ContinuousTrading => {
                    // 连续交易
                    let trade_logs = self.engine.continuous_trading(self);
                    // stock_code -> price -> order_ids
                    let mut ready_del_buy_order: BTreeMap<String, BTreeMap<u32, (Vec<u64>, u64)>> =
                        BTreeMap::new();
                    let mut ready_del_sell_order: BTreeMap<String, BTreeMap<u32, (Vec<u64>, u64)>> =
                        BTreeMap::new();

                    for trade_log in trade_logs {
                        self.log_manager.add_log(trade_log.clone());
                        let sell_order_id = trade_log.sell_order_id;
                        let buy_order_id = trade_log.buy_order_id;
                        let stock_code = trade_log.stock_code;
                        let price: u32 = trade_log.price;
                        let quantity = trade_log.quantity;

                        let buy_order = self.order_manager.get_order_mut(buy_order_id).unwrap();
                        buy_order.execute(quantity, price, timestamp);

                        if buy_order.is_filled() {
                            let temp = ready_del_buy_order
                                .entry(stock_code.clone())
                                .or_insert_with(BTreeMap::new)
                                .entry(buy_order.price)
                                .or_insert_with(|| (Vec::new(), 0));
                            temp.0.push(buy_order_id);
                            temp.1 += quantity as u64;
                        }

                        let sell_order = self.order_manager.get_order_mut(sell_order_id).unwrap();
                        sell_order.execute(quantity, price, timestamp);

                        if sell_order.is_filled() {
                            let temp = ready_del_sell_order
                                .entry(stock_code.clone())
                                .or_insert_with(BTreeMap::new)
                                .entry(sell_order.price)
                                .or_insert_with(|| (Vec::new(), 0));
                            temp.0.push(sell_order_id);
                            temp.1 += quantity as u64;
                        }

                        let stock = self.stock_manager.get_stock_mut(&stock_code).unwrap();
                        stock.set_current_price(price);
                        stock.add_price_to_history(timestamp, price, quantity);
                        stock.update_daily_info();
                    }

                    // 整理股票委托，清空已成交的委托
                    for (stock_code, price_orders) in ready_del_buy_order.iter() {
                        let stock = self.stock_manager.get_stock_mut(stock_code).unwrap();
                        for (price, (order_ids, num)) in price_orders.iter() {
                            stock.buy_orders.entry(*price).and_modify(|orders| {
                                orders.retain(|order_id| !order_ids.contains(order_id));
                            });
                            stock.buy_quantity.entry(*price).and_modify(|quantity| {
                                *quantity -= *num;
                            });
                            if stock.buy_orders.get(price).unwrap().is_empty() {
                                stock.buy_orders.remove(&price);
                            }
                        }
                    }

                    for (stock_code, price_orders) in ready_del_sell_order.iter() {
                        let stock = self.stock_manager.get_stock_mut(stock_code).unwrap();
                        for (price, (order_ids, num)) in price_orders.iter() {
                            stock.sell_orders.entry(*price).and_modify(|orders| {
                                orders.retain(|order_id| !order_ids.contains(order_id));
                            });
                            stock.sell_quantity.entry(*price).and_modify(|quantity| {
                                *quantity -= *num;
                            });

                            if stock.sell_orders.get(price).unwrap().is_empty() {
                                stock.sell_orders.remove(&price);
                            }
                        }
                    }
                }
                TradingPeriodType::ClosingAuction => {
                    // 收盘竞价
                    self.engine.execute_match_trades(self);
                }
                TradingPeriodType::MarketClosed => {
                    // 市场关闭
                }
                TradingPeriodType::MiddayBreak => {
                    // 午间休市
                }
            }

            if period.allow_record_price_history {
                for stock_code in self.stock_manager.get_stock_codes() {
                    let stock = self.stock_manager.get_stock_mut(&stock_code).unwrap();
                    stock.add_price_to_history(timestamp, stock.current_price, 0);
                    stock.update_order_queue();
                }
            }
        }
    }

    pub fn next_trade_day(&mut self) {
        self.current_trade_day = self
            .trade_day_manager
            .next_trade_day(self.current_trade_day);

        // 清算
        self.order_manager.clear_orders();
        // 用户持仓变可用
        self.user_manager.reset_positions();
        // 设置当前时间戳
        self.config.set_current_timestamp(0);
    }

    pub fn get_config(&self) -> ExchangeConfig {
        self.config.clone()
    }

    pub fn get_current_timestamp(&self) -> Timestamp {
        self.config.current_timestamp
    }

    fn check_tick_allowed(&self, action: &str) -> Result<(), ExchangeError> {
        let is_allowed = match action {
            "order" => self.config.is_allow_order(),
            "cancel" => self.config.is_allow_cancel(),
            _ => false,
        };

        if !is_allowed {
            Err(ExchangeError::ActionNotAllowed {
                action: action.to_string(),
                time: timestamp_to_string(self.config.current_timestamp)
                    + " in "
                    + &self.config.get_current_period().unwrap().to_string(),
            })
        } else {
            Ok(())
        }
    }

    /** 下单 */
    pub fn submit_order(
        &mut self,
        user_id: UserId,
        stock_code: StockCode,
        order_type: OrderType,
        price: Price,
        quantity: Quantity,
    ) -> Result<u64, ExchangeError> {
        self.check_tick_allowed("order")?;

        let user = self
            .user_manager
            .get_user_mut(user_id)
            .ok_or(ExchangeError::UserNotFound(user_id))?;
        // 判断用户现金是否足够
        if order_type == OrderType::Buy && !user.has_enough_balance(price, quantity) {
            println!(
                "submit_order failed!!! user: {} has_enough_balance: {}",
                user.id,
                user.has_enough_balance(price, quantity)
            );
            return Err(ExchangeError::InsufficientBalance);
        }
        // 判断股票是否存在
        let stock = self
            .stock_manager
            .get_stock_mut(&stock_code)
            .ok_or(ExchangeError::StockNotFound(stock_code.to_string()))?;

        // 判断价格是否在限制范围内
        if price < stock.price_limit.lower || price > stock.price_limit.upper {
            println!(
                "submit_order failed!!! stock: {} price: {} limit: {:?}",
                stock_code, price, stock.price_limit
            );
            return Err(ExchangeError::PriceOutOfLimit(stock_code.to_string()));
        }

        // 创建订单
        let order = self
            .order_manager
            .create_order(user_id, stock_code, order_type, price, quantity);

        stock.add_order(&order);

        Ok(order.id)
    }

    /** 撤单 */
    pub fn cancel_order(&mut self, order_id: u64) -> Result<(), ExchangeError> {
        self.check_tick_allowed("cancel")?;

        // 判断订单是否存在
        let order = self
            .order_manager
            .get_order_mut(order_id)
            .ok_or(ExchangeError::OrderNotFound(order_id))?;
        // 判断订单是否可以撤单
        if !order.is_cancellable() {
            return Err(ExchangeError::OrderNotCancellable(order_id));
        }

        order.cancel();
        let stock = self
            .stock_manager
            .get_stock_mut(&order.stock_code)
            .ok_or(ExchangeError::StockNotFound(order.stock_code.to_string()))?;
        stock.remove_order(order);

        Ok(())
    }

    /** 添加股票 */
    pub fn add_stock(
        &mut self,
        stock_code: &str,
        stock_name: &str,
        start_price: Price,
    ) -> Result<(), ExchangeError> {
        let price_limit_start =
            (start_price as f64 * (1.0 - self.config.price_limit_percentage)).round() as Price;
        let price_limit_end =
            (start_price as f64 * (1.0 + self.config.price_limit_percentage)).round() as Price;
        self.stock_manager.add_stock(
            stock_code.to_string(),
            stock_name.to_string(),
            start_price,
            price_limit_start,
            price_limit_end,
        )?;
        Ok(())
    }

    /** 获取股票列表 */
    pub fn get_stock_list(&self) -> Vec<StockCode> {
        self.stock_manager.get_stock_codes()
    }

    /** 获取股票信息 */
    pub fn get_stock_info(&self, stock_code: &str) -> Option<StockInfo> {
        self.stock_manager
            .get_stock(&stock_code.to_string())
            .map(|stock| StockInfo {
                code: stock.code.clone(),
                name: stock.name.clone(),
                start_price: stock.start_price,
                current_price: stock.current_price,
                price_limit: stock.price_limit.clone(),
                daily_info: stock.daily_info.clone(),
            })
    }

    /** 获取股票曲线 */
    pub fn get_price_history(&self, stock_code: &str) -> Option<Vec<PriceHistoryInfo>> {
        let stock = self.stock_manager.get_stock(&stock_code.to_string())?;
        Some(stock.price_history.clone())
    }

    /** 获取交易记录 */
    pub fn get_trade_logs(
        &self,
        stock_code: &str,
        page: usize,
        page_size: usize,
    ) -> (Vec<TradeLog>, usize) {
        self.log_manager.page_logs(stock_code, page, page_size)
    }

    pub fn add_user(
        &mut self,
        user_name: &str,
        initial_balance: u64,
    ) -> Result<UserId, ExchangeError> {
        let user_id = self
            .user_manager
            .create_user(user_name.to_string(), initial_balance);
        Ok(user_id)
    }

    /** 买卖队列 */
    pub fn get_order_queue(
        &self,
        stock_code: StockCode,
        limit: usize,
    ) -> (Vec<(Price, u64)>, Vec<(Price, u64)>) {
        let stock = self.stock_manager.get_stock(&stock_code).unwrap();
        let (buy_orders, sell_orders) = &stock.order_queue;
        let buy_orders: Vec<(Price, u64)> = buy_orders.iter().take(limit).cloned().collect();
        let sell_orders: Vec<(Price, u64)> = sell_orders.iter().take(limit).cloned().collect();

        (buy_orders, sell_orders)
    }

    pub fn add_robot(
        &mut self,
        user_name: &str,
        initial_balance: u64,
        strategy: TradingStrategyType,
        initial_holdings: Vec<(&str, u64)>,
    ) -> Result<UserId, ExchangeError> {
        let user_id = self.add_user(user_name, initial_balance)?;
        let user = self.user_manager.get_user_mut(user_id).unwrap();
        for (stock_code, quantity) in initial_holdings {
            user.add_holding(stock_code.to_string(), quantity);
        }
        self.bot_manager.add_bot(user_id, strategy);
        Ok(user_id)
    }

    // You might want to add a method to execute all bot strategies
    pub fn execute_robot_strategies(&mut self) -> Result<(), ExchangeError> {
        let bot_actions = self.bot_manager.execute_strategy(self);

        for (user_id, action) in bot_actions {
            match action {
                TradingAction::Buy {
                    stock_code,
                    price,
                    quantity,
                } => {
                    let _ = self.submit_order(user_id, stock_code, OrderType::Buy, price, quantity);
                }
                TradingAction::Sell {
                    stock_code,
                    price,
                    quantity,
                } => {
                    let _ =
                        self.submit_order(user_id, stock_code, OrderType::Sell, price, quantity);
                }
                TradingAction::Hold => {}
            }
        }
        Ok(())
    }
}
