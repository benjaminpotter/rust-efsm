use crate::bound::Bound;
use crate::machine::{Machine, State, Update};
use num::Bounded;
use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;

/// A monitor for observing and verifying properties of a machine.
///
/// A `Monitor` consists of a prover and a falsifier, which track system behavior
/// to determine if a property is violated or satisfied.
///
/// # Type Parameters
///
/// * `D` - The data type for machine states, must implement `Eq + Hash`
/// * `I` - The input type for the machine
/// * `U` - The update type with update function
///
/// # Examples
///
/// ```
/// use rust_efsm::machine::{Machine, MachineBuilder, Transition, AddUpdate};
/// use rust_efsm::monitor::Monitor;
/// use rust_efsm::bound::Bound;
///
/// // Create a simple counter machine
/// let machine = MachineBuilder::<u32, u32, AddUpdate<u32>>::new()
///     .with_transition("start", Transition {
///         to_location: "running".into(),
///         enable: |_, _| true,
///         bound: Bound::unbounded(),
///         update: AddUpdate { amount: 1 },
///     })
///     .with_accepting("running")
///     .build();
///
/// // Create a monitor to verify properties
/// let mut monitor = Monitor::new("start", 0, machine).unwrap();
///
/// // Process inputs and check for verdicts
/// let verdict = monitor.next(&5).unwrap();
/// ```
pub struct Monitor<D, I, U>
where
    D: Eq + Hash,
{
    prover: PartialMonitor<D, I, U>,
    falsifier: PartialMonitor<D, I, U>,
}

#[derive(Debug)]
/// Errors that can occur during monitor operation.
pub enum MonitorError {
    TransitionFailed(String),
    ConstructionFailed(String),
}

impl<D, I, U> Monitor<D, I, U>
where
    D: Eq + Hash,
{
    /// Creates a new monitor for the given machine starting at the specified location and data.
    ///
    /// The monitor consists of a prover (looking for property satisfaction) and a falsifier
    /// (looking for property violation).
    ///
    /// # Arguments
    ///
    /// * `location` - The initial location in the machine
    /// * `data` - The initial data value
    /// * `machine` - The machine to monitor
    ///
    /// # Returns
    ///
    /// A new `Monitor` instance or an error if construction fails
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_efsm::machine::{Machine, MachineBuilder, AddUpdate};
    /// use rust_efsm::monitor::Monitor;
    ///
    /// let machine = MachineBuilder::<u32, u32, AddUpdate<u32>>::new()
    ///     .with_accepting("valid")
    ///     .build();
    ///
    /// let monitor = Monitor::new("start", 0, machine);
    /// ```
    pub fn new(location: &str, data: D, machine: Machine<D, I, U>) -> Result<Self, MonitorError>
    where
        D: Eq + Hash + Clone + fmt::Debug + Bounded + Ord + Copy + fmt::Display,
        I: Clone,
        U: Clone + Update<D = D>,
    {
        let prover = PartialMonitor::prove_from(location, data, machine.clone())?;
        let falsifier = PartialMonitor::falsify_from(location, data, machine)?;

        Ok(Monitor { prover, falsifier })
    }

    /// Processes the next input and determines if a verdict can be reached.
    ///
    /// The monitor uses both the prover and falsifier to determine if the property is
    /// satisfied (true), violated (false), or still inconclusive (None).
    ///
    /// # Arguments
    ///
    /// * `input` - The input to process
    ///
    /// # Returns
    ///
    /// * `Ok(Some(true))` - Property is satisfied (proven)
    /// * `Ok(Some(false))` - Property is violated (falsified)
    /// * `Ok(None)` - No verdict yet (inconclusive)
    /// * `Err(MonitorError)` - An error occurred during processing
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use rust_efsm::machine::{Machine, MachineBuilder, AddUpdate};
    /// use rust_efsm::monitor::Monitor;
    ///
    /// // Assume we have a machine and monitor already set up
    /// let mut monitor = Monitor::new("start", 0, machine).unwrap();
    ///
    /// // Process an input and check for a verdict
    /// match monitor.next(&42) {
    ///     Ok(Some(true)) => println!("Property satisfied!"),
    ///     Ok(Some(false)) => println!("Property violated!"),
    ///     Ok(None) => println!("Still inconclusive..."),
    ///     Err(e) => println!("Error: {:?}", e),
    /// }
    /// ```
    pub fn next(&mut self, input: &I) -> Result<Option<bool>, MonitorError>
    where
        D: Eq + Hash + Clone + fmt::Debug + Bounded + Ord + Copy + fmt::Display,
        I: Clone,
        U: Clone + Update<D = D>,
    {
        let mut verdict = None;
        if self.prover.next(input)? {
            // Prover found satisfaction.
            verdict = Some(true);
        } else if self.falsifier.next(input)? {
            // Falsifier found violation.
            verdict = Some(false);
        }

        Ok(verdict)
    }
}

