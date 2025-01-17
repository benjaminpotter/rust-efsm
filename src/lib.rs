//! # Extended Finite State Machine (EFSM)
//!
//! `a1` provides a Rust implementation of the EFSM mostly defined in \[1\]. In this crate, an EFSM
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
//! use a1::{MachineBuilder, Transition};
//!
//! let machine = MachineBuilder::new()
//!     .with_transition("init", Transition::new(   // Begin a new transition,
//!         |i: &u32| { *i == 1 },                  // continue if input is one,
//!         |d: &u32| { *d != 2 },                  // continue if counter is not two,
//!         "init",                                 // transition to init,
//!         |d, _| { d + 1 }))                      // increment counter.
//!     .with_transition("init", Transition::new(
//!         |i: &u32| { *i == 1 },
//!         |d: &u32| { *d == 2 },
//!         "accept",
//!         |d, _| { d + 1 }))
//!     .with_transition("accept", Transition::new(
//!         |i: &u32| { *i == 1 },
//!         |d: &u32| { true },
//!         "init",
//!         |d, _| { d + 1 }))
//!     .with_accepting("accept")
//!     .build();
//!
//! assert!(machine.exec("init", vec![3, 1, 1, 1, 10, 8]));
//! assert!(!machine.exec("init", vec![3, 0, 1, 1, 10, 8]));
//! assert!(!machine.exec("init", vec![3, 0, 1, 1, 10, 1, 1]));
//! ```
//!
//! # References
//!
//! \[1\] Cheng, K.-T. & Krishnakumar, A. Automatic Functional Test Generation Using The Extended Finite State Machine Model.

use std::collections::{HashMap, HashSet};
use tracing::{error, info, warn};

type Validate<I> = fn(&I) -> bool;
type Enable<D> = fn(&D) -> bool;
type Update<D, I> = fn(D, &I) -> D;

/// Describes a single transition relation.
///
/// # Generics
/// A transition's validate, enable, and update functions take generic types D and I. These types are expected to be defined by the user and represent the configuration and input data respectively.
pub struct Transition<D, I> {
    // Checks if the given input satisfies transition relation.
    validate: Validate<I>,

    // Checks if current configuration satisfies transition relation.
    enable: Enable<D>,

    // Refers to the next state.
    s_out: String,

    // Updates current configuration on a transition.
    update: Update<D, I>,
}

impl<D, I> Transition<D, I> {
    /// Create a new transition to `s_out` that applies `update` when `validate` and `enable` return true.
    pub fn new(
        validate: Validate<I>,
        enable: Enable<D>,
        s_out: &str,
        update: Update<D, I>,
    ) -> Self {
        Transition {
            validate,
            enable,
            s_out: s_out.into(),
            update,
        }
    }
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

impl<D: Default, I> Machine<D, I> {
    fn new(states: HashMap<String, Vec<Transition<D, I>>>, accepting: HashSet<String>) -> Self {
        Machine { states, accepting }
    }

    /// Checks if the input sequence `is` belongs to the language defined by this machine.
    ///
    /// # Panics
    /// This function will panic if it discovers that the machine specification is invalid.
    pub fn exec(&self, s_init: &str, is: Vec<I>) -> bool {
        info!("executing input sequence");

        let mut rf = D::default();
        let mut s: String = s_init.into();

        for i in is {
            info!("received input");

            let next = match self.states.get(&s) {
                Some(ts) => {
                    let mut next: Option<&Transition<D, I>> = None;
                    for t in ts {
                        if (t.validate)(&i) && (t.enable)(&rf) {
                            if !next.is_none() {
                                panic!(">1 possible transition from state '{}'", s);
                            }

                            next = Some(t);
                        }
                    }
                    next
                }
                None => None,
            };

            if let Some(next) = next {
                info!(
                    "found transition from state '{}' to state '{}'",
                    s, next.s_out
                );
                rf = (next.update)(rf, &i);
                s = next.s_out.clone();
            } else {
                warn!("no valid transition for this input");
            }
        }

        info!("reached end of input in state '{}'", s);
        self.accepting.contains(&s)
    }
}

/// Helps with specifying [Machines](Machine).
pub struct MachineBuilder<D, I> {
    states: HashMap<String, Vec<Transition<D, I>>>,
    accepting: HashSet<String>,
}

impl<D: Default, I> MachineBuilder<D, I> {
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

    #[derive(Clone, Copy)]
    struct Input {
        i1: bool,
        i2: u64,
    }

    #[derive(Default)]
    struct RegisterFile {
        r1: u64,
    }

    #[test]
    fn it_works() {
        tracing_subscriber::fmt::init();

        let machine = MachineBuilder::new()
            .with_transition(
                "s0",
                Transition::new(
                    |i: &Input| i.i1,
                    |rf: &RegisterFile| true,
                    "s1",
                    |mut rf, &i| {
                        rf.r1 = i.i2;
                        rf
                    },
                ),
            )
            .with_transition(
                "s0",
                Transition::new(
                    |i: &Input| !i.i1,
                    |rf: &RegisterFile| true,
                    "s0",
                    |rf, _| rf,
                ),
            )
            .with_transition(
                "s1",
                Transition::new(
                    |i: &Input| i.i1,
                    |rf: &RegisterFile| rf.r1 <= 7,
                    "s1",
                    |mut rf, _| {
                        rf.r1 += 4;
                        rf
                    },
                ),
            )
            .with_transition(
                "s1",
                Transition::new(
                    |i: &Input| i.i1,
                    |rf: &RegisterFile| rf.r1 >= 8,
                    "s0",
                    |rf, _| rf,
                ),
            )
            .with_transition(
                "s1",
                Transition::new(
                    |i: &Input| !i.i1,
                    |rf: &RegisterFile| true,
                    "s1",
                    |rf, _| rf,
                ),
            )
            .with_accepting("s1")
            .build();

        assert!(!machine.exec(
            "s0",
            vec![
                Input { i1: true, i2: 6 },
                Input { i1: false, i2: 0 },
                Input { i1: true, i2: 0 },
                Input { i1: false, i2: 0 },
                Input { i1: false, i2: 0 },
                Input { i1: false, i2: 0 },
                Input { i1: false, i2: 0 },
                Input { i1: false, i2: 0 },
                Input { i1: true, i2: 0 },
            ]
        ));
    }
}
