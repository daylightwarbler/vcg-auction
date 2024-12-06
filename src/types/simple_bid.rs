//! A simple bid type using strings for bidder and item names, and unsigned
//! integers for bid values and item quantities.
//!
//! ```
//! use vcg_auction::types::SimpleBid;
//! SimpleBid::new("Alice", 10, [("chair", 1)]);
//! ```

use crate::Bid;

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct SimpleBid {
    pub name: String,
    pub value: u64,
    pub items: Vec<(String, u64)>,
}

impl SimpleBid {
    pub fn new<T: Into<String>>(
        name: impl Into<String>,
        value: u64,
        items: impl IntoIterator<Item = (T, u64)>,
    ) -> Self {
        Self {
            name: name.into(),
            value,
            items: items.into_iter().map(|x| (x.0.into(), x.1)).collect::<Vec<(
                String,
                u64,
            )>>(
            ),
        }
    }
}

impl Bid for SimpleBid {
    type Name = String;
    type Value = u64;
    type Item = String;
    type Quantity = u64;

    fn bidder_name(&self) -> &Self::Name {
        &self.name
    }
    fn bid_value(&self) -> &Self::Value {
        &self.value
    }
    fn bid_items(&self) -> &[(Self::Item, Self::Quantity)] {
        &self.items
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vcg_auction;

    #[test]
    fn simple_case() {
        let items = [("chair".into(), 2)];
        let bids = [
            vec![
                SimpleBid::new("Alice", 5, [("chair", 1)]),
                SimpleBid::new("Alice", 7, [("chair", 2)]),
            ],
            vec![SimpleBid::new("Bob", 4, [("chair", 1)])],
        ];
        let result = vcg_auction(&items, &bids).unwrap();
        assert_eq!(result.winning_bids, [&bids[0][0], &bids[1][0]]);
        assert_eq!(result.payments, [(&"Alice".into(), 0), (&"Bob".into(), 2)]);
    }
}