/// A partial monitor that tracks one aspect of property verification.
///
/// A partial monitor is used internally by the main Monitor to track either
/// property satisfaction (prove) or property violation (falsify).
///
/// # Type Parameters
///
/// * `D` - The data type for machine states
/// * `I` - The input type for the machine
/// * `U` - The update type with update function
struct PartialMonitor<D, I, U> {
    state: State<D>,
    machine: Machine<D, I, U>,
    non_empty_states: HashMap<String, Bound<D>>,
}

impl<D, I, U> PartialMonitor<D, I, U> {
    /// Creates a prover monitor from the given location, data, and machine.
    ///
    /// A prover monitor is used to detect when a property is satisfied.
    ///
    /// # Arguments
    ///
    /// * `location` - The initial location in the machine
    /// * `data` - The initial data value
    /// * `machine` - The machine to monitor
    ///
    /// # Returns
    ///
    /// A new `PartialMonitor` instance or an error if construction fails
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let prover = PartialMonitor::prove_from("start", 0, machine)?;
    /// ```
    fn prove_from(location: &str, data: D, machine: Machine<D, I, U>) -> Result<Self, MonitorError>
    where
        D: Eq + Hash + Clone + fmt::Debug + Bounded + Ord + Copy + fmt::Display,
        U: Clone + Update<D = D>,
    {
        let complement = machine
            .complement()
            .map_err(|e| MonitorError::ConstructionFailed(format!("complement failed: {}", e)))?;

        PartialMonitor::falsify_from(location, data, complement)
    }

    /// Creates a falsifier monitor from the given location, data, and machine.
    ///
    /// A falsifier monitor is used to detect when a property is violated.
    ///
    /// # Arguments
    ///
    /// * `location` - The initial location in the machine
    /// * `data` - The initial data value
    /// * `machine` - The machine to monitor
    ///
    /// # Returns
    ///
    /// A new `PartialMonitor` instance or an error if construction fails
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let falsifier = PartialMonitor::falsify_from("start", 0, machine)?;
    /// ```
    fn falsify_from(
        location: &str,
        data: D,
        machine: Machine<D, I, U>,
    ) -> Result<Self, MonitorError>
    where
        D: Eq + Hash + Clone + fmt::Debug + Bounded + Ord + Copy + fmt::Display,
        U: Clone + Update<D = D>,
    {
        let location = String::from(location);

        // Find all states
        let non_empty_states = machine
            .find_non_empty(&location)
            .map_err(|e| MonitorError::ConstructionFailed(format!("partial monitor: {}", e)))?;

        // Construct the initial state of the monitor.
        let state = State { location, data };

        Ok(PartialMonitor {
            state,
            machine,
            non_empty_states,
        })
    }

    fn next(&mut self, input: &I) -> Result<bool, MonitorError>
    where
        D: Eq + Hash + Clone + fmt::Debug + Bounded + Ord + Copy + fmt::Display,
        U: Clone + Update<D = D>,
    {
        // Feed the input to the partial monitor using the current state.
        // Record the output state as next.
        let mut next = self.machine.transition(input, vec![self.state.clone()]);

        // If there is more than one next state, return an error.
        if next.len() == 1 {
            let (location, data) = next.pop().expect("the length was just checked").into();

            // If the next state is in the sink state list, then check the non_empty interval.
            if let Some(bound) = self.non_empty_states.get(&location) {
                // If the next state is in the interval, we are still inconclusive.
                if bound.contains(&data) {
                    // This is because a verdict can only be returned when the next state cannot reach an
                    // accepting condition.

                    self.state = (location, data).into();
                    return Ok(false);
                }
            }

            // In this case we are in an empty state with no possible path to an accepting
            // condition.
            // Return a conclusive verdict.
            return Ok(true);
        }

        // The machine is non-deterministic or malformed.
        Err(MonitorError::TransitionFailed(format!(
            "length of states is not 1: {:?}",
            next
        )))
    }
}
