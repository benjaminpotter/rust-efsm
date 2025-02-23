//! # Extended Finite State Machine (EFSM)
//!
//! `rust-efsm` provides a Rust implementation of the EFSM mostly defined in \[1\]. In this crate, an EFSM
//! is simply referred to as a [machine](Machine). A machine defines a language by __accepting__
//! and __rejecting__ different input sequences called words. Machines should be specified using the
//! [builder](MachineBuilder).
//!
//! # Example
//!
//! In the following example, we consider a machine that defines the
//! language `{ a | a contains exactly 3 ones }`. So, this language contains words like
//! `1, 10, 34, 1, 1` and `1, 1, 1`, but not `13, 2, 1, 1` and `42, 0, 9, 1, 1, 1, 1`.
//!
//! ```
//! use rust_efsm::{MachineBuilder, Transition};
//!
//! tracing_subscriber::fmt::init();
//!
//! let machine = MachineBuilder::<u32, u32>::new()
//!     .with_transition("init", Transition {
//!         s_out: "init".into(),
//!         validate: |i| *i != 1,
//!         ..Default::default()
//!     })
//!     .with_transition("init", Transition {   // Begin a new transition,
//!         s_out: "init".into(),               // transition to init,
//!         validate: |i| *i == 1,              // continue if input is one,
//!         enable: |d| *d != 2,                // continue if counter is not two,
//!         update: |d, _|  d + 1,              // increment counter.
//!         ..Default::default()
//!     })
//!     .with_transition("init", Transition {
//!         s_out: "accept".into(),
//!         validate: |i| *i == 1,
//!         enable: |d| *d == 2,
//!         update: |d, _| d + 1,
//!         ..Default::default()
//!     })
//!     .with_transition("accept", Transition {
//!         s_out: "accept".into(),
//!         validate: |i|  *i != 0,
//!         update: |d, _| d + 1,
//!         ..Default::default()
//!     })
//!     .with_transition("accept", Transition {
//!         s_out: "init".into(),
//!         validate: |i| *i == 1,
//!         update: |d, _| d + 1,
//!         ..Default::default()
//!     })
//!     .with_accepting("accept")
//!     .build();
//!
//! assert!(machine.exec("init", 0, vec![3, 1, 1, 1, 10, 8]));
//! ```
//!
//! # References
//!
//! \[1\] Cheng, K.-T. & Krishnakumar, A. Automatic Functional Test Generation Using The Extended Finite State Machine Model.

use std::cmp::{max, min};
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use tracing::info;

type Validate<I> = fn(&I) -> bool;
type Enable<D> = fn(&D) -> bool;
type Update<D, I> = fn(D, &I) -> D;

/// Describes a single transition relation.
///
/// # Generics
/// A transition's validate, enable, and update functions take generic types D and I. These types are expected to be defined by the user and represent the configuration and input data respectively.
pub struct Transition<D, I> {
    // Checks if the given input satisfies transition relation.
    pub validate: Validate<I>,

    // Checks if current configuration satisfies transition relation.
    pub enable: Enable<D>,

    pub bound: TransitionBound<D>,

    // Refers to the next state.
    pub s_out: String,

    // Updates current configuration on a transition.
    pub update: Update<D, I>,
}

impl<D, I> Default for Transition<D, I> {
    fn default() -> Self {
        Transition {
            validate: |_| true,
            enable: |_| true,
            bound: TransitionBound::unbounded(),
            s_out: "default".into(),
            update: |d, _| d,
        }
    }
}

/// Inclusive bound over type D.
#[derive(Debug, PartialEq)]
pub struct TransitionBound<D> {
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

// TODO: Can we just require Ord?
impl TransitionBound<u32> {
    // Replaces None with an explict value.
    // This value depends on which generic type we are implementing.
    // For u32, we use [0, std::u32::MAX] as the absolute bounds.
    fn as_explicit(&self) -> (u32, u32) {
        let lower = match self.lower {
            Some(lower) => lower,
            None => 0,
        };

        let upper = match self.upper {
            Some(upper) => upper,
            None => std::u32::MAX,
        };

        (lower, upper)
    }

