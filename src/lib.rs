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
//! assert!(machine.exec("init", vec![3, 1, 1, 1, 10, 8]));
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

pub struct TransitionBound<D> {
    pub lower: Option<D>,
    pub upper: Option<D>,
}

impl<D> TransitionBound<D> {
    pub fn unbounded() -> Self {
        TransitionBound {
            lower: None,
            upper: None,
        }
    }
}

impl TransitionBound<u32> {
    pub fn intersect(&self, other: &Self) -> Option<Self> {
        if self.lower > other.upper || self.upper < other.lower {
            None
        } else {
            Some(TransitionBound {
                lower: max(self.lower, other.lower),
                upper: min(self.upper, other.upper),
            })
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

impl<D: Default + Clone + Debug, I: Debug> Machine<D, I> {
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
    pub fn exec(&self, s_init: &str, is: Vec<I>) -> bool {
        info!("executing input sequence");

        let mut b = Block {
            configs: vec![(s_init.into(), Default::default())],
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
