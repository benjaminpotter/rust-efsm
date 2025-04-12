//! # Bound Module
//!
//! This module provides the `Bound<D>` struct for representing numeric intervals with optional
//! lower and upper bounds. It supports operations on bounded ranges including intersection,
//! containment testing, and bound expansion.
//!
//! ## Key Features
//!
//! - Represent intervals with optional lower and upper bounds
//! - Convert between explicit bounds and potentially unbounded (`None`) bounds
//! - Test for interval intersection and containment
//! - Expand bounds to include other intervals
//! - Type-safe handling of minimum and maximum values for the bounded type
//!
//! The `Bound<D>` struct is generic over the type `D`, which must implement appropriate
//! traits according to the operations being performed.
//!
//! ## Examples
//!
//! ```
//! use rust_efsm::bound::Bound;
//!
//! // Create a bound representing all values >= 10
//! let lower_only = Bound { lower: Some(10_u32), upper: None };
//!
//! // Create a bound representing all values <= 20
//! let upper_only = Bound { lower: None, upper: Some(20_u32) };
//!
//! // Find their intersection: the interval [10, 20]
//! let intersection = lower_only.intersect(&upper_only);
//! assert_eq!(intersection, Some(Bound { lower: Some(10_u32), upper: Some(20_u32) }));
//! ```

use num::Bounded;
use std::cmp::{max, min};
use std::fmt;
use std::fmt::Debug;
use std::hash::Hash;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
/// A generic structure representing a bounded interval with optional lower and upper bounds.
///
/// `Bound<D>` can represent various interval types:
/// - Fully bounded intervals with both lower and upper bounds specified
/// - Half-bounded intervals with only a lower or upper bound
/// - Completely unbounded intervals with neither bound specified
///
/// A value of `None` for a bound indicates the absence of that bound (unbounded in that direction).
/// When operations require explicit bounds, the type's minimum or maximum values are used
/// in place of `None` values through the `as_explicit` and `from_explicit` methods.
///
/// # Type Parameters
///
/// * `D` - The domain type of the bounds, typically a numeric type that implements
///   required traits based on the operations being performed.
///
/// # Examples
///
/// ```
/// use rust_efsm::bound::Bound;
///
/// // Bounded on both sides: [5, 10]
/// let fully_bounded = Bound { lower: Some(5_u32), upper: Some(10_u32) };
///
/// // Bounded below only: [5, ∞)
/// let lower_bounded = Bound { lower: Some(5_u32), upper: None };
///
/// // Bounded above only: (-∞, 10]
/// let upper_bounded = Bound { lower: None, upper: Some(10_u32) };
///
/// // Completely unbounded: (-∞, ∞)
/// let unbounded = Bound::<u32>::unbounded();
/// ```
pub struct Bound<D> {
    // TODO: This really needs to be an enum...
    pub lower: Option<D>,
    pub upper: Option<D>,
}

impl<D> fmt::Display for Bound<D>
where
    D: fmt::Display + Bounded + Copy,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (lower, upper) = self.as_explicit();
        write!(f, "[{}, {}]", lower, upper)
    }
}

impl<D> Bound<D> {
    /// Creates a completely unbounded interval where both lower and upper bounds are None.
    ///
    /// ```
    /// use rust_efsm::bound::Bound;
    ///
    /// let unbounded: Bound<u32> = Bound::unbounded();
    /// assert_eq!(unbounded.lower, None);
    /// assert_eq!(unbounded.upper, None);
    /// ```
    pub fn unbounded() -> Self {
        Bound {
            lower: None,
            upper: None,
        }
    }

    /// Converts a bound with possible `None` values to explicit values by replacing
    /// `None` with the respective minimum or maximum value for the type.
    ///
    /// When a lower bound is `None`, it is replaced with the minimum value for type `D`.
    /// When an upper bound is `None`, it is replaced with the maximum value for type `D`.
    ///
    /// ```
    /// use rust_efsm::bound::Bound;
    ///
    /// let a = Bound {
    ///     lower: Some(10_u32),
    ///     upper: None,
    /// };

    /// let b = Bound {
    ///     lower: None,
    ///     upper: Some(15_u32),
    /// };
    ///
    /// assert!(a.as_explicit() == (10, std::u32::MAX));
    /// assert!(b.as_explicit() == (0, 15));
    /// ```
    pub fn as_explicit(&self) -> (D, D)
    where
        D: Bounded + Copy,
    {
        let lower = match self.lower {
            Some(lower) => lower,
            None => D::min_value(),
        };

        let upper = match self.upper {
            Some(upper) => upper,
            None => D::max_value(),
        };

        (lower, upper)
    }

