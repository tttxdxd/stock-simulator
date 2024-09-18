use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    routing::{get, post},
    serve, Router,
};
use chrono::{Duration, NaiveTime, Timelike};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::Read,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio;
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;

use exchange::{
    config::{ExchangeConfig, TradingPeriod},
    exchange::Exchange,
    exchange_error::ExchangeError,
    order::OrderType,
    trading_strategy::TradingStrategyType,
    types::{OrderId, Price, Quantity, StockCode, Timestamp, UserId},
};

#[derive(Clone)]
struct AppState {
    exchange: Arc<Mutex<Exchange>>,
}

#[derive(Deserialize, ToSchema)]
struct OrderRequest {
    user_id: UserId,
    stock_code: StockCode,
    quantity: Quantity,
    price: Price,
}

#[derive(Serialize, ToSchema)]
struct OrderResponse {
    order_id: OrderId,
}

#[derive(Serialize, ToSchema)]
struct OrderQueue {
    bids: Vec<(Price, u64)>,
    asks: Vec<(Price, u64)>,
}

#[derive(Serialize, ToSchema)]
struct ExchangeDetails {
    name: String,
    current_timestamp: String,
    current_period: TradingPeriod,
}

#[derive(Deserialize, ToSchema)]
struct TradeHistoryParams {
    page: usize,
    page_size: usize,
}

#[derive(Serialize, ToSchema)]
struct TradeHistoryResponse {
    list: Vec<TradeLog>,
    total: usize,
}

#[derive(Deserialize, ToSchema)]
struct PriceHistoryParams {
    start_time: String,
    end_time: String,
}

#[derive(Serialize, ToSchema)]
struct StockInfo {
    code: StockCode,
    name: String,
    start_price: Price,
    opening_price: Price,
    current_price: Price,
    highest_price: Price,
    lowest_price: Price,
    price_amplitude: f64,
    limit_upper: Price,
    limit_lower: Price,
}

#[derive(Serialize, ToSchema)]
struct TradeLog {
    stock_code: StockCode,
    price: Price,
    quantity: Quantity,
    timestamp: Timestamp,
    trade_type: u8,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        home_page,
        get_stocks,
        buy_order,
        sell_order,
        get_order_queue,
        get_stock_detail,
        get_price_history,
        get_trade_history,
        get_exchange_details
    ),
    components(
        schemas(OrderRequest, OrderResponse, OrderQueue, ExchangeDetails, TradeHistoryParams, TradeHistoryResponse, PriceHistoryParams, StockInfo, TradeLog)
    ),
    tags(
        (name = "stock_exchange", description = "Stock Exchange API")
    )
)]
struct ApiDoc;

#[derive(Serialize)]
struct ApiResponse<T> {
    code: u32,
    message: String,
    data: Option<T>,
}

impl<T: Serialize> ApiResponse<T> {
    fn success(data: T) -> Self {
        Self {
            code: 0,
            message: "Success".to_string(),
            data: Some(data),
        }
    }
}

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

// 修改错误处理函数为泛型，并添加 Serialize 约束
fn handle_exchange_error<T: Serialize>(err: ExchangeError) -> ApiResponse<T> {
    let (code, message) = match err {
        ExchangeError::InsufficientBalance => (1001, "Insufficient balance".to_string()),
        ExchangeError::StockNotFound(stock_code) => {
            (1002, format!("Stock not found: {}", stock_code))
        }
        ExchangeError::UserNotFound(user_id) => (1003, format!("User not found: {}", user_id)),
        ExchangeError::OrderNotFound(order_id) => (1004, format!("Order not found: {}", order_id)),
        ExchangeError::OrderNotCancellable(order_id) => {
            (1005, format!("Order not cancellable: {}", order_id))
        }
        ExchangeError::PriceOutOfLimit(stock_code) => (
            1006,
            format!("Price out of limit for stock: {}", stock_code),
        ),
        ExchangeError::ActionNotAllowed { action, time } => {
            (1007, format!("Action not allowed: {} at {}", action, time))
        }
        ExchangeError::StockAlreadyExists(stock_code) => {
            (1008, format!("Stock already exists: {}", stock_code))
        }
        // 添加其他错误类型的处理
        _ => (9999, "Unknown error".to_string()),
    };
    ApiResponse {
        code,
        message,
        data: None,
    }
}