    // Replaces absolute bounds with None.
    // Inverse operation of as_explicit.
    // TODO: Can we implement this using From<(u32, u32)>?
    fn from_explicit(bound: (u32, u32)) -> Self {
        let lower = Some(bound.0)
            // Set lower to None if it's equal to zero.
            .filter(|b| *b != 0);

        let upper = Some(bound.1)
            // Set upper to None if it's equal to u32 MAX.
            .filter(|b| *b != std::u32::MAX);

        TransitionBound { lower, upper }
    }

    /// Returns inclusive intersection if it exists.
    /// Otherwise, returns None.
    ///
    /// ```
    /// use rust_efsm::TransitionBound;
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
}

#[derive(Debug)]
struct Block<D> {
    configs: Vec<(String, D)>,
}

/// Describes an EFSM and subsequently a regular language. In most cases, use the
/// [builder](MachineBuilder) to specify a machine.
///
/// # See also
///
/// * [MachineBuilder]
pub struct Machine<D, I> {
    // Represents the directed graph of states and transitions.
    states: HashMap<String, Vec<Transition<D, I>>>,

    // Represents accepting states.
    accepting: HashSet<String>,
}

impl<D: Clone + Debug, I: Debug> Machine<D, I> {
    fn new(states: HashMap<String, Vec<Transition<D, I>>>, accepting: HashSet<String>) -> Self {
        Machine { states, accepting }
    }

    pub fn get_accepting(&self) -> HashSet<String> {
        self.accepting.clone()
    }

    pub fn get_transitions(&self, s: &str) -> Option<&Vec<Transition<D, I>>> {
        self.states.get(s)
    }

    /// Checks if the input sequence `is` belongs to the language defined by this machine.
    pub fn exec(&self, s_init: &str, d_init: D, is: Vec<I>) -> bool {
        info!("executing input sequence");

        let mut b = Block {
            configs: vec![(s_init.into(), d_init)],
        };

        for i in is {
            info!("received input {:?}", i);
            info!("from block {:?}", b);

            b = self.transition(&i, b);

            info!("to block {:?}", b);
        }

        info!("reached end of input");
        self.block_accepts(b)
    }

    fn transition(&self, i: &I, b: Block<D>) -> Block<D> {
        let mut configs: Vec<(String, D)> = Vec::new();
        for (state, data) in b.configs {
            if let Some(transitions) = self.states.get(&state) {
                for transition in transitions {
                    if (transition.validate)(&i) && (transition.enable)(&data) {
                        let data = (transition.update)(data.clone(), i);
                        configs.push((transition.s_out.clone(), data));
                    }
                }
            }
        }

        Block { configs }
    }

    fn block_accepts(&self, b: Block<D>) -> bool {
        b.configs
            .iter()
            .map(|(state, _)| self.accepting.contains(state))
            .fold(false, |acc, accept| acc || accept)
    }
}

/// Helps with specifying [Machines](Machine).
pub struct MachineBuilder<D, I> {
    states: HashMap<String, Vec<Transition<D, I>>>,
    accepting: HashSet<String>,
}

impl<D: Default + Clone + Debug, I: Debug> MachineBuilder<D, I> {
    /// Create a new machine builder.
    pub fn new() -> Self {
        MachineBuilder {
            states: HashMap::new(),
            accepting: HashSet::new(),
        }
    }

    /// Add a transition from state `s_in`.
    pub fn with_transition(mut self, s_in: &str, t: Transition<D, I>) -> Self {
        info!(
            "add transition from state '{}' to state '{}'",
            s_in, t.s_out
        );
        self.states.entry(s_in.into()).or_insert(Vec::new()).push(t);
        self
    }

    /// Mark state `s` as accepting.
    pub fn with_accepting(mut self, s: &str) -> Self {
        self.accepting.insert(s.into());
        self
    }

    /// Create and return a new machine from the current specification.
    pub fn build(self) -> Machine<D, I> {
        info!("build machine with {} states", self.states.keys().len());
        Machine::new(self.states, self.accepting)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transition_bound_as_explicit() {
        let a = TransitionBound {
            lower: Some(10),
            upper: None,
        };

        let b = TransitionBound {
            lower: None,
            upper: Some(15),
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
                    lower: Some(10),
                    upper: None,
                }
        );

        assert!(
            TransitionBound::from_explicit(b)
                == TransitionBound {
                    lower: None,
                    upper: Some(15),
                }
        );
    }
}
