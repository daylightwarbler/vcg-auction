use std::{
    cmp::Ordering,
    ops::{Add, Sub},
};

use num_traits::Zero;

use vcg_auction::vcg_auction;

/// Type wrapper for a float to implement [`Ord`]. Existing wrappers like
/// OrderedFloat<f64> from the `ordered-float` crate are simpler to use in
/// practice than a fully custom implementation like this.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
struct FloatValue(pub f64);

// Trait impls for Ord
impl Eq for FloatValue {}
impl Ord for FloatValue {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.total_cmp(&other.0)
    }
}
impl PartialOrd for FloatValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // delegate to Ord implementation
        Some(self.cmp(other))
    }
}
impl Zero for FloatValue {
    fn zero() -> Self {
        Self(0.0)
    }
    fn is_zero(&self) -> bool {
        self.0 == 0.0
    }
}
impl Add for FloatValue {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}
impl Sub for FloatValue {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
struct FloatBid {
    name: String,
    value: FloatValue,
    items: Vec<(String, f64)>,
}

impl FloatBid {
    fn new(
        name: impl Into<String>,
        value: f64,
        items: Vec<(String, f64)>,
    ) -> Self {
        Self {
            name: name.into(),
            value: FloatValue(value),
            items,
        }
    }
}

impl vcg_auction::Bid for FloatBid {
    type Name = String;
    type Value = FloatValue;
    type Item = String;
    type Quantity = f64;

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

/// Case with floating point bid values and item quantities. Bid values
/// implement [`Ord`] with a custom type wrapper, but crates like
/// `ordered-float` are easier to use in practice.
#[test]
fn simple_float_case() {
    let items = vec![("kg of rare metal".into(), 2.2)];
    let bids = vec![
        vec![
            FloatBid::new("Alice", 5.5, vec![("kg of rare metal".into(), 1.0)]),
            FloatBid::new("Alice", 7.0, vec![("kg of rare metal".into(), 2.1)]),
        ],
        vec![FloatBid::new(
            "Bob",
            4.1,
            vec![("kg of rare metal".into(), 1.1)],
        )],
    ];
    let result = vcg_auction(&items, &bids).unwrap();
    assert_eq!(result.winning_bids, [&bids[0][0], &bids[1][0]]);
    assert_eq!(
        result.payments,
        [
            (&"Alice".into(), FloatValue(0.0)),
            (&"Bob".into(), FloatValue(1.5))
        ]
    );
}