#[tokio::main]
async fn main() {
    let config = ExchangeConfig::new();
    let exchange = Arc::new(Mutex::new(Exchange::new(config)));

    {
        let mut ex = exchange.lock().unwrap();
        ex.add_stock("000001", "平安银行", 15000).unwrap();
        ex.add_stock("000002", "万科A", 280000).unwrap();
        ex.add_user("user1", 100000000).unwrap();
        ex.add_user("user2", 150000000).unwrap();
        ex.add_robot("robot1", 100000000, TradingStrategyType::Random, vec![("000001", 100000)])
            .unwrap();
        ex.add_robot("robot2", 100000000, TradingStrategyType::Random, vec![("000002", 100000)])
            .unwrap();
        ex.add_robot("robot3", 100000000, TradingStrategyType::Random, vec![("000001", 100000), ("000002", 100000)])
            .unwrap();
        ex.add_robot("robot4", 100000000, TradingStrategyType::Random, vec![("000001", 100000), ("000002", 100000)])
            .unwrap();
        ex.add_robot("robot5", 100000000, TradingStrategyType::Random, vec![("000001", 100000), ("000002", 100000)])
            .unwrap();
        ex.add_robot("robot6", 100000000, TradingStrategyType::Random, vec![("000001", 100000), ("000002", 100000)])
            .unwrap();
        ex.add_robot("robot7", 100000000, TradingStrategyType::Random, vec![("000001", 100000), ("000002", 100000)])
            .unwrap();
        ex.add_robot("robot8", 100000000, TradingStrategyType::Random, vec![("000001", 100000), ("000002", 100000)])
            .unwrap();
        ex.add_robot("robot9", 100000000, TradingStrategyType::Random, vec![("000001", 100000), ("000002", 100000)])
            .unwrap();
    }
    let app_state = AppState {
        exchange: exchange.clone(),
    };
    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/", get(home_page))
        .route("/stocks", get(get_stocks))
        .route("/buy", post(buy_order))
        .route("/sell", post(sell_order))
        .route("/order_queue/:stock_code", get(get_order_queue))
        .route("/stock_detail/:stock_code", get(get_stock_detail))
        .route("/price_history/:stock_code", get(get_price_history))
        .route("/trade_history/:stock_code", get(get_trade_history))
        .route("/exchange_details", get(get_exchange_details))
        .with_state(app_state);

    // 启动交易所时间更新任务
    let exchange_clone = exchange.clone();
    tokio::spawn(async move {
        update_exchange_time(exchange_clone).await;
    });

    println!("Server running on http://localhost:3000");
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    serve(listener, app).await.unwrap();
}

async fn update_exchange_time(exchange: Arc<Mutex<Exchange>>) {
    let mut interval = tokio::time::interval(Duration::milliseconds(100).to_std().unwrap());
    let mut time = NaiveTime::from_hms_opt(9, 30, 0).unwrap();
    loop {
        interval.tick().await;
        // 更新交易所时间 每 tick 更新 1 秒
        time = time + Duration::seconds(1);

        let mut ex = exchange.lock().unwrap();
        ex.next_timestamp(&time.format("%H:%M:%S").to_string());

        // 打印当前交易所时间
        println!(
            "Exchange time: {} period: {}",
            exchange::types::timestamp_to_string(ex.get_config().current_timestamp),
            ex.get_config()
                .get_current_period()
                .unwrap()
                .name
                .to_string()
        );

        // 每 3 秒执行一次机器人策略
        if time.second() % 3 == 0 {
            ex.execute_robot_strategies().unwrap();
        }
    }
}

