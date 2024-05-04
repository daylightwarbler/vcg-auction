//! Main VCG auction implementation.

use itertools::Itertools;
use num_traits::Zero;
#[cfg(feature = "rand")]
use rand::{thread_rng, Rng};

use crate::{AddSubSelf, Bid};

/// Result of a VCG auction. Contains the set of winning bids, and the payments
/// to be made by each bidder.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct AuctionResult<'a, B: Bid> {
    pub winning_bids: Vec<&'a B>,
    pub payments: Vec<(&'a B::Name, B::Value)>,
}

/// Calculate a Vickrey-Clarke-Groves auction. Takes a set of items with the
/// quantities to be auctioned, and a collection of "bid sets", each containing
/// bids which are mutually-exclusive of one another. Bids are typically grouped
/// by bidder in these mutually-exclusive bid sets, but bidders can also put
/// bids in separate bid sets if they are independent of each other. If multiple
/// outcomes are tied, one is selected at random using a uniform distribution.
#[cfg(feature = "rand")]
#[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
pub fn vcg_auction<'a, B: Bid>(
    items: &[(B::Item, B::Quantity)],
    exclusive_bid_sets: &'a [Vec<B>],
) -> Option<AuctionResult<'a, B>> {
    let tiebreaker = |options: &[&Vec<&B>]| {
        if !options.is_empty() {
            thread_rng().gen_range::<usize, _>(0..options.len())
        } else {
            0
        }
    };
    vcg_auction_with_tiebreaker(items, exclusive_bid_sets, tiebreaker)
}

/// Calculate a VCG auction with a tiebreaking scheme passed in as a closure.
/// The tiebreaker takes a collection of bid sets that all scored the highest,
/// and returns the index of the winning bid set. An invalid index will cause
/// `None` to be returned.
///
/// Here's a simple tiebreaker that always returns the first index of zero.
/// ```
/// # type Bid = usize; // just to get this to compile
/// let tiebreak = |_: &[&Vec<&Bid>]| 0;
/// ```
///
/// [`vcg_auction`] uses a random uniform tiebreaker.
pub fn vcg_auction_with_tiebreaker<'a, B: Bid>(
    items: &[(B::Item, B::Quantity)],
    exclusive_bid_sets: &'a [Vec<B>],
    tiebreaker: impl FnOnce(&[&Vec<&B>]) -> usize,
) -> Option<AuctionResult<'a, B>> {
    let valid_bid_sets = find_all_valid_bid_sets(items, exclusive_bid_sets);
    let scored_bid_sets = score_bid_sets(valid_bid_sets);
    let highest_bid_sets = scored_bid_sets.iter().max_set_by_key(|a| &a.0);
    let winning_bid_set = if highest_bid_sets.len() <= 1 {
        highest_bid_sets.get(0)?
    } else {
        let just_bids =
            highest_bid_sets.iter().map(|x| &x.1).collect::<Vec<_>>();
        highest_bid_sets.get(tiebreaker(&just_bids))?
    };
    let payments = calculate_payments(winning_bid_set, &scored_bid_sets);
    Some(AuctionResult {
        winning_bids: winning_bid_set.1.clone(),
        payments,
    })
}

/// Find all valid sets of bids, given the items to be auctioned and a
/// collection of bid sets that contain mutually-exclusive bids.
fn find_all_valid_bid_sets<'a, B: Bid>(
    items: &[(B::Item, B::Quantity)],
    exclusive_bid_sets: &'a [Vec<B>], // sets of mutually-exclusive bids
) -> Vec<Vec<&'a B>> {
    let mut valid_bid_sets = vec![];
    for i in 0..exclusive_bid_sets.len() + 1 {
        for exclusive_bid_set_combination in
            exclusive_bid_sets.iter().combinations(i)
        {
            // get all possible bid set combinations for the bidders
            let bid_sets = exclusive_bid_set_combination
                .iter()
                .map(|x| x.iter())
                .multi_cartesian_product();
            for bid_set in bid_sets {
                let is_valid = is_valid_bid_set::<B>(items, &bid_set);
                if is_valid {
                    valid_bid_sets.push(bid_set);
                }
            }
        }
    }
    valid_bid_sets
}

/// Determine if a set of bids are valid.
fn is_valid_bid_set<B: Bid>(
    items: &[(B::Item, B::Quantity)],
    bid_set: &[&B],
) -> bool {
    // create a new collection of the auction stock with a mutable field so we
    // can tally up the total items in this bid set and determine if it exceeds
    // the available items
    let mut items_tally = items
        .iter()
        .map(|item| (&item.0, &item.1, B::Quantity::zero()))
        .collect::<Vec<_>>();
    for bid in bid_set {
        for (bid_item, bid_qty) in bid.bid_items().iter() {
            if let Some((_, stock_qty, demanded_qty)) =
                items_tally.iter_mut().find(|i| i.0 == bid_item)
            {
                *demanded_qty =
                    <B::Quantity as AddSubSelf>::add(demanded_qty, bid_qty);

                if *demanded_qty > **stock_qty {
                    return false;
                }
            }
        }
    }
    true
}

/// Score the total social value of a all the bid sets.
fn score_bid_sets<B: Bid>(bid_sets: Vec<Vec<&B>>) -> Vec<(B::Value, Vec<&B>)> {
    bid_sets
        .into_iter()
        .map(|bs| {
            (
                bs.iter().fold(B::Value::zero(), |acc, b| {
                    <B::Value as AddSubSelf>::add(&acc, b.bid_value())
                }),
                bs,
            )
        })
        .collect::<Vec<_>>()
}

/// Calculate the payments each winning bidder makes given the winning bid set
/// and all other valid bid sets.
fn calculate_payments<'a, B: Bid>(
    winning_bid_set: &(B::Value, Vec<&'a B>),
    scored_bid_sets: &[(B::Value, Vec<&'a B>)],
) -> Vec<(&'a B::Name, B::Value)> {
    let mut payments = vec![];
    let zero = B::Value::zero();
    for winning_bid in &winning_bid_set.1 {
        if payments
            .iter()
            .any(|(name, _)| *name == winning_bid.bidder_name())
        {
            // already calculated this bidder's payment
            continue;
        }
        // find the auction value without this bidder
        let auction_value_without_bidder = scored_bid_sets
            .iter()
            .filter(|sbs| {
                !sbs.1
                    .iter()
                    .any(|b| *b.bidder_name() == *winning_bid.bidder_name())
            })
            .max_by_key(|sbs| &sbs.0)
            .map(|sbs| &sbs.0)
            .unwrap_or(&zero);
        // since the same bidder can make multiple unrelated bids that are not
        // mutually-exclusive (as a convenient alternative to listing all the
        // combinations in one bid set), we consider these separated bids as a
        // single bid to determine the total value of the bidder's winning bids
        let value_of_bidders_bids = winning_bid_set
            .1
            .iter()
            .filter(|b| *b.bidder_name() == *winning_bid.bidder_name())
            .fold(B::Value::zero(), |acc, b| {
                <B::Value as AddSubSelf>::add(&acc, b.bid_value())
            });
        // invariant: these subtractions never underflow on unsigned types
        let auction_value_without_bid =
            winning_bid_set.0.sub(&value_of_bidders_bids);
        let payment =
            auction_value_without_bidder.sub(&auction_value_without_bid);
        payments.push((winning_bid.bidder_name(), payment));
    }
    payments
}
