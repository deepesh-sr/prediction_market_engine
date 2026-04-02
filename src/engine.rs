use std::sync::{Mutex, atomic::AtomicU64};

use tokio::sync::broadcast;

use crate::{models::Fill, orderbook::OrderBook};

pub struct AppState{
   pub orderbook : Mutex<OrderBook>,
   pub next_order_id : AtomicU64,
    pub fill_sender : broadcast::Sender<Fill>
}

impl AppState {
    
    pub fn new()-> Self{
        let ( fill_sender , _) = broadcast::channel(100);
        AppState { orderbook: Mutex::new(OrderBook::new()), next_order_id: AtomicU64::new(1) , fill_sender }
    }
}