#[utoipa::path(
    get,
    path = "/",
    responses(
        (status = 200, description = "Home page HTML", content_type = "text/html")
    ),
    tag = "stock_exchange"
)]
async fn home_page() -> Html<String> {
    let mut file = File::open("stock_server/templates/index.html").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    Html(contents)
}

#[utoipa::path(
    get,
    path = "/stocks",
    responses(
        (status = 200, description = "List of stocks", body = ApiResponse<Vec<StockInfo>>)
    ),
    tag = "stock_exchange"
)]
async fn get_stocks(State(state): State<AppState>) -> ApiResponse<Vec<StockInfo>> {
    let exchange = state.exchange.lock().unwrap();
    let stocks: Vec<StockInfo> = exchange
        .get_stock_list()
        .into_iter()
        .filter_map(|code| exchange.get_stock_info(&code))
        .map(|stock| StockInfo {
            code: stock.code,
            name: stock.name,
            start_price: stock.start_price,
            opening_price: stock
                .daily_info
                .opening_price
                .unwrap_or(stock.current_price),
            current_price: stock.current_price,
            highest_price: stock.daily_info.highest_price,
            lowest_price: stock.daily_info.lowest_price,
            price_amplitude: stock.daily_info.price_amplitude,
            limit_upper: stock.price_limit.upper,
            limit_lower: stock.price_limit.lower,
        })
        .collect();
    ApiResponse::success(stocks)
}

#[utoipa::path(
    post,
    path = "/buy",
    request_body = OrderRequest,
    responses(
        (status = 200, description = "Buy order placed successfully", body = ApiResponse<OrderResponse>), 
    ),
    tag = "stock_exchange"
)]
async fn buy_order(
    State(state): State<AppState>,
    Json(order_req): Json<OrderRequest>,
) -> ApiResponse<OrderResponse> {
    let mut exchange = state.exchange.lock().unwrap();
    match exchange.submit_order(
        order_req.user_id,
        order_req.stock_code,
        OrderType::Buy,
        order_req.price,
        order_req.quantity,
    ) {
        Ok(order_id) => ApiResponse::success(OrderResponse { order_id }),
        Err(err) => handle_exchange_error(err),
    }
}

#[utoipa::path(
    post,
    path = "/sell",
    request_body = OrderRequest,
    responses(
        (status = 200, description = "Sell order placed successfully", body = ApiResponse<OrderResponse>), 
    ),
    tag = "stock_exchange"
)]
async fn sell_order(
    State(state): State<AppState>,
    Json(order_req): Json<OrderRequest>,
) -> ApiResponse<OrderResponse> {
    let mut exchange = state.exchange.lock().unwrap();
    match exchange.submit_order(
        order_req.user_id,
        order_req.stock_code,
        OrderType::Sell,
        order_req.price,
        order_req.quantity,
    ) {
        Ok(order_id) => ApiResponse::success(OrderResponse { order_id }),
        Err(err) => handle_exchange_error(err),
    }
}

#[utoipa::path(
    get,
    path = "/order_queue/{stock_code}",
    params(
        ("stock_code" = String, Path, description = "Stock code")
    ),
    responses(
        (status = 200, description = "Order queue for the given stock", body = ApiResponse<OrderQueue>), 
    ),
    tag = "stock_exchange"
)]
async fn get_order_queue(
    State(state): State<AppState>,
    Path(stock_code): Path<String>,
) -> ApiResponse<OrderQueue> {
    let exchange = state.exchange.lock().unwrap();
    let (bids, asks) = exchange.get_order_queue(stock_code.clone(), 10);
    ApiResponse::success(OrderQueue { bids, asks })
}

