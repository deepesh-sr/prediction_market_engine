use std::collections::{BTreeMap, VecDeque};

use crate::models::{Fill, Order, Side};

pub struct OrderBook {
    bids: BTreeMap<u64, VecDeque<Order>>,
    asks: BTreeMap<u64, VecDeque<Order>>,
}

impl OrderBook {
    pub fn new() -> Self {
        OrderBook {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }

    pub fn add_order(&mut self, mut order: Order) -> Vec<Fill> {
        let mut fills = Vec::new();
        match order.side {
            Side::Buy => {
                while order.qty > 0 {
                    let Some((&ask_price, _)) = self.asks.first_key_value() else {
                        break;
                    };
                    let queue = self.asks.get_mut(&ask_price).unwrap();
                    let maker = queue.front_mut().unwrap();
                   
                    if order.price >= ask_price {
                        let fill_qty = std::cmp::min(order.qty, maker.qty);
                        let fill = Fill {
                            maker_order_id: maker.id,
                            taker_order_id: order.id,
                            price: maker.price,
                            qty: fill_qty,
                        };
                        maker.qty-=fill_qty;

                        // all qty of makers got sold , hence remove it from queue's vec
                        if maker.qty == 0 {queue.pop_front();}

                        // if queue gets empty then remove it from asks too
                        if queue.is_empty() {self.asks.remove(&ask_price);}

                        order.qty-=fill_qty;
                        fills.push(fill);
                    }else{
                        break;
                    }
                }
                if order.qty > 0 {
                    self.bids.entry(order.price).or_insert_with(VecDeque::new).push_back(order);
                }
            }
            Side::Sell => {}
        }

        fills
    }
}
