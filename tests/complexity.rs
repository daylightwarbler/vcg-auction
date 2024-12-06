use std::time::Instant;

use rand::{thread_rng, Rng};

use vcg_auction::{vcg_auction, Bid};

/// A fast bid type using only unsigned integers for bidders, items names, bid
/// values and item quantities. This speeds up the computation since bidder and
/// items ids are fast to compare as integers, whereas string comparisons are
/// slightly slower.
///
/// ```
/// use vcg_auction::types::FastBid;
/// let bidder_id = 1;
/// let item_id = 5;
/// FastBid::new(bidder_id, 10, [(item_id, 1)]);
/// ```
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct FastBid {
    pub name: u64,              // bidder id
    pub value: u64,             // bundle utility
    pub items: Vec<(u64, u64)>, // (item id, quantity)
}

impl FastBid {
    pub fn new(
        name: u64,
        value: u64,
        items: impl IntoIterator<Item = (u64, u64)>,
    ) -> Self {
        Self {
            name,
            value,
            items: items.into_iter().collect::<Vec<_>>(),
        }
    }
}

impl Bid for FastBid {
    type Name = u64;
    type Value = u64;
    type Item = u64;
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
fn fastbid_test() {
    let item_id = 5;
    let items = [(item_id, 2)];
    let bids = [
        vec![
            FastBid::new(1, 5, [(item_id, 1)]),
            FastBid::new(1, 7, [(item_id, 2)]),
        ],
        vec![FastBid::new(2, 4, [(item_id, 1)])],
    ];
    let result = vcg_auction(&items, &bids).unwrap();
    assert_eq!(result.winning_bids, [&bids[0][0], &bids[1][0]]);
    assert_eq!(result.payments, [(&1, 0), (&2, 2)]);
}

/// Test X items by X bidders with random valuations, and time how long it takes
/// to compute.
///
/// Pass the `--release` flag to compile with optimizations, and the
/// `--nocapture` flag to show the timing output.
///
/// ```shell
/// cargo test --release -- square_complexity --nocapture
/// ```
#[test]
fn square_complexity() {
    let n_items = 6; // set these values for desired complexity
    let n_bidders = n_items;
    let items = (0..n_items).map(|i| (i, 1)).collect::<Vec<_>>();
    let bids = (0..n_bidders)
        .map(|j| {
            // for each bidder, bid on each item as an independent outcome
            (0..n_items)
                .map(|i| {
                    FastBid::new(
                        j,
                        // random valuation
                        thread_rng().gen_range::<u64, _>(0..100),
                        [(i, 1)],
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let start = Instant::now();
    let _result = vcg_auction(&items, &bids).unwrap();
    println!(
        "{:?} coputation time with {n_items} items and {n_bidders} bidders.",
        Instant::now() - start
    );
}
