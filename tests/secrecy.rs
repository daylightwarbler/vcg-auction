//! Test using the `secrecy` crate to help keep bid values confidential.
//!
//! Any bid values are kept in a Secret wrapper so they can be zeroed on drop,
//! even if math was done with them. Combinations of bids can include only a
//! single bid, and so the total value calculated for that combination would
//! just be the value of that bid. Keeping bid values in the Secret wrapper
//! ensures any copies of bid values like this are zeroed out.

use std::fmt;
use std::{cmp::Ordering, ops::Add};

use num_traits::Zero;
use secrecy::{ExposeSecret, Secret};

use vcg_auction::{vcg_auction, AddSubSelf};

struct BidValue(pub Secret<u64>);

// Trait impls for Ord
impl PartialEq for BidValue {
    fn eq(&self, other: &Self) -> bool {
        self.0.expose_secret().eq(other.0.expose_secret())
    }
}
impl Eq for BidValue {}
impl Ord for BidValue {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.expose_secret().cmp(other.0.expose_secret())
    }
}
impl PartialOrd for BidValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // delegate to Ord implementation
        Some(self.cmp(other))
    }
}

impl AddSubSelf for BidValue {
    fn add(&self, other: &Self) -> Self {
        Self(Secret::new(
            self.0.expose_secret() + other.0.expose_secret(),
        ))
    }
    fn sub(&self, other: &Self) -> Self {
        Self(Secret::new(
            self.0.expose_secret() - other.0.expose_secret(),
        ))
    }
}

impl Zero for BidValue {
    fn zero() -> Self {
        Self(Secret::new(0))
    }
    fn is_zero(&self) -> bool {
        *self.0.expose_secret() == 0
    }
}

// Required by Zero
impl Add for BidValue {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(Secret::new(self.0.expose_secret() + rhs.0.expose_secret()))
    }
}

impl fmt::Debug for BidValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "[REDACTED]")
    }
}

#[derive(Debug)]
struct SecretBid {
    name: String,
    value: BidValue,
    items: Vec<(String, u64)>,
}

impl SecretBid {
    fn new(
        name: impl Into<String>,
        value: u64,
        items: Vec<(String, u64)>,
    ) -> Self {
        Self {
            name: name.into(),
            value: BidValue(Secret::new(value)),
            items,
        }
    }
}

impl vcg_auction::Bid for SecretBid {
    type Name = String;
    type Value = BidValue;
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

#[test]
fn simple_secret_case() {
    let items = vec![("chair".into(), 2)];
    let bids = vec![
        vec![
            SecretBid::new("Alice", 5, vec![("chair".into(), 1)]),
            SecretBid::new("Alice", 7, vec![("chair".into(), 2)]),
        ],
        vec![SecretBid::new("Bob", 4, vec![("chair".into(), 1)])],
    ];
    let result = vcg_auction(&items, &bids).unwrap();
    assert_eq!(
        result.payments,
        [
            (&"Alice".into(), BidValue(Secret::new(0))),
            (&"Bob".into(), BidValue(Secret::new(2)))
        ]
    );

    // bid values are redacted from debug logging;
    // println!("{bids:?}");
}
