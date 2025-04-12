use crate::bound::TransitionBound;
use num::{Bounded, CheckedAdd};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::Add;
use tracing::{debug, info};

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
    fn update_interval(&self, interval: TransitionBound<Self::D>) -> TransitionBound<Self::D>;
}

#[derive(Clone)]
pub struct AddUpdate<D, I>
where
    D: Add,
{
    amount: D,
    phantom: PhantomData<I>,
}

impl<D, I> Update for AddUpdate<D, I>
where
    D: Add<Output = D> + Bounded + Copy + CheckedAdd,
{
    type D = D;
    type I = I;

    fn update(&self, data: D, _input: &I) -> D {
        data + self.amount
    }
    fn update_interval(&self, interval: TransitionBound<D>) -> TransitionBound<D> {
        let (lower, upper) = interval.as_explicit();
        TransitionBound {
            lower: Some(lower + self.amount),
            upper: upper.checked_add(&self.amount),
        }
    }
}

/// Describes a single transition relation.
#[derive(Clone)]
pub struct Transition<D, I, U> {
    pub to_location: String,
    pub enable: Enable<D, I>,
    pub bound: TransitionBound<D>,
    pub update: U,
}

impl<D, I, U: Default> Default for Transition<D, I, U> {
    fn default() -> Self {
        Transition {
            to_location: "default".into(),
            enable: |_, _| true,
            bound: TransitionBound::unbounded(),
            update: Default::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct State<D> {
    pub location: String,
    pub data: D,
}

impl<D> From<State<D>> for (String, D) {
    fn from(state: State<D>) -> (String, D) {
        (state.location, state.data)
    }
}

impl<D> From<(String, D)> for State<D> {
    fn from(tuple: (String, D)) -> State<D> {
        let (location, data) = tuple;
        State { location, data }
    }
}

/// Describes an EFSM.
/// In most cases, use the [builder](MachineBuilder) to specify a machine.
///
/// # See also
///
/// * [MachineBuilder]
#[derive(Clone)]
pub struct Machine<D, I, U> {
    // Represents the directed graph of locations and transitions.
    locations: HashMap<String, Vec<Transition<D, I, U>>>,

    // Represents accepting locations.
    accepting: HashSet<String>,
}

impl<D, I, U> Machine<D, I, U>
where
    D: Clone + Debug,
    I: Debug,
    U: Update<D = D, I = I>,
{
    /// Checks if the input sequence `input` belongs to the language defined by this machine.
    pub fn exec(&self, location: &str, data: D, input: Vec<I>) -> bool {
        info!("executing input sequence");

        let mut states = vec![State {
            location: location.into(),
            data,
        }];

        for i in input {
            info!("received input {:?}", i);
            info!("in states {:?}", states);

            states = self.transition(&i, states);

            info!("transitioned to states {:?}", states);
        }

        states
            .iter()
            .map(|state| self.accepting.contains(&state.location))
            .fold(false, |acc, accept| acc || accept)
    }
}

impl<D, I, U> Machine<D, I, U> {
    fn new(
        locations: HashMap<String, Vec<Transition<D, I, U>>>,
        accepting: HashSet<String>,
    ) -> Self {
        Machine {
            locations,
            accepting,
        }
    }

    pub fn get_locations(&self) -> &HashMap<String, Vec<Transition<D, I, U>>> {
        &self.locations
    }

    pub fn get_accepting(&self) -> &HashSet<String> {
        &self.accepting
    }

    pub fn get_transitions_from(&self, location: &str) -> Option<&Vec<Transition<D, I, U>>> {
        self.locations.get(location)
    }
}

impl<D, I, U> Machine<D, I, U>
where
    D: Clone,
    U: Update<D = D, I = I>,
{
    pub fn transition(&self, i: &I, states: Vec<State<D>>) -> Vec<State<D>> {
        let mut next_states: Vec<State<D>> = Vec::new();
        for (location, data) in states.into_iter().map(|state| state.into()) {
            if let Some(transitions) = self.locations.get(&location) {
                for transition in transitions {
                    if (transition.enable)(&data, &i) {
                        let data = transition.update.update(data.clone(), i);
                        next_states.push(State {
                            location: transition.to_location.clone(),
                            data,
                        });
                    }
                }
            }
        }

        next_states
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct StateInterval<D>
where
    D: Eq + Hash,
{
    pub location: String,
    pub interval: TransitionBound<D>,
}

impl<D> fmt::Display for StateInterval<D>
where
    D: fmt::Display + Eq + Hash + Bounded + Copy,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.location, self.interval)
    }
}

#[derive(Debug)]
pub struct PathNode<D>
where
    D: Eq + Hash,
{
    idx: usize,
    parent: Option<(usize, TransitionBound<D>)>,
    location: String,
    interval: TransitionBound<D>,
}

impl<D> PathNode<D>
where
    D: Eq + Hash + Clone,
{
    pub fn path_to(&self, table: &[PathNode<D>]) -> impl Iterator<Item = usize> {
        let mut path: Vec<usize> = vec![];
        let mut next = self.idx;

        loop {
            let node = &table[next];
            path.push(next);

            if let Some((parent_idx, _)) = node.parent {
                next = parent_idx;
            } else {
                break;
            }
        }

        path.reverse();
        path.into_iter()
    }
}

impl<D> fmt::Display for PathNode<D>
where
    D: Eq + Hash + fmt::Display + Copy + Bounded,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(loc: {}, interval: {})", self.location, self.interval)
    }
}