#[utoipa::path(
    get,
    path = "/stock_detail/{stock_code}",
    params(
        ("stock_code" = String, Path, description = "Stock code")
    ),
    responses(
        (status = 200, description = "Stock details for the given stock", body = ApiResponse<StockInfo>)
    ),
    tag = "stock_exchange"
)]
async fn get_stock_detail(
    State(state): State<AppState>,
    Path(stock_code): Path<String>,
) -> ApiResponse<StockInfo> {
    let exchange = state.exchange.lock().unwrap();
    match exchange.get_stock_info(&stock_code) {
        Some(stock) => ApiResponse::success(StockInfo {
            code: stock.code,
            name: stock.name,
            start_price: stock.start_price,
            opening_price: stock
                .daily_info
                .opening_price
                .unwrap_or(stock.current_price),
            current_price: stock.current_price,
            highest_price: stock.daily_info.highest_price,
            lowest_price: stock.daily_info.lowest_price,
            price_amplitude: stock.daily_info.price_amplitude,
            limit_upper: stock.price_limit.upper,
            limit_lower: stock.price_limit.lower,
        }),
        None => handle_exchange_error(ExchangeError::StockNotFound(stock_code)),
    }
}

#[utoipa::path(
    get,
    path = "/price_history/{stock_code}",
    params(
        ("stock_code" = String, Path, description = "Stock code"),
        ("start_time" = String, Query, description = "Start time"),
        ("end_time" = String, Query, description = "End time")
    ),
    responses(
        (status = 200, description = "Price history for the given stock", body = ApiResponse<Vec<(String, Price, Quantity)>>) 
    ),
    tag = "stock_exchange"
)]
async fn get_price_history(
    State(state): State<AppState>,
    Path(stock_code): Path<String>,
    Query(params): Query<PriceHistoryParams>,
) -> ApiResponse<Vec<(String, Price, Quantity)>> {
    let exchange = state.exchange.lock().unwrap();
    let start_time = exchange::types::string_to_timestamp(&params.start_time).unwrap();
    let end_time = exchange::types::string_to_timestamp(&params.end_time).unwrap();
    let price_history = exchange
        .get_price_history(&stock_code)
        .unwrap()
        .iter()
        .filter(|item| item.timestamp >= start_time && item.timestamp <= end_time)
        .map(|item| {
            (
                exchange::types::timestamp_to_string(item.timestamp),
                item.price,
                item.volume,
            )
        })
        .collect();

    ApiResponse::success(price_history)
}

#[utoipa::path(
    get,
    path = "/trade_history/{stock_code}",
    params(
        ("stock_code" = String, Path, description = "Stock code"),
        ("page" = usize, Query, description = "Page number"),
        ("page_size" = usize, Query, description = "Page size")
    ),
    responses(
        (status = 200, description = "Trade history for the given stock", body = ApiResponse<TradeHistoryResponse>) 
    ),
    tag = "stock_exchange"
)]
async fn get_trade_history(
    State(state): State<AppState>,
    Path(stock_code): Path<String>,
    Query(params): Query<TradeHistoryParams>,
) -> ApiResponse<TradeHistoryResponse> {
    let exchange = state.exchange.lock().unwrap();
    let (list, total) = exchange.get_trade_logs(&stock_code, params.page, params.page_size);
    let list = list
        .into_iter()
        .map(|log| TradeLog {
            stock_code: log.stock_code,
            price: log.price,
            quantity: log.quantity,
            timestamp: log.timestamp,
            trade_type: if log.buy_order_id < log.sell_order_id { 0 } else { 1 },
        })
        .collect();
    ApiResponse::success(TradeHistoryResponse { list, total })
}

#[utoipa::path(
    get,
    path = "/exchange_details",
    responses(
        (status = 200, description = "Exchange details", body = ApiResponse<ExchangeDetails>)
    ),
    tag = "stock_exchange"
)]
async fn get_exchange_details(State(state): State<AppState>) -> ApiResponse<ExchangeDetails> {
    let exchange = state.exchange.lock().unwrap();
    let config = exchange.get_config();
    let current_period = config.get_current_period().unwrap();
    let details = ExchangeDetails {
        name: config.name.clone(),
        current_timestamp: exchange::types::timestamp_to_string(config.current_timestamp),
        current_period: current_period.clone(),
    };
    ApiResponse::success(details)
}
