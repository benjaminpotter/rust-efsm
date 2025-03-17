//! # Extended Finite State Machine (EFSM)
//!
//! `rust-efsm` provides a Rust implementation of the EFSM mostly defined in \[1\]. In this crate, an EFSM
//! is simply referred to as a [machine](Machine). A machine defines a language by __accepting__
//! and __rejecting__ different input sequences called words. Machines should be specified using the
//! [builder](MachineBuilder).
//!
//! # References
//!
//! \[1\] Cheng, K.-T. & Krishnakumar, A. Automatic Functional Test Generation Using The Extended Finite State Machine Model.

use std::cmp::{max, min};
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Display};
use tracing::info;

type Enable<D, I> = fn(&D, &I) -> bool;

/// Creates a D based on information from an existing D and a new I.
/// It can also use an immutable reference to self.
///
/// It is similar to Enable, because it is called during a transition.
/// However, the Update function may store read-only state.
pub trait Update {
    type D;
    type I;

    // NOTE: ATM, there is only one implementation of update function used for every transition.
    // NOTE: The user can store data in the update state, so they can just switch on some enum.
    // NOTE: I don't know if this is really desirable yet?
    // NOTE: I think the trade off is between suffering dynamic disbatch to enable different
    // updates or using generics but only get one update struct.
    fn update(&self, data: Self::D, input: &Self::I) -> Self::D;
}

/// Describes a single transition relation.
pub struct Transition<D, I, U> {
    pub s_out: String,
    pub enable: Enable<D, I>,
    pub enable_hint: Option<String>,
    pub bound: TransitionBound<D>,
    pub update: U,
}

