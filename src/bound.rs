use num::Bounded;
use std::cmp::{max, min};
use std::fmt;
use std::fmt::Debug;
use std::hash::Hash;

/// Inclusive bound over type D.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct TransitionBound<D> {
    // TODO: This really needs to be an enum...
    pub lower: Option<D>,
    pub upper: Option<D>,
}

impl<D> TransitionBound<D> {
    pub fn unbounded() -> Self {
        // A bound of None indicates there is no bound.
        // This is useful when implementations do not care about bounding D.
        // If we force D to implement Ord, then this might change.
        TransitionBound {
            lower: None,
            upper: None,
        }
    }
}

impl<D> fmt::Display for TransitionBound<D>
where
    D: fmt::Display + Bounded + Copy,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (lower, upper) = self.as_explicit();
        write!(f, "[{}, {}]", lower, upper)
    }
}

impl<D> TransitionBound<D>
where
    D: Bounded + Copy,
{
    // Replaces None with an explict value.
    // This value depends on which generic type we are implementing.
    // For u32, we use [0, std::u32::MAX] as the absolute bounds.
    pub fn as_explicit(&self) -> (D, D) {
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

    // Replaces absolute bounds with None.
    // Inverse operation of as_explicit.
    pub fn from_explicit(bound: (D, D)) -> Self
    where
        D: Eq,
    {
        let lower = Some(bound.0)
            // Set lower to None if it's equal to zero.
            .filter(|b| !(*b == D::min_value()));

        let upper = Some(bound.1)
            // Set upper to None if it's equal to u32 MAX.
            .filter(|b| !(*b == D::max_value()));

        TransitionBound { lower, upper }
    }
}

impl<D> TransitionBound<D>
where
    D: Ord + Copy + Bounded,
{
    // /// Returns a copy of self but shifted by amount.
    // ///
    // /// ```
    // /// use rust_efsm::TransitionBound;
    // ///
    // /// let a = TransitionBound { lower: Some(10), upper: None };
    // /// let b = TransitionBound { lower: None, upper: Some(15) };
    // /// let c = TransitionBound { lower: Some(10), upper: Some(std::u32::MAX) };
    // ///
    // /// assert!(a.shifted_by(5) == TransitionBound { lower: Some(15), upper: None });
    // /// assert!(b.shifted_by(5) == TransitionBound { lower: Some(5), upper: Some(20) });
    // /// assert!(c.shifted_by(5) == TransitionBound { lower: Some(15), upper: None });
    // /// ```
    // pub fn shifted_by(&self, amount: u32) -> Self {
    //     let (lower, upper) = self.as_explicit();
    //     TransitionBound {
    //         // If overflow, panic.
    //         lower: Some(lower + amount),

    //         // If overflow, checked_add will return None.
    //         // Since None indicates no upper bound, going above u32 MAX should result in None.
    //         upper: upper.checked_add(amount),
    //     }
    // }

    /// Returns inclusive intersection if it exists.
    /// Otherwise, returns None.
    ///
    /// ```
    /// use rust_efsm::bound::TransitionBound;
    ///
    /// let a = TransitionBound { lower: Some(10), upper: None };
    /// let b = TransitionBound { lower: None, upper: Some(15) };
    /// let c = TransitionBound { lower: None, upper: None };
    ///
    /// assert!(a.intersect(&b) == Some(TransitionBound { lower: Some(10), upper: Some(15) }));
    /// assert!(a.intersect(&c) == Some(TransitionBound { lower: Some(10), upper: None }));
    /// assert!(b.intersect(&c) == Some(TransitionBound { lower: None, upper: Some(15) }));
    /// ```
    pub fn intersect(&self, other: &Self) -> Option<Self> {
        let (s_lower, s_upper) = self.as_explicit();
        let (o_lower, o_upper) = other.as_explicit();

        if s_lower > o_upper || s_upper < o_lower {
            None
        } else {
            Some(TransitionBound::from_explicit((
                max(s_lower, o_lower),
                min(s_upper, o_upper),
            )))
        }
    }

    pub fn union_with(&mut self, rhs: &TransitionBound<D>) {
        // TODO: disjoint parts???

        let (l_lower, l_upper) = self.as_explicit();
        let (r_lower, r_upper) = rhs.as_explicit();

        // if l_lower > r_upper || l_upper < r_lower {
        //     None
        // } else {
        //     Some(TransitionBound::from_explicit((
        //         min(l_lower, r_lower),
        //         max(l_upper, r_upper),
        //     )))
        // }

        self.lower = Some(min(l_lower, r_lower));
        self.upper = Some(max(l_upper, r_upper));
    }

    pub fn contains(&self, data: &D) -> bool {
        let (lower, upper) = self.as_explicit();
        *data >= lower && *data <= upper
    }

    pub fn contains_interval(&self, rhs: &TransitionBound<D>) -> bool {
        let (ll, lu) = self.as_explicit();
        let (rl, ru) = rhs.as_explicit();
        ll <= rl && lu >= ru
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transition_bound_as_explicit() {
        let a = TransitionBound {
            lower: Some(10_u32),
            upper: None,
        };

        let b = TransitionBound {
            lower: None,
            upper: Some(15_u32),
        };

        assert!(a.as_explicit() == (10, std::u32::MAX));
        assert!(b.as_explicit() == (0, 15));
    }

    #[test]
    fn transition_bound_from_explicit() {
        let a = (10, std::u32::MAX);
        let b = (0, 15);

        assert!(
            TransitionBound::from_explicit(a)
                == TransitionBound {
                    lower: Some(10_u32),
                    upper: None,
                }
        );

        assert!(
            TransitionBound::from_explicit(b)
                == TransitionBound {
                    lower: None,
                    upper: Some(15_u32),
                }
        );
    }
}
