use crate::bound::TransitionBound;
use crate::machine::{Machine, State, Update};
use num::Bounded;
use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;

pub struct Monitor<D, I, U>
where
    D: Eq + Hash,
{
    prover: PartialMonitor<D, I, U>,
    falsifier: PartialMonitor<D, I, U>,
}

#[derive(Debug)]
pub enum MonitorError {
    TransitionFailed(String),
    ConstructionFailed(String),
}

impl<D, I, U> Monitor<D, I, U>
where
    D: Eq + Hash + Clone + fmt::Debug + Bounded + Ord + Copy + fmt::Display,
    I: Clone,
    U: Clone + Update<D = D, I = I>,
{
    pub fn new(location: &str, data: D, machine: Machine<D, I, U>) -> Result<Self, MonitorError> {
        let prover = PartialMonitor::prove_from(location, data, machine.clone())?;
        let falsifier = PartialMonitor::falsify_from(location, data, machine)?;

        Ok(Monitor { prover, falsifier })
    }

    pub fn next(&mut self, input: &I) -> Result<Option<bool>, MonitorError> {
        if self.prover.next(input)? {
            // Prover found tautology.
            return Ok(Some(true));
        } else if self.falsifier.next(input)? {
            // Falsifier found contradiction.
            return Ok(Some(false));
        }

        // Neither partial monitor returned a conclusive verdict.
        return Ok(None);
    }
}

struct PartialMonitor<D, I, U> {
    state: State<D>,
    machine: Machine<D, I, U>,
    non_empty_states: HashMap<String, TransitionBound<D>>,
}

impl<D, I, U> PartialMonitor<D, I, U>
where
    D: Eq + Hash + Clone + fmt::Debug + Bounded + Ord + Copy + fmt::Display,
    U: Clone + Update<D = D, I = I>,
{
    fn prove_from(
        location: &str,
        data: D,
        machine: Machine<D, I, U>,
    ) -> Result<Self, MonitorError> {
        let complement = machine
            .complement()
            .map_err(|e| MonitorError::ConstructionFailed(format!("complement failed: {}", e)))?;

        PartialMonitor::falsify_from(location, data, complement)
    }

    fn falsify_from(
        location: &str,
        data: D,
        machine: Machine<D, I, U>,
    ) -> Result<Self, MonitorError> {
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

    fn next(&mut self, input: &I) -> Result<bool, MonitorError> {
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