impl<D, I, U: Default> Default for Transition<D, I, U> {
    fn default() -> Self {
        Transition {
            s_out: "default".into(),
            enable: |_, _| true,
            enable_hint: None,
            bound: TransitionBound::unbounded(),
            update: Default::default(),
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

    /// Returns a copy of self but shifted by amount.
    ///
    /// ```
    /// use rust_efsm::TransitionBound;
    ///
    /// let a = TransitionBound { lower: Some(10), upper: None };
    /// let b = TransitionBound { lower: None, upper: Some(15) };
    /// let c = TransitionBound { lower: Some(10), upper: Some(std::u32::MAX) };
    ///
    /// assert!(a.shifted_by(5) == TransitionBound { lower: Some(15), upper: None });
    /// assert!(b.shifted_by(5) == TransitionBound { lower: Some(5), upper: Some(20) });
    /// assert!(c.shifted_by(5) == TransitionBound { lower: Some(15), upper: None });
    /// ```
    pub fn shifted_by(&self, amount: u32) -> Self {
        let (lower, upper) = self.as_explicit();
        TransitionBound {
            // If overflow, panic.
            lower: Some(lower + amount),

            // If overflow, checked_add will return None.
            // Since None indicates no upper bound, going above u32 MAX should result in None.
            upper: upper.checked_add(amount),
        }
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
struct Location<D> {
    // TODO: Right now state=location and location=state, we need to swap them.
    state: String,
    data: D,
}

impl<D> From<Location<D>> for (String, D) {
    fn from(loc: Location<D>) -> (String, D) {
        (loc.state, loc.data)
    }
}

/// Describes an EFSM.
/// In most cases, use the [builder](MachineBuilder) to specify a machine.
///
/// # See also
///
/// * [MachineBuilder]
pub struct Machine<D, I, U> {
    // Represents the directed graph of states and transitions.
    states: HashMap<String, Vec<Transition<D, I, U>>>,

    // Represents accepting states.
    accepting: HashSet<String>,
}

impl<D: Clone + Debug, I: Debug, U: Update<D = D, I = I>> Machine<D, I, U> {
    fn new(states: HashMap<String, Vec<Transition<D, I, U>>>, accepting: HashSet<String>) -> Self {
        Machine { states, accepting }
    }

    pub fn get_accepting(&self) -> HashSet<String> {
        self.accepting.clone()
    }

    pub fn get_transitions(&self, s: &str) -> Option<&Vec<Transition<D, I, U>>> {
        self.states.get(s)
    }

    /// Checks if the input sequence `is` belongs to the language defined by this machine.
    pub fn exec(&self, s_init: &str, d_init: D, is: Vec<I>) -> bool {
        info!("executing input sequence");

        let mut locations = vec![Location {
            state: s_init.into(),
            data: d_init,
        }];

        for i in is {
            info!("received input {:?}", i);
            info!("from locations {:?}", locations);

            locations = self.transition(&i, locations);

            info!("to locations {:?}", locations);
        }

        info!("reached end of input");
        locations
            .iter()
            .map(|loc| self.accepting.contains(&loc.state))
            .fold(false, |acc, accept| acc || accept)
    }

    fn transition(&self, i: &I, locations: Vec<Location<D>>) -> Vec<Location<D>> {
        let mut next_locs: Vec<Location<D>> = Vec::new();
        for (state, data) in locations.into_iter().map(|loc| loc.into()) {
            if let Some(transitions) = self.states.get(&state) {
                for transition in transitions {
                    if (transition.enable)(&data, &i) {
                        let data = transition.update.update(data.clone(), i);
                        next_locs.push(Location {
                            state: transition.s_out.clone(),
                            data,
                        });
                    }
                }
            }
        }

        next_locs
    }
}

impl<D, I, U: Display> Machine<D, I, U> {
    // TODO: Abstract the conversion of a machine in memory to a static output.
    // TODO: Getting a DOT buffer is one concrete implementation of this functionality.
    // TODO: Another example would be decoding synthesizing the language that a machine implements.
    pub fn get_dot_buffer(&self) -> Vec<u8> {
        let mut buffer = String::new();

        // Begin a new graph definition.
        buffer.push_str("digraph machine {\n");
        buffer.push_str("graph [center=true pad=.5];\n");
        for (state, transitions) in &self.states {
            // Double line for accepting states.
            let peripheries = match self.accepting.contains(state) {
                true => 2,
                false => 1,
            };

            // Define all states as nodes.
            buffer.push_str(&format!(
                "{}[shape=circle,peripheries={}];\n",
                state, peripheries
            ));

            // Define all transitions as directed edges.
            for (transition, tailport) in transitions
                .iter()
                .zip(["e", "w", "n", "s", "ne", "sw", "nw", "se"].iter().cycle())
            {
                let label = format!(
                    "<table border=\"0\"><tr><td><font>{}</font></td></tr> <tr><td bgcolor=\"black\"></td></tr> <tr><td><font>{}</font></td></tr></table>",
                    transition.enable_hint.clone().unwrap_or("true".into()),
                    transition.update
                );

                let def = match *state == transition.s_out {
                    true => &format!(
                        "{} -> {} [label=<{}>, tailport={}, headport={}];\n",
                        state, transition.s_out, label, tailport, tailport
                    ),
                    false => &format!(
                        "{} -> {} [label=<{}>, tailport={}];\n",
                        state, transition.s_out, label, tailport
                    ),
                };

                buffer.push_str(def);
            }
        }

        // Close the graph definition block.
        buffer.push_str("}\n");

        // Return the completed graph definition as UTF-8 bytes.
        buffer.into_bytes()
    }
}

/// Helps with specifying [Machines](Machine).
pub struct MachineBuilder<D, I, U> {
    states: HashMap<String, Vec<Transition<D, I, U>>>,
    accepting: HashSet<String>,
}

impl<D: Default + Clone + Debug, I: Debug, U: Update<D = D, I = I>> MachineBuilder<D, I, U> {
    /// Create a new machine builder.
    pub fn new() -> Self {
        MachineBuilder {
            states: HashMap::new(),
            accepting: HashSet::new(),
        }
    }

    /// Add a transition from state `s_in`.
    pub fn with_transition(mut self, s_in: &str, t: Transition<D, I, U>) -> Self {
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
    pub fn build(self) -> Machine<D, I, U> {
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
