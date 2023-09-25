// these wrapper types exist simply to provide the
// notion of an infinite value for the min/max tree
// types, since using the actual f64 infinity value
// could cause overflow errors

use std::cmp::Ordering;
use super::{ NegativeInfinity, Infinity };
use serde::{ Serialize, Deserialize };

#[derive(Clone, Copy)]
#[derive(Serialize, Deserialize)]
pub struct MinNode<P>(Option<P>);

impl<P> Infinity for MinNode<P> {
    fn infinity() -> Self {
        MinNode(None)
    }
}

impl<P : PartialEq> PartialEq for MinNode<P> {
    fn eq(&self, other : &Self) -> bool {
        match (&self.0, &other.0) {
            (None, None) => true,
            (Some(lhs), Some(rhs)) => lhs == rhs,
            _ => false
        }
    }
}

impl<P : PartialOrd> PartialOrd for MinNode<P> {
    fn partial_cmp(&self, other : &Self) -> Option<Ordering> {
        match (&self.0, &other.0) {
            (None, None) => Some(Ordering::Equal),
            (Some(lhs), Some(rhs)) => lhs.partial_cmp(rhs),
            (None, Some(_)) => Some(Ordering::Greater),
            (Some(_), None) => Some(Ordering::Less)
        }
    }
}

impl<P> From<MinNode<P>> for Option<P> {
    fn from(value : MinNode<P>) -> Self {
        value.0
    }
}

impl<P> From<P> for MinNode<P> {
    fn from(value : P) -> Self {
        MinNode(Some(value))
    }
}

#[derive(Clone, Copy)]
#[derive(Serialize, Deserialize)]
pub struct MaxNode<P>(Option<P>);

impl<P> NegativeInfinity for MaxNode<P> {
    fn negative_infinity() -> Self {
        MaxNode(None)
    }
}

impl<P : PartialEq> PartialEq for MaxNode<P> {
    fn eq(&self, other : &Self) -> bool {
        match (&self.0, &other.0) {
            (None, None) => true,
            (Some(lhs), Some(rhs)) => lhs == rhs,
            _ => false
        }
    }
}

impl<P : PartialOrd> PartialOrd for MaxNode<P> {
    fn partial_cmp(&self, other : &Self) -> Option<Ordering> {
        match (&self.0, &other.0) {
            (None, None) => Some(Ordering::Equal),
            (Some(lhs), Some(rhs)) => lhs.partial_cmp(rhs),
            (None, Some(_)) => Some(Ordering::Less),
            (Some(_), None) => Some(Ordering::Greater)
        }
    }
}

impl<P> From<MaxNode<P>> for Option<P> {
    fn from(value : MaxNode<P>) -> Self {
        value.0
    }
}

impl<P> From<P> for MaxNode<P> {
    fn from(value : P) -> Self {
        MaxNode(Some(value))
    }
}
