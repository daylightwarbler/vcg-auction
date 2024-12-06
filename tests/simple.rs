use pretty_assertions::assert_eq;

use vcg_auction::{types::SimpleBid, vcg_auction, vcg_auction_with_tiebreaker};

#[test]
fn vickrey_case() {
    let items = vec![("chair".into(), 1)];
    let bids = [
        vec![SimpleBid::new("Alice", 10, [("chair", 1)])],
        vec![SimpleBid::new("Bob", 20, [("chair", 1)])],
    ];
    let result = vcg_auction(&items, &bids).unwrap();
    assert_eq!(result.winning_bids, [&bids[1][0]]);
    assert_eq!(result.payments, [(&"Bob".into(), 10)]);
}

#[test]
fn simple_case() {
    let items = vec![("chair".into(), 2)];
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

#[test]
fn non_exclusive_bids() {
    let items = vec![("chair".into(), 1), ("table".into(), 1)];
    let bids = vec![
        vec![SimpleBid::new("Alice", 5, [("chair", 1)])],
        vec![SimpleBid::new("Alice", 10, [("table", 1)])],
        vec![SimpleBid::new("Bob", 20, [("chair", 1), ("table", 1)])],
    ];
    let result = vcg_auction(&items, &bids).unwrap();
    assert_eq!(result.winning_bids, [&bids[2][0]]);
    assert_eq!(result.payments, [(&"Bob".into(), 15)]);
}

/// Example from the Wikipedia page
#[test]
fn wikipedia_example() {
    let items = vec![("apple".into(), 2)];
    let bids = vec![
        vec![SimpleBid::new("Alice", 5, [("apple", 1)])],
        vec![SimpleBid::new("Bob", 2, [("apple", 1)])],
        vec![SimpleBid::new("Carol", 6, [("apple", 2)])],
    ];
    let result = vcg_auction(&items, &bids).unwrap();
    assert_eq!(result.winning_bids, [&bids[0][0], &bids[1][0]]);
    assert_eq!(result.payments, [(&"Alice".into(), 4), (&"Bob".into(), 1)]);
}

#[test]
fn mutually_exclusive_bidders() {
    let items = vec![("chair".into(), 2)];
    let bids = vec![
        vec![SimpleBid::new("Alice", 5, [("chair", 1)])],
        vec![
            SimpleBid::new("Bob", 4, [("chair", 1)]),
            SimpleBid::new("Carol", 3, [("chair", 1)]),
        ],
    ];
    let result = vcg_auction(&items, &bids).unwrap();
    assert_eq!(result.winning_bids, [&bids[0][0], &bids[1][0]]);
    assert_eq!(result.payments, [(&"Alice".into(), 0), (&"Bob".into(), 3)]);
}

#[test]
fn no_valid_bids() {
    let items = vec![("apple".into(), 2)];
    let no_bid_cases: &[Vec<Vec<SimpleBid>>] = &[
        vec![],
        vec![vec![]],
        vec![vec![], vec![]],
        // Alice wants 3 apples, but only 2 are available
        vec![vec![SimpleBid::new("Alice", 5, [("apple", 3)])]],
    ];
    for no_bid_case in no_bid_cases {
        let result = vcg_auction(&items, no_bid_case).unwrap();
        assert!(result.winning_bids.is_empty());
        assert!(result.payments.is_empty());
    }
}

#[test]
fn simple_tiebreaker() {
    let items = vec![("chair".into(), 1)];
    let bids = vec![
        vec![SimpleBid::new("Alice", 10, vec![("chair", 1)])],
        vec![SimpleBid::new("Bob", 10, vec![("chair", 1)])],
    ];
    // tiebreak favors Bob
    let tiebreak = |_: &[Vec<&SimpleBid>]| 1;
    let result = vcg_auction_with_tiebreaker(&items, &bids, tiebreak).unwrap();
    assert_eq!(result.winning_bids, [&bids[1][0]]);
    assert_eq!(result.payments, [(&"Bob".into(), 10),]);
}

#[test]
fn unrelated_bids_same_bidder() {
    let items = vec![("chair".into(), 2), ("table".into(), 1)];
    let bids = vec![
        // Alice wants the chairs and the table independently of each other
        vec![SimpleBid::new("Alice", 10, vec![("chair", 2)])],
        vec![SimpleBid::new("Alice", 5, vec![("table", 1)])],
        vec![SimpleBid::new("Bob", 4, vec![("table", 1)])],
    ];
    let result = vcg_auction(&items, &bids).unwrap();
    assert_eq!(result.winning_bids, [&bids[0][0], &bids[1][0]]);
    assert_eq!(result.payments, [(&"Alice".into(), 4),]);
}
