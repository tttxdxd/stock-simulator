use crate::exchange_error::{ExchangeError, ExchangeResult};
use crate::types::*;
use std::collections::HashMap;

// 委托单结构体
#[derive(Clone, Debug)]
pub struct Order {
    /** 委托单ID */
    pub id: OrderId,
    /** 用户ID */
    pub user_id: UserId,
    /** 股票代码 */
    pub stock_code: StockCode,
    /** 委托类型 */
    pub order_type: OrderType,
    /** 委托价格 */
    pub price: Price,
    /** 委托数量 */
    pub quantity: Quantity,
    /** 剩余可用数量 */
    pub remaining_quantity: Quantity,
    /** 委托时间 */
    pub timestamp: Timestamp,
    /** 执行记录 */
    pub executions: Vec<Execution>,
}

// 新增：执行记录结构体
#[derive(Clone, Debug)]
pub struct Execution {
    /** 执行数量 */
    pub quantity: Quantity,
    /** 执行价格 */
    pub price: Price,
    /** 执行时间 */
    pub timestamp: Timestamp,
}

// 订单类型枚举
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OrderType {
    /** 买入 */
    Buy,
    /** 卖出 */
    Sell,
}

// 新增：订单方法实现
impl Order {
    /** 创建新订单 */
    pub fn new(
        user_id: UserId,
        stock_code: StockCode,
        order_type: OrderType,
        price: Price,
        quantity: Quantity,
    ) -> Self {
        Order {
            id: 0,
            user_id,
            stock_code,
            order_type,
            price,
            quantity,
            remaining_quantity: quantity,
            timestamp: 0,
            executions: Vec::new(),
        }
    }

    /** 总价值 */
    pub fn value(&self) -> u64 {
        self.price as u64 * self.quantity as u64
    }

    /** 是否已完全成交 */
    pub fn is_filled(&self) -> bool {
        self.remaining_quantity == 0
    }

    /** 是否部分成交 */
    pub fn is_partially_filled(&self) -> bool {
        self.remaining_quantity > 0 && self.remaining_quantity < self.quantity
    }

    /** 是否可取消 */
    pub fn is_cancellable(&self) -> bool {
        self.remaining_quantity > 0
    }

    /** 执行委托 */
    pub fn execute(
        &mut self,
        execution_quantity: u32,
        execution_price: u32,
        execution_time: Timestamp,
    ) {
        let actual_execution_quantity = execution_quantity.min(self.remaining_quantity);
        self.remaining_quantity -= actual_execution_quantity;
        self.executions.push(Execution {
            quantity: actual_execution_quantity,
            price: execution_price,
            timestamp: execution_time,
        });
    }

    /** 取消委托 */
    pub fn cancel(&mut self) -> u32 {
        let cancelled_quantity = self.remaining_quantity;
        self.remaining_quantity = 0;
        cancelled_quantity
    }
}

// OrderManager 结构体及其实现
pub struct OrderManager {
    orders: HashMap<u64, Order>,
    next_order_id: u64,
}

impl OrderManager {
    pub fn new() -> Self {
        OrderManager {
            orders: HashMap::new(),
            next_order_id: 1,
        }
    }

    pub fn create_order(
        &mut self,
        user_id: UserId,
        stock_code: StockCode,
        order_type: OrderType,
        price: Price,
        quantity: Quantity,
    ) -> Order {
        let order_id = self.next_order_id;
        self.next_order_id += 1;
        let mut order = Order::new(user_id, stock_code, order_type, price, quantity);
        order.id = order_id;
        self.orders.insert(order_id, order.clone());
        order
    }

    pub fn get_order(&self, order_id: u64) -> Option<&Order> {
        self.orders.get(&order_id)
    }

    pub fn get_order_mut(&mut self, order_id: u64) -> Option<&mut Order> {
        self.orders.get_mut(&order_id)
    }

    pub fn remove_order(&mut self, order_id: u64) -> ExchangeResult<Order> {
        self.orders
            .remove(&order_id)
            .ok_or(ExchangeError::OrderNotFound(order_id))
    }

    pub fn update_order(&mut self, order: Order) -> ExchangeResult<()> {
        if self.orders.contains_key(&order.id) {
            self.orders.insert(order.id, order);
            Ok(())
        } else {
            Err(ExchangeError::OrderNotFound(order.id))
        }
    }

    pub fn get_user_orders(&self, user_id: u64) -> Vec<&Order> {
        self.orders
            .values()
            .filter(|order| order.user_id == user_id)
            .collect()
    }

    pub fn get_stock_orders(&self, stock_code: &str) -> Vec<&Order> {
        self.orders
            .values()
            .filter(|order| order.stock_code == stock_code)
            .collect()
    }

    pub fn clear_orders(&mut self) {
        self.orders.clear();
    }
}
