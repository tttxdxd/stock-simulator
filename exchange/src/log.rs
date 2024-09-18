use serde::Serialize;

use crate::types::*;
use std::collections::{HashMap, VecDeque};

// 交易记录结构体
#[derive(Debug, Clone, Serialize)]
pub struct TradeLog {
    pub id: TradeId,
    pub buyer_id: UserId,
    pub seller_id: UserId,
    pub stock_code: StockCode,
    pub price: Price,
    pub quantity: Quantity,
    pub buy_order_id: OrderId,
    pub sell_order_id: OrderId,
    pub timestamp: Timestamp,
    pub direction: PriceDirection,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum PriceDirection {
    Flat,
    Up,
    Down,
}

impl TradeLog {
    pub fn new(
        buyer_id: UserId,
        seller_id: UserId,
        price: Price,
        quantity: Quantity,
        buy_order_id: OrderId,
        sell_order_id: OrderId,
        direction: PriceDirection,
    ) -> Self {
        TradeLog {
            id: 0,
            buyer_id,
            seller_id,
            stock_code: "".to_string(),
            price,
            quantity,
            buy_order_id,
            sell_order_id,
            timestamp: 0,
            direction,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TradeType {
    Market,
    Limit,
}

pub struct LogManager {
    logs: HashMap<TradeId, TradeLog>,
    id_queue: VecDeque<TradeId>,
    next_id: TradeId,
}

impl LogManager {
    pub fn new() -> Self {
        LogManager {
            logs: HashMap::new(),
            id_queue: VecDeque::new(),
            next_id: 1,
        }
    }

    pub fn add_log(&mut self, mut log: TradeLog) -> TradeId {
        let id = self.next_id;
        log.id = id;
        self.logs.insert(id, log);
        self.id_queue.push_front(id); // 在队列前端添加新的 ID
        self.next_id += 1;
        id
    }

    pub fn get_log(&self, id: TradeId) -> Option<TradeLog> {
        self.logs.get(&id).cloned()
    }

    pub fn get_all_logs(&self) -> Vec<TradeLog> {
        self.id_queue
            .iter()
            .filter_map(|&id| self.logs.get(&id))
            .cloned()
            .collect()
    }

    /** 倒序获取分页数据 */
    pub fn page_logs(
        &self,
        stock_code: &str,
        page: usize,
        page_size: usize,
    ) -> (Vec<TradeLog>, usize) {
        let start = (page - 1) * page_size;
        let end = std::cmp::min(page * page_size, self.id_queue.len());
        let stock_logs = self.get_logs_by_stock(&stock_code.to_string());
        let logs = stock_logs.iter().skip(start).take(end).cloned().collect();
        let total_pages = stock_logs.len();
        (logs, total_pages)
    }

    pub fn get_logs_by_user(&self, user_id: UserId) -> Vec<TradeLog> {
        self.id_queue
            .iter()
            .filter_map(|&id| self.logs.get(&id))
            .filter(|log| log.buyer_id == user_id || log.seller_id == user_id)
            .cloned()
            .collect()
    }

    pub fn get_logs_by_stock(&self, stock_code: &StockCode) -> Vec<TradeLog> {
        self.id_queue
            .iter()
            .filter_map(|&id| self.logs.get(&id))
            .filter(|log| log.stock_code == *stock_code)
            .cloned()
            .collect()
    }
}
