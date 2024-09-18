pub type StockCode = String;

pub type UserId = u64;

pub type OrderId = u64;

pub type TradeId = u64;

pub type Price = u32;

pub type Quantity = u32;

pub type PriceLimit = u32;

pub type Timestamp = u32;

// 新的 trait，定义了我们想要的时间戳行为
pub trait TimestampBehavior: Sized {
    fn format(&self) -> String;
    fn parse(s: &str) -> Result<Self, ()>;
}

// 为 u32 实现 TimestampBehavior
impl TimestampBehavior for Timestamp {
    fn format(&self) -> String {
        let hours = self / 3600;
        let minutes = (self % 3600) / 60;
        let seconds = self % 60;
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    }

    fn parse(s: &str) -> Result<Self, ()> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 3 {
            return Err(());
        }

        let hours: u32 = parts[0].parse().map_err(|_| ())?;
        let minutes: u32 = parts[1].parse().map_err(|_| ())?;
        let seconds: u32 = parts[2].parse().map_err(|_| ())?;

        Ok(hours * 3600 + minutes * 60 + seconds)
    }
}

// Helper functions for easier conversion
pub fn timestamp_to_string(ts: Timestamp) -> String {
    ts.format()
}

pub fn string_to_timestamp(s: &str) -> Result<Timestamp, ()> {
    Timestamp::parse(s)
}
