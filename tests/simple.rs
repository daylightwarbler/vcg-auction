use vcg_auction::{vcg_auction, vcg_auction_with_tiebreaker};

#[derive(Debug, PartialEq)]
struct Bid {
    name: String,
    value: u64,
    items: Vec<(String, u64)>,
}

impl Bid {
    fn new(
        name: impl Into<String>,
        value: u64,
        items: Vec<(String, u64)>,
    ) -> Self {
        Self {
            name: name.into(),
            value,
            items,
        }
    }
}

impl vcg_auction::Bid for Bid {
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

#[test]
fn vickrey_case() {
    let items = vec![("chair".into(), 1)];
    let bids = vec![
        vec![Bid::new("Alice", 10, vec![("chair".into(), 1)])],
        vec![Bid::new("Bob", 20, vec![("chair".into(), 1)])],
    ];
    let result = vcg_auction(&items, &bids).unwrap();
    assert_eq!(result.winning_bids, [&bids[1][0]]);
    assert_eq!(result.payments, [(&"Bob".into(), 10)]);
}

#[test]
fn simple_case() {
    let items = vec![("chair".into(), 2)];
    let bids = vec![
        vec![
            Bid::new("Alice", 5, vec![("chair".into(), 1)]),
            Bid::new("Alice", 7, vec![("chair".into(), 2)]),
        ],
        vec![Bid::new("Bob", 4, vec![("chair".into(), 1)])],
    ];
    let result = vcg_auction(&items, &bids).unwrap();
    assert_eq!(result.winning_bids, [&bids[0][0], &bids[1][0]]);
    assert_eq!(result.payments, [(&"Alice".into(), 0), (&"Bob".into(), 2)]);
}

/// Example from the Wikipedia page
#[test]
fn wikipedia_example() {
    let items = vec![("apple".into(), 2)];
    let bids = vec![
        vec![Bid::new("Alice", 5, vec![("apple".into(), 1)])],
        vec![Bid::new("Bob", 2, vec![("apple".into(), 1)])],
        vec![Bid::new("Carol", 6, vec![("apple".into(), 2)])],
    ];
    let result = vcg_auction(&items, &bids).unwrap();
    assert_eq!(result.winning_bids, [&bids[0][0], &bids[1][0]]);
    assert_eq!(result.payments, [(&"Alice".into(), 4), (&"Bob".into(), 1)]);
}

#[test]
fn mutually_exclusive_bidders() {
    let items = vec![("chair".into(), 2)];
    let bids = vec![
        vec![Bid::new("Alice", 5, vec![("chair".into(), 1)])],
        vec![
            Bid::new("Bob", 4, vec![("chair".into(), 1)]),
            Bid::new("Carol", 3, vec![("chair".into(), 1)]),
        ],
    ];
    let result = vcg_auction(&items, &bids).unwrap();
    assert_eq!(result.winning_bids, [&bids[0][0], &bids[1][0]]);
    assert_eq!(result.payments, [(&"Alice".into(), 0), (&"Bob".into(), 3)]);
}

#[test]
fn no_valid_bids() {
    let items = vec![("apple".into(), 2)];
    let no_bid_cases: &[Vec<Vec<Bid>>] = &[
        vec![],
        vec![vec![]],
        vec![vec![], vec![]],
        // Alice wants 3 apples, but only 2 are available
        vec![vec![Bid::new("Alice", 5, vec![("apple".into(), 3)])]],
    ];
    for no_bid_case in no_bid_cases {
        assert!(vcg_auction(&items, no_bid_case).is_none());
    }
}

#[test]
fn simple_tiebreaker() {
    let items = vec![("chair".into(), 1)];
    let bids = vec![
        vec![Bid::new("Alice", 10, vec![("chair".into(), 1)])],
        vec![Bid::new("Bob", 10, vec![("chair".into(), 1)])],
    ];
    // tiebreak favors Bob
    let tiebreak = |_: &[&Vec<&Bid>]| 1;
    let result = vcg_auction_with_tiebreaker(&items, &bids, tiebreak).unwrap();
    assert_eq!(result.winning_bids, [&bids[1][0]]);
    assert_eq!(result.payments, [(&"Bob".into(), 10),]);
}

#[test]
fn unrelated_bids_same_bidder() {
    let items = vec![("chair".into(), 2), ("table".into(), 1)];
    let bids = vec![
        // Alice wants the chairs and the table independently of each other
        vec![Bid::new("Alice", 10, vec![("chair".into(), 2)])],
        vec![Bid::new("Alice", 5, vec![("table".into(), 1)])],
        vec![Bid::new("Bob", 4, vec![("table".into(), 1)])],
    ];
    let result = vcg_auction(&items, &bids).unwrap();
    assert_eq!(result.winning_bids, [&bids[0][0], &bids[1][0]]);
    assert_eq!(result.payments, [(&"Alice".into(), 4),]);
}
