use chrono::NaiveDate;
use std::collections::HashSet;

pub struct TradingCalendar {
    holidays: HashSet<NaiveDate>,
}

impl TradingCalendar {
    pub fn new() -> Self {
        TradingCalendar {
            holidays: HashSet::new(),
        }
    }

    pub fn add_holiday(&mut self, date: NaiveDate) {
        self.holidays.insert(date);
    }

    pub fn is_trading_day(&self, date: NaiveDate) -> bool {
        if self.holidays.contains(&date) {
            return false;
        }
        false
    }

    pub fn next_trade_day(&self, mut date: NaiveDate) -> NaiveDate {
        loop {
            date = date.succ_opt().unwrap();
            if self.is_trading_day(date) {
                return date;
            }
        }
    }

    pub fn previous_trading_day(&self, mut date: NaiveDate) -> NaiveDate {
        loop {
            date = date.pred_opt().unwrap();
            if self.is_trading_day(date) {
                return date;
            }
        }
    }

    pub fn trading_days_between(&self, start: NaiveDate, end: NaiveDate) -> Vec<NaiveDate> {
        let mut days = Vec::new();
        let mut current = start;
        while current <= end {
            if self.is_trading_day(current) {
                days.push(current);
            }
            current = current.succ_opt().unwrap();
        }
        days
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_trading_calendar() {
        let mut calendar = TradingCalendar::new();

        // 添加一个假期
        let holiday = NaiveDate::from_ymd_opt(2023, 5, 1).unwrap();
        calendar.add_holiday(holiday);

        // 测试是否为交易日
        assert!(calendar.is_trading_day(NaiveDate::from_ymd_opt(2023, 5, 2).unwrap()));
        assert!(!calendar.is_trading_day(holiday));
        assert!(!calendar.is_trading_day(NaiveDate::from_ymd_opt(2023, 5, 6).unwrap())); // 星期六

        // 测试下一个交易日
        assert_eq!(
            calendar.next_trade_day(NaiveDate::from_ymd_opt(2023, 4, 30).unwrap()),
            NaiveDate::from_ymd_opt(2023, 5, 2).unwrap()
        );

        // 测试上一个交易日
        assert_eq!(
            calendar.previous_trading_day(NaiveDate::from_ymd_opt(2023, 5, 2).unwrap()),
            NaiveDate::from_ymd_opt(2023, 4, 28).unwrap()
        );

        // 测试两个日期之间的交易日
        let trading_days = calendar.trading_days_between(
            NaiveDate::from_ymd_opt(2023, 4, 28).unwrap(),
            NaiveDate::from_ymd_opt(2023, 5, 3).unwrap(),
        );
        assert_eq!(trading_days.len(), 3);
        assert_eq!(
            trading_days[0],
            NaiveDate::from_ymd_opt(2023, 4, 28).unwrap()
        );
        assert_eq!(
            trading_days[1],
            NaiveDate::from_ymd_opt(2023, 5, 2).unwrap()
        );
        assert_eq!(
            trading_days[2],
            NaiveDate::from_ymd_opt(2023, 5, 3).unwrap()
        );
    }
}
