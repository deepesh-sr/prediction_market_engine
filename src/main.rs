

use std:: sync::Arc;

use axum::{Json, Router, extract::State, routing::{get, post}};
use prediction_market_engine::{engine::AppState, models::{Fill, Order, Side}};
use serde::{Deserialize, Serialize};
use axum::extract::ws::{WebSocket, WebSocketUpgrade, Message};
use futures::{SinkExt,StreamExt};


#[derive(Deserialize)]
struct OrderRequest{
    side : Side , 
    price : u64 , 
    qty : u64
}

#[derive(Serialize)]
struct OrderResponse{
    order_id : u64 , 
    fills: Vec<Fill>
}
#[tokio::main]
async fn main() {
    
    let state = Arc::new(AppState::new());

    let app = Router::new()
    .route("/health", get(|| async {"Hello Workd"}))
    .route("/order", post(submit_order))
    .route("/orderbook", get(get_orderbook))
    .route("/ws", get(ws_handler))
    .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

pub async fn submit_order(
    State(state): State<Arc<AppState>>,
    Json(payload) : Json<OrderRequest>
)->Result<Json<OrderResponse>, String>{


    let order_id = state.next_order_id.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let mut orderbook = state.orderbook.lock().unwrap();

    let order = Order {
        id : order_id, 
        side : payload.side,
        price : payload.price,
        qty : payload.qty
    };

    let fills = orderbook.add_order(order);

    for fill in &fills { 
       let _ =  state.fill_sender.send(fill.clone());
    }

    Ok(Json(OrderResponse { order_id, fills }))
}
#[derive(Serialize)]
struct OrderBookResponse {
    bids: Vec<(u64, u64)>,   
    asks: Vec<(u64, u64)>,
}

pub async fn get_orderbook (
    State(state): State<Arc<AppState>>
)-> Result<Json<OrderBookResponse>, String>{
    
    let orderbook = state.orderbook.lock().unwrap();

    let bids = orderbook.get_bids().iter().map(|(price,queue)| {
        let total_qty: u64 = queue.iter().map(|o| o.qty).sum();
        (*price,total_qty)
    }).collect();
    
    let asks = orderbook.get_asks().iter().map(|(price,queue)| {
        let total_qty: u64 = queue.iter().map(|o| o.qty).sum();
        (*price,total_qty)
    }).collect();

    Ok(Json(OrderBookResponse{bids ,asks }))
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl axum::response::IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(
    socket : WebSocket, 
    state : Arc<AppState>
){
    let mut rx = state.fill_sender.subscribe();

    let (mut sender , mut _receiver) = socket.split();

    while let Ok(fill) = rx.recv().await {
        let json = serde_json::to_string(&fill).unwrap();

        if sender.send(Message::Text(json.into())).await.is_err(){
            break;
        }
    }
}