    /// Converts explicit bounds to a `Bound` structure, potentially with `None` values.
    ///
    /// This is the inverse operation of `as_explicit`. When a bound equals the
    /// minimum value for type `D`, the lower bound is set to `None`. When a bound
    /// equals the maximum value for type `D`, the upper bound is set to `None`.
    ///
    /// ```
    /// use rust_efsm::bound::Bound;
    ///
    /// let a = (10, std::u32::MAX);
    /// let b = (0, 15);
    ///
    /// assert!(
    ///     Bound::from_explicit(a)
    ///         == Bound {
    ///             lower: Some(10_u32),
    ///             upper: None,
    ///         }
    /// );
    ///
    /// assert!(
    ///     Bound::from_explicit(b)
    ///         == Bound {
    ///             lower: None,
    ///             upper: Some(15_u32),
    ///         }
    /// );
    /// ```
    pub fn from_explicit(bound: (D, D)) -> Self
    where
        D: Bounded + Copy + Eq,
    {
        let lower = Some(bound.0).filter(|b| !(*b == D::min_value()));
        let upper = Some(bound.1).filter(|b| !(*b == D::max_value()));
        Bound { lower, upper }
    }

    /// Returns inclusive intersection if it exists.
    /// Otherwise, returns None.
    ///
    /// ```
    /// use rust_efsm::bound::Bound;
    ///
    /// let a = Bound { lower: Some(10), upper: None };
    /// let b = Bound { lower: None, upper: Some(15) };
    /// let c = Bound { lower: None, upper: None };
    ///
    /// assert!(a.intersect(&b) == Some(Bound { lower: Some(10), upper: Some(15) }));
    /// assert!(a.intersect(&c) == Some(Bound { lower: Some(10), upper: None }));
    /// assert!(b.intersect(&c) == Some(Bound { lower: None, upper: Some(15) }));
    /// ```
    pub fn intersect(&self, other: &Self) -> Option<Self>
    where
        D: Ord + Copy + Bounded,
    {
        let (s_lower, s_upper) = self.as_explicit();
        let (o_lower, o_upper) = other.as_explicit();

        if s_lower > o_upper || s_upper < o_lower {
            None
        } else {
            Some(Bound::from_explicit((
                max(s_lower, o_lower),
                min(s_upper, o_upper),
            )))
        }
    }

    /// Expands the current bound to include the entire range of another bound.
    ///
    /// This method modifies `self` by adjusting its lower and upper bounds to ensure
    /// that the entire range of `rhs` is contained within it.
    ///
    /// ```
    /// use rust_efsm::bound::Bound;
    ///
    /// let mut a = Bound { lower: Some(10_u32), upper: Some(20_u32) };
    /// let b = Bound { lower: Some(5_u32), upper: Some(15_u32) };
    ///
    /// a.make_contain(&b);
    /// assert_eq!(a.lower, Some(5_u32));
    /// assert_eq!(a.upper, Some(20_u32));
    /// ```
    pub fn make_contain(&mut self, rhs: &Bound<D>)
    where
        D: Ord + Copy + Bounded,
    {
        let (l_lower, l_upper) = self.as_explicit();
        let (r_lower, r_upper) = rhs.as_explicit();

        self.lower = Some(min(l_lower, r_lower));
        self.upper = Some(max(l_upper, r_upper));
    }

    /// Checks if the bound contains a specific value.
    ///
    /// Returns true if the value is within the inclusive range defined by the bound.
    ///
    /// ```
    /// use rust_efsm::bound::Bound;
    ///
    /// let bound = Bound { lower: Some(10_u32), upper: Some(20_u32) };
    ///
    /// assert!(bound.contains(&10_u32));
    /// assert!(bound.contains(&15_u32));
    /// assert!(bound.contains(&20_u32));
    /// assert!(!bound.contains(&5_u32));
    /// assert!(!bound.contains(&25_u32));
    /// ```
    pub fn contains(&self, data: &D) -> bool
    where
        D: Ord + Copy + Bounded,
    {
        let (lower, upper) = self.as_explicit();
        *data >= lower && *data <= upper
    }

    /// Checks if this bound completely contains another bound.
    ///
    /// Returns true if the entire range of `rhs` is within the range of `self`.
    ///
    /// ```
    /// use rust_efsm::bound::Bound;
    ///
    /// let a = Bound { lower: Some(5_u32), upper: Some(25_u32) };
    /// let b = Bound { lower: Some(10_u32), upper: Some(20_u32) };
    /// let c = Bound { lower: Some(15_u32), upper: Some(30_u32) };
    ///
    /// assert!(a.contains_interval(&b)); // [5,25] contains [10,20]
    /// assert!(!a.contains_interval(&c)); // [5,25] doesn't contain [15,30]
    /// assert!(!b.contains_interval(&a)); // [10,20] doesn't contain [5,25]
    /// ```
    pub fn contains_interval(&self, rhs: &Bound<D>) -> bool
    where
        D: Ord + Copy + Bounded,
    {
        let (ll, lu) = self.as_explicit();
        let (rl, ru) = rhs.as_explicit();
        ll <= rl && lu >= ru
    }
}
