use thiserror::Error;

// 错误类型
#[derive(Error, Debug)]
pub enum ExchangeError {
    #[error("Action not allowed: {action} at {time}")]
    ActionNotAllowed { action: String, time: String },

    #[error("Stock not found: {0}")]
    StockNotFound(String),

    #[error("Order not found: {0}")]
    OrderNotFound(u64),

    #[error("Order not cancellable: {0}")]
    OrderNotCancellable(u64),

    #[error("Insufficient balance")]
    InsufficientBalance,

    #[error("Insufficient stock")]
    InsufficientStock,

    #[error("Invalid order: {0}")]
    InvalidOrder(String),

    #[error("Market closed")]
    MarketClosed,

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Unknown error: {0}")]
    UnknownError(String),

    #[error("User not found: {0}")]
    UserNotFound(u64),

    #[error("Stock already exists: {0}")]
    StockAlreadyExists(String),

    #[error("Price out of limit: {0}")]
    PriceOutOfLimit(String),
}

pub type ExchangeResult<T> = Result<T, ExchangeError>;
