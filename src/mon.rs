use crate::{Machine, Transition};
use std::collections::{HashMap, HashSet};

pub struct Monitor<D, I, U> {
    prover: PartialMonitor<D, I, U>,
    falsifier: PartialMonitor<D, I, U>,
}

impl<D, I, U> Monitor<D, I, U> {
    pub fn next(self, input: &I) -> Option<bool> {
        None
    }
}

#[derive(Debug)]
pub enum MonitorConstructionError {
    ComplementationFailed,
}

impl<D, I, U> Monitor<D, I, U> {
    pub fn from_machine(property: Machine<D, I, U>) -> Result<Self, MonitorConstructionError> {
        let complement = property
            .complement()
            .map_err(|_| MonitorConstructionError::ComplementationFailed)?;

        let prover = PartialMonitor::from_machine(complement)?;
        let falsifier = PartialMonitor::from_machine(property)?;

        Ok(Monitor { prover, falsifier })
    }
}

struct PartialMonitor<D, I, U> {
    locations: HashMap<String, Vec<Transition<D, I, U>>>,
    rejecting: HashSet<String>,
}

impl<D, I, U> PartialMonitor<D, I, U> {
    fn from_machine(machine: Machine<D, I, U>) -> Result<Self, MonitorConstructionError> {
        Ok(PartialMonitor {
            locations: machine.locations,
            rejecting: HashSet::new(),
        })
    }
}
