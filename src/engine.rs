use std::sync::{Mutex, atomic::AtomicU64};

use crate::orderbook::OrderBook;

pub struct AppState{
   pub orderbook : Mutex<OrderBook>,
    pub next_order_id : AtomicU64
}

impl AppState {
    
    pub fn new()-> Self{
        AppState { orderbook: Mutex::new(OrderBook::new()), next_order_id: AtomicU64::new(1) }
    }
}