#[derive(Debug)]
pub enum MachineError {
    Undecidable,
    FindNonEmptyFailed,
}

impl fmt::Display for MachineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MachineError::Undecidable => write!(f, "{:?}", self),
            MachineError::FindNonEmptyFailed => write!(f, "{:?}", self),
        }
    }
}

impl<D, I, U> Machine<D, I, U> {
    pub fn complement(mut self) -> Result<Machine<D, I, U>, MachineError> {
        // Preconditions:
        // (1) Machine is deterministic.
        // (2) Machine is total i.e. its state is defined for all inputs.
        //
        // TODO: I need some infrastructure for checking these and returing errors.

        let mut rejecting: HashSet<String> = HashSet::new();
        for loc in self.locations.keys() {
            if !self.accepting.contains(loc) {
                rejecting.insert(loc.clone());
            }
        }

        self.accepting = rejecting;
        Ok(self)
    }
}

impl<D, I, U> Machine<D, I, U>
where
    D: Eq + Hash + Clone + Ord + Copy + Bounded + Debug + fmt::Display,
    U: Update<D = D>,
{
    /// Find all StateIntervals that lead to acceptance.
    ///
    /// ```
    /// use rust_efsm::machine::{Machine, MachineBuilder, AddUpdate, Transition, Update};
    /// use rust_efsm::bound::TransitionBound;
    /// let machine = MachineBuilder::<u8, u8, AddUpdate<u8, u8>>::new().build();
    ///
    ///
    /// ```
    pub fn find_non_empty(
        &self,
        location: &str,
    ) -> Result<HashMap<String, TransitionBound<D>>, MachineError> {
        // Prerequisites
        // Deterministic?
        // FIXME: Cycles can cause unbounded execution... I think?
        // All transitions must be bounded.

        // A path is a vector of state intervals.
        // A path is completed when it reaches an accepting state.
        // A path is completed when it reaches a previously validated state interval.
        // All state intervals in a completed path are not sink state intervals.

        let mut safe: HashMap<String, TransitionBound<D>> = HashMap::new();
        for location in &self.accepting {
            safe.insert(location.clone(), TransitionBound::unbounded());
        }

        let mut nodes: Vec<PathNode<D>> = Vec::new();

        let location = String::from(location);
        let path_root = PathNode {
            idx: nodes.len(),
            parent: None,
            interval: TransitionBound::unbounded(),
            location,
        };

        nodes.push(path_root);

        // Depth first search for accepting paths.
        let mut nodes_to_visit: Vec<usize> = vec![0];

        const MAX_NODES: usize = 100;
        while nodes.len() <= MAX_NODES {
            // Check if current node is accepting
            // Check if current node is in safe.
            // If either of these, then add the full path to safe.
            // We do not care if the intervals we push to safe are unique because the hash set will
            // handle that.

            if let Some(idx) = nodes_to_visit.pop() {
                let current = &nodes[idx];

                debug!(
                    "visit {} with interval {}",
                    current.location, current.interval
                );

                // Check if the interval is completely inside of already safe bounds.
                let is_bound = match safe.get(&current.location) {
                    Some(bound) => bound.contains_interval(&current.interval),
                    None => false,
                };

                if is_bound || self.accepting.contains(&current.location) {
                    // Add path to safe.
                    // Traverse up the parents to get the path.

                    debug!("safe:");

                    let path_iter = nodes[idx].path_to(&nodes[..]);
                    for (location, safe_interval) in path_iter
                        .filter_map(|idx| nodes[idx].parent.clone())
                        .map(|(idx, bound)| (nodes[idx].location.clone(), bound))
                    {
                        debug!("    (loc:{}, cond: {})", location, safe_interval);
                        safe.entry(location.clone())
                            .and_modify(|bound| bound.union_with(&safe_interval))
                            .or_insert(safe_interval.clone());
                    }

                    debug!("after adding we have the following safe states:");
                    for (location, interval) in &safe {
                        debug!("    loc: {} is safe over interval: {}", location, interval);
                    }
                }

                // Iterate over transitions out of current node.
                if let Some(transitions) = self.locations.get(&nodes[idx].location) {
                    debug!("exploring transitions");
                    for trans in transitions {
                        // Compute intersection of the current state interval with the transition bounds.
                        // If the resulting state interval is invalid, then continue.
                        // This result indicates that this transition is not enabled from this state interval.

                        let child_idx = nodes.len();
                        let node = &mut nodes[idx];
                        if let Some(postcondition) = node.interval.clone().intersect(&trans.bound) {
                            // Apply the update function to the state interval.
                            // The resulting state interval represents a new node in the path.

                            let location = trans.to_location.clone();
                            let next_interval = trans.update.update_interval(postcondition.clone());

                            debug!("    found: ({}: {})", location, next_interval);
                            let path_node = PathNode {
                                idx: child_idx,
                                parent: Some((idx, postcondition)),
                                interval: next_interval,
                                location,
                            };

                            nodes_to_visit.push(child_idx);
                            nodes.push(path_node);
                        }
                    }
                }
            } else {
                break;
            }
        }

        Ok(safe)
    }
}

