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

pub mod bound;
pub mod gviz;
pub mod machine;
pub mod monitor;

#[cfg(test)]
mod tests {
    use crate::machine::{IdentityUpdate, Machine, MachineBuilder, Transition};
    use crate::monitor::Monitor;
    use std::fmt;

    #[test]
    fn monitor_not() {
        let machine = make_machine();
        let input = vec![1, 2, 3, 4, 0, 4, 3, 2, 1];

        if let Ok(mut monitor) = Monitor::new("safe", input[0], machine) {
            for verdict in input.into_iter().map(|input| monitor.next(&input)) {
                if let Ok(verdict) = verdict {
                    if let Some(verdict) = verdict {
                        // We expect the verdict to be false.
                        assert!(!verdict);

                        return;
                    }

                    // No verdict
                    continue;
                }

                // Err
                break;
            }
        }

        assert!(false);
    }

    fn make_machine() -> Machine<u8, u8, IdentityUpdate<u8>> {
        MachineBuilder::<u8, u8, IdentityUpdate<u8>>::new()
            .with_transition(
                "safe",
                Transition {
                    to_location: "safe".into(),
                    enable: |_, p| *p != 0,
                    ..Default::default()
                },
            )
            .with_transition(
                "safe",
                Transition {
                    to_location: "unsafe".into(),
                    enable: |_, p| *p == 0,
                    ..Default::default()
                },
            )
            .with_transition(
                "unsafe",
                Transition {
                    to_location: "unsafe".into(),
                    ..Default::default()
                },
            )
            .with_accepting("safe")
            .build()
    }
}
