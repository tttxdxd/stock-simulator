use crate::types::{string_to_timestamp, timestamp_to_string, Timestamp};
use serde::Serialize;

#[derive(Clone)]
pub struct ExchangeConfig {
    /** 交易所名称 */
    pub name: String,
    /** 当前时间戳 */
    pub current_timestamp: Timestamp,
    pub trading_periods: Vec<TradingPeriod>,
    pub price_limit_percentage: f64,
    pub ticks_per_trading_day: u32,

    current_period: Option<TradingPeriod>,
    next_period: Option<TradingPeriod>,
}

impl ExchangeConfig {
    pub fn new() -> Self {
        ExchangeConfig {
            name: "模拟交易所".to_string(),
            current_timestamp: 0,
            trading_periods: vec![
                TradingPeriod {
                    name: "集合竞价 可以撤单".to_string(),
                    start_tick: string_to_timestamp("9:15:00").unwrap(), // 9:15:00
                    end_tick: string_to_timestamp("9:19:59").unwrap(),   // 9:19:59
                    period_type: TradingPeriodType::CallAuctionWithCancel,
                    allow_order: true,
                    allow_cancel: true,
                    allow_matching: false,
                    allow_record_price_history: false,
                },
                TradingPeriod {
                    name: "集合竞价 不可以撤单".to_string(),
                    start_tick: string_to_timestamp("9:20:00").unwrap(), // 9:20:00
                    end_tick: string_to_timestamp("9:24:59").unwrap(),   // 9:24:59
                    period_type: TradingPeriodType::CallAuctionNoCancel,
                    allow_order: true,
                    allow_cancel: false,
                    allow_matching: false,
                    allow_record_price_history: false,
                },
                TradingPeriod {
                    name: "开盘集合竞价".to_string(),
                    start_tick: string_to_timestamp("9:25:00").unwrap(), // 9:25:00
                    end_tick: string_to_timestamp("9:29:59").unwrap(),   // 9:29:59
                    period_type: TradingPeriodType::OpeningAuction,
                    allow_order: true,
                    allow_cancel: false,
                    allow_matching: true,
                    allow_record_price_history: false,
                },
                TradingPeriod {
                    name: "连续竞价交易".to_string(),
                    start_tick: string_to_timestamp("9:30:00").unwrap(), // 9:30:00
                    end_tick: string_to_timestamp("11:29:59").unwrap(),  // 11:29:59
                    period_type: TradingPeriodType::ContinuousTrading,
                    allow_order: true,
                    allow_cancel: true,
                    allow_matching: true,
                    allow_record_price_history: true,
                },
                TradingPeriod {
                    name: "午间休市".to_string(),
                    start_tick: string_to_timestamp("11:30:00").unwrap(), // 11:30:00
                    end_tick: string_to_timestamp("13:00:00").unwrap(),   // 13:00:00
                    period_type: TradingPeriodType::MiddayBreak,
                    allow_order: false,
                    allow_cancel: false,
                    allow_matching: false,
                    allow_record_price_history: false,
                },
                TradingPeriod {
                    name: "下午连续竞价交易".to_string(),
                    start_tick: string_to_timestamp("13:00:00").unwrap(), // 13:00:00
                    end_tick: string_to_timestamp("14:56:59").unwrap(),   // 14:57:59
                    period_type: TradingPeriodType::ContinuousTrading,
                    allow_order: true,
                    allow_cancel: true,
                    allow_matching: true,
                    allow_record_price_history: true,
                },
                TradingPeriod {
                    name: "收盘集合竞价".to_string(),
                    start_tick: string_to_timestamp("14:57:00").unwrap(), // 14:57:00
                    end_tick: string_to_timestamp("15:00:00").unwrap(),   // 15:00:00
                    period_type: TradingPeriodType::ClosingAuction,
                    allow_order: true,
                    allow_cancel: false,
                    allow_matching: true,
                    allow_record_price_history: true,
                },
            ],
            price_limit_percentage: 0.10,
            ticks_per_trading_day: 28800,
            current_period: None,
            next_period: None,
        }
    }

    pub fn set_current_timestamp(&mut self, timestamp: Timestamp) {
        self.current_timestamp = timestamp;
        self.current_period = self
            .trading_periods
            .iter()
            .find(|period| {
                self.current_timestamp >= period.start_tick
                    && self.current_timestamp <= period.end_tick
            })
            .cloned();
        self.next_period = self
            .trading_periods
            .iter()
            .find(|period| self.current_timestamp < period.start_tick)
            .cloned();
    }

    pub fn get_current_period(&self) -> Option<&TradingPeriod> {
        self.current_period.as_ref()
    }

    pub fn get_next_period(&self) -> Option<&TradingPeriod> {
        self.next_period.as_ref()
    }

    // 判断当前tick是否可以下单
    pub fn is_allow_order(&self) -> bool {
        self.get_current_period()
            .map_or(false, |period| period.allow_order)
    }

    // 判断当前tick是否可以撤单
    pub fn is_allow_cancel(&self) -> bool {
        self.get_current_period()
            .map_or(false, |period| period.allow_cancel)
    }

    // 判断当前tick是否可以撮合
    pub fn is_allow_matching(&self) -> bool {
        self.get_current_period()
            .map_or(false, |period| period.allow_matching)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct TradingPeriod {
    /** 交易时段名称 */
    pub name: String,
    pub start_tick: Timestamp,
    pub end_tick: Timestamp,
    pub period_type: TradingPeriodType,
    pub allow_order: bool,
    pub allow_cancel: bool,
    pub allow_matching: bool,
    /** 是否记录价格历史 */
    pub allow_record_price_history: bool,
}

impl ToString for TradingPeriod {
    fn to_string(&self) -> String {
        format!(
            "{} - {}",
            timestamp_to_string(self.start_tick),
            timestamp_to_string(self.end_tick)
        )
    }
}
#[derive(Clone, Debug, Serialize)]
pub enum TradingPeriodType {
    /** 集合竞价阶段 交易所接受申报，可以撤单，但不进行撮合 */
    CallAuctionWithCancel,
    /** 集合竞价阶段 交易所只接受申报，不可以撤单，仍然不进行撮合 */
    CallAuctionNoCancel,
    /** 开盘集合竞价 确定开盘价并进行撮合 */
    OpeningAuction,
    /** 连续竞价交易 可下单 可撤单 实时撮合 */
    ContinuousTrading,
    /** 收盘集合竞价 确定收盘价 */
    ClosingAuction,
    /** 闭市 */
    MarketClosed,
    /** 午间休市 */
    MiddayBreak,
}