/// Helps with specifying [Machines](Machine).
pub struct MachineBuilder<D, I, U> {
    locations: HashMap<String, Vec<Transition<D, I, U>>>,
    accepting: HashSet<String>,
}

impl<D, I, U> MachineBuilder<D, I, U>
where
    D: Default + Clone + Debug,
    I: Debug,
    U: Update<D = D, I = I>,
{
    /// Create a new machine builder.
    pub fn new() -> Self {
        MachineBuilder {
            locations: HashMap::new(),
            accepting: HashSet::new(),
        }
    }

    /// Add a transition from state `from_location`.
    pub fn with_transition(mut self, from_location: &str, transition: Transition<D, I, U>) -> Self {
        info!(
            "add transition {} to {}",
            from_location, transition.to_location
        );
        self.locations
            .entry(from_location.into())
            .or_insert(Vec::new())
            .push(transition);
        self
    }

    /// Mark state `s` as accepting.
    pub fn with_accepting(mut self, location: &str) -> Self {
        info!("mark location {} as accepting", location);
        self.accepting.insert(location.into());
        self
    }

    /// Create and return a new machine from the current specification.
    pub fn build(self) -> Machine<D, I, U> {
        info!(
            "build machine with {} locations",
            self.locations.keys().len()
        );
        Machine::new(self.locations, self.accepting)
    }
}
