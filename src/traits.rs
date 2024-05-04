use std::ops::{Add, Sub};

use num_traits::Zero;

/// Trait with methods that take two references of the same type and returns the
/// result of addition or subtraction. Used as a trait bound for [`Bid`]
/// associated types.
pub trait AddSubSelf {
    fn add(&self, other: &Self) -> Self;
    fn sub(&self, other: &Self) -> Self;
}

/// Implementation for any type that implements Add and Sub with itself, such as
/// integer and floating point primitives.
impl<T> AddSubSelf for T
where
    T: Add<Output = T> + Sub<Output = T> + Copy,
{
    fn add(&self, other: &Self) -> Self {
        *self + *other
    }
    fn sub(&self, other: &Self) -> Self {
        *self - *other
    }
}

/// Trait for a bid that can be auctioned.
pub trait Bid {
    /// Identifier for bidders. E.g. strings or integers.
    type Name: Eq;
    /// Bid value. E.g. integers. Floats can be used with a type wrapper for
    /// [`Ord`]. See the tests for an example.
    type Value: Ord + AddSubSelf + Zero;
    /// Identifier for items. E.g. strings or integers.
    type Item: Eq;
    /// Quantity of an item. E.g. integers or floats.
    type Quantity: PartialOrd + AddSubSelf + Zero;

    /// Get the name of the bidder.
    fn bidder_name(&self) -> &Self::Name;
    /// Get the value of the bid.
    fn bid_value(&self) -> &Self::Value;
    /// Get the items that are bid on, and their quantities
    fn bid_items(&self) -> &[(Self::Item, Self::Quantity)];
}
