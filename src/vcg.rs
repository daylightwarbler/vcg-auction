//! Main VCG auction implementation.

use std::cmp::Ordering;

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
    let tiebreaker = |options: &[Vec<&B>]| {
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
/// let tiebreak = |_: &[Vec<&Bid>]| 0;
/// ```
///
/// [`vcg_auction`] uses a random uniform tiebreaker.
pub fn vcg_auction_with_tiebreaker<'a, B: Bid>(
    items: &[(B::Item, B::Quantity)],
    exclusive_bid_sets: &'a [Vec<B>],
    tiebreaker: impl FnOnce(&[Vec<&B>]) -> usize,
) -> Option<AuctionResult<'a, B>> {
    let exclusive_bid_sets = exclusive_bid_sets
        .iter()
        .map(|bs| bs.iter().collect::<Vec<_>>())
        .collect::<Vec<_>>();
    // multiple sets of bids could be tied for the highest value
    let (highest_bid_sets, _highest_value) =
        find_highest_value_bid_sets(items, &exclusive_bid_sets);
    let winning_bid_set = if highest_bid_sets.len() <= 1 {
        highest_bid_sets.get(0)?
    } else {
        highest_bid_sets.get(tiebreaker(&highest_bid_sets))?
    };
    let payments =
        calculate_payments(winning_bid_set, items, &exclusive_bid_sets);
    Some(AuctionResult {
        winning_bids: winning_bid_set.to_vec(),
        payments,
    })
}

fn find_highest_value_bid_sets<'a, B: Bid>(
    items: &[(B::Item, B::Quantity)],
    exclusive_bid_sets: &[Vec<&'a B>], // sets of mutually-exclusive bids
) -> (Vec<Vec<&'a B>>, B::Value) {
    let mut highest_value_bid_sets: Vec<Vec<&'a B>> = vec![]; // empty
    let mut highest_value = B::Value::zero();
    // the items selected so far
    let items_selected = items
        .iter()
        .map(|(item, _)| (item, B::Quantity::zero()))
        .collect::<Vec<_>>();
    // annotate the max possible value of each bid set, used to quickly prune
    // the solution space
    let bid_sets_remaining = exclusive_bid_sets
        .iter()
        .filter_map(|bs| {
            bs.iter().map(|b| b.bid_value()).max().map(|max| (bs, max))
        })
        .collect::<Vec<_>>();
    find_highest_value_helper(
        items,
        &items_selected,
        &bid_sets_remaining,
        &[],
        B::Value::zero(),
        &mut highest_value_bid_sets,
        &mut highest_value,
    );
    (highest_value_bid_sets, highest_value)
}

/// Finds valid combinations of bids using recursive backtracking to limit the
/// exploration space where bid combinations are invalid.
fn find_highest_value_helper<'a, B: Bid>(
    item_stock: &[(B::Item, B::Quantity)], // max number of items available
    items_selected: &[(&B::Item, B::Quantity)], // items in selected bids
    bid_sets_remaining: &[(&Vec<&'a B>, &B::Value)], // bid sets to consider
    bids_selected: &[&'a B],               // selected bids
    selected_value: B::Value,
    highest_value_bid_sets: &mut Vec<Vec<&'a B>>, // highest-scoring bid sets
    highest_value: &mut B::Value,                 // highest value found
) {
    // check that the allocated items is not greater than the stock
    for i in 0..items_selected.len() {
        if items_selected[i].1 > item_stock[i].1 {
            // selected bids not valid -> return without further exploring
            return;
        }
    }

    // search reached full depth, check if selected bids are more valuable
    if bid_sets_remaining.is_empty() {
        match selected_value.cmp(highest_value) {
            Ordering::Greater => {
                *highest_value_bid_sets = vec![bids_selected.to_vec()];
                *highest_value = selected_value;
            }
            Ordering::Equal => {
                highest_value_bid_sets.push(bids_selected.to_vec());
            }
            Ordering::Less => (),
        }
        return;
    }

    // check the possible value achievable with remaining bids
    let max_remaining_value = bid_sets_remaining
        .iter()
        .fold(B::Value::zero(), |sum, (_bs, max_bid_value)| {
            sum.add(max_bid_value)
        });
    let possible_value = selected_value.add(&max_remaining_value);
    if possible_value < *highest_value {
        // can't achieve a result with a higher value than we've already
        // found -> return
        return;
    }

    // recurse with next element
    let (next_bid_set, _max_bid_value) = bid_sets_remaining[0];
    for bid in next_bid_set {
        let mut bids_selected_with_new_bid = bids_selected.to_vec();
        bids_selected_with_new_bid.push(bid);
        let mut items_selected_with_new_bid = items_selected
            .iter()
            .map(|(id, qty)| (*id, qty.clone()))
            .collect::<Vec<_>>();
        for (item, qty) in items_selected_with_new_bid.iter_mut() {
            if let Some((_, bid_qty)) =
                bid.bid_items().iter().find(|(id, _)| *id == **item)
            {
                *qty = qty.add(bid_qty)
            }
        }
        find_highest_value_helper(
            item_stock,
            &items_selected_with_new_bid,
            &bid_sets_remaining[1..],
            &bids_selected_with_new_bid,
            selected_value.add(bid.bid_value()),
            highest_value_bid_sets,
            highest_value,
        );
    }
    // also recurse without using any bids from this bid set
    find_highest_value_helper(
        item_stock,
        items_selected,
        &bid_sets_remaining[1..],
        bids_selected,
        selected_value,
        highest_value_bid_sets,
        highest_value,
    );
}

/// Calculate the payments each winning bidder makes given the winning bid set.
fn calculate_payments<'a, B: Bid>(
    winning_bid_set: &[&'a B],
    items: &[(B::Item, B::Quantity)],
    exclusive_bid_sets: &[Vec<&'a B>], // sets of mutually-exclusive bids
) -> Vec<(&'a B::Name, B::Value)> {
    let mut payments = vec![];
    for winning_bid in winning_bid_set {
        let bidder_name = winning_bid.bidder_name();
        if payments.iter().any(|(name, _)| *name == bidder_name) {
            // already calculated this bidder's payment
            continue;
        }
        // find the auction value without this bidder
        let bid_sets_without_bidder = exclusive_bid_sets
            .iter()
            .map(|bs| {
                bs.iter()
                    .filter(|b| *b.bidder_name() != *bidder_name)
                    .copied()
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let auction_value_without_bidder =
            find_highest_value_bid_sets(items, &bid_sets_without_bidder).1;
        // find the value of the bids placed by other bidders
        let value_of_other_bids = winning_bid_set
            .iter()
            .filter(|b| *b.bidder_name() != *bidder_name)
            .fold(B::Value::zero(), |acc, b| acc.add(b.bid_value()));
        // invariant: this subtraction never underflows on unsigned types
        let payment = auction_value_without_bidder.sub(&value_of_other_bids);
        payments.push((winning_bid.bidder_name(), payment));
    }
    payments
}
