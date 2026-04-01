use std::collections::{BTreeMap, VecDeque};

use crate::models::Order;

pub struct OrderBook {
    bids : BTreeMap<u64 , VecDeque<Order>>,
    asks : BTreeMap<u64, VecDeque<Order>>
}

impl OrderBook{

    pub fn new()-> Self{
        OrderBook { bids: BTreeMap::new(), asks: BTreeMap::new() }
    }
}
