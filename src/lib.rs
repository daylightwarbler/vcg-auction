//! A [Vickrey-Clarke-Groves
//! auction](https://en.wikipedia.org/wiki/Vickrey%E2%80%93Clarke%E2%80%93Groves_auction)
//! library.
//!
//! Given a set of items and bids, the highest value combination of bids is
//! calculated, along with the payments each winner must make. Payments are the
//! harm each winner causes to other bidders.
//!
//! Compatible bid types implement the [`Bid`] trait.
//!
//! The default feature `rand` can be disabled if only the non-tiebreaking
//! implementation is desired.
//!
//! # Bid Combinations
//!
//! Bids associate a bidder name, a bid value, and a collection of items and
//! quantities being bid on. Bids are supplied as a collection of "bid sets",
//! where the bids inside a bid set are mutually-exclusive. Bids that are in
//! separate bid sets are not mutually-exclusive.
//!
//! ```no_test
//! [
//!     [
//!         (Alice, 5, [(chair, 1)]),
//!         (Alice, 7, [(chair, 2)])
//!     ],
//!     [
//!         (Bob, 4, [(chair, 1)])
//!     ]
//! ]
//! ```
//!
//! Here Alice either wants one chair for 5, or two chairs for 7. Her bids are
//! mutually-exclusive, and no outcome is considered where both bids win. Even
//! if three chairs were available, Alice couldn't win all three.
//!
//! Bob's bid is in another bid set, so any combination of Bob's bid with one of
//! Alice's bids is valid.
//!
//! Mutually-exclusive bids let bidders express their demand curves when
//! their valuations are different for different combinations of items.
//!
//! If bidders want to bid for multiple items with unrelated valuations, those
//! bids can be placed in separate bid sets instead of enumerating all
//! combinations.
//!
//! ```no_test
//! [
//!     [
//!         (Alice, 5, [(chair, 1)])
//!     ],
//!     [
//!         (Alice, 10, [(table, 1)])
//!     ],
//!     [
//!         (Bob, 20, [(chair, 1), (table, 1)])
//!     ]
//! ]
//! ```
//!
//! Here Alice wants a chair for 5, a table for 10, or a table and a chair for
//! 15. Bob only wants the chair and table together for 20.
//!
//! Similarly, if bidders are mutually-exclusive they can be put into the same
//! bid set. If Bob and Carol wouldn't want to win a chair if the other person
//! was also going to win a chair, their bids could be expressed like this:
//!
//! ```no_test
//! [
//!     [
//!         (Bob, 4, [(chair, 1)]),
//!         (Carol, 3, [(chair, 1)])
//!     ]
//! ]
//! ```
//!
//! No outcome is considered where both win, even if two chairs are available.
//! Since these bidders are different people, Bob's payment for winning a chair
//! accounts for Carol's exclusion.
//!
//! # Example
//!
//! ```
//! use vcg_auction::{vcg_auction, types::SimpleBid};
//!
//! # // Function wrapper lets us avoid unwrap() in the doc example.
//! #
//! # use std::error::Error;
//! #
//! # fn run_example() -> Option<()> {
//!
//! // Two chairs up for auction.
//! let items = vec![("chair".to_string(), 2)];
//! let bids = [
//!     vec![
//!         SimpleBid::new("Alice", 5, [("chair", 1)]),
//!         SimpleBid::new("Alice", 7, [("chair", 2)]),
//!     ],
//!     vec![SimpleBid::new("Bob", 4, [("chair", 1)])],
//! ];
//! let result = vcg_auction(&items, &bids)?;
//!
//! // Alice and Bob each win a chair.
//! assert_eq!(result.winning_bids, [&bids[0][0], &bids[1][0]]);
//! // Bob's participation in the auction prevented Alice from getting a second
//! // chair for an additional value of 2, so Bob only pays 2. Alice pays
//! // nothing since her participation didn't prevent any other valuable
//! // outcomes.
//! assert_eq!(
//!     result.payments,
//!     [(&"Alice".to_string(), 0), (&"Bob".to_string(), 2)]
//! );
//!
//! // Example from the VCG auction Wikipedia page.
//! let items = vec![("apple".to_string(), 2)];
//! let bids = vec![
//!     vec![SimpleBid::new("Alice", 5, [("apple", 1)])],
//!     vec![SimpleBid::new("Bob", 2, [("apple", 1)])],
//!     vec![SimpleBid::new("Carol", 6, [("apple", 2)])],
//! ];
//! let result = vcg_auction(&items, &bids)?;
//!
//! assert_eq!(result.winning_bids, [&bids[0][0], &bids[1][0]]);
//! assert_eq!(
//!     result.payments,
//!     [(&"Alice".to_string(), 4), (&"Bob".to_string(), 1)]
//! );
//!
//! #     Some(())
//! # }
//! # assert!(run_example().is_some()) // still check for example success
//! ```
//!
//! See the tests directory for examples using floating point numbers for bid
//! values and item quantities, and the
//! [`secrecy`](https://crates.io/crates/secrecy) crate to help keep bid values
//! confidential. For floating point bid values, which must implement [`Ord`],
//! you may want to use
//! [`ordered-float`](https://crates.io/crates/ordered-float) or a similar
//! crate.

#![cfg_attr(docsrs, feature(doc_cfg))]

mod traits;
pub mod types;
mod vcg;

pub use traits::*;
pub use vcg::*;
