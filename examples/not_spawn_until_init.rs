//! This example operates on a sequence of atomic propositions.
//! If it sees init before spawn, it accepts.
//! Otherwise, it rejects.
//!
//! Essentially we consider the LTL: not spawn until init.

use rust_efsm::{MachineBuilder, Transition};

#[derive(Debug, PartialEq)]
enum Ap {
    Init,
    Spawn,
    Other,
}

fn main() {
    tracing_subscriber::fmt::init();
    
    let machine = MachineBuilder::<bool, Ap>::new()
        .with_transition("Accept", Transition::new(
                |i| { *i == Ap::Other },
                |_| { true },
                "Accept",
                |is_init, _| { is_init }))
        .with_transition("Accept", Transition::new(
                |i| { *i == Ap::Init },
                |_| { true },
                "Accept",
                |_, _| { true }))
        .with_transition("Accept", Transition::new(
                |i| { *i == Ap::Spawn },
                |is_init| { *is_init },
                "Accept",
                |_, _| { true }))
        .with_accepting("Accept")
        .build();

    // Should accept.
    assert!(machine.exec("Accept", vec![Ap::Other, Ap::Init, Ap::Spawn]));
    assert!(machine.exec("Accept", vec![Ap::Init, Ap::Other, Ap::Spawn, Ap::Other]));

    // Should reject.
    assert!(machine.exec("Accept", vec![Ap::Spawn, Ap::Other, Ap::Other, Ap::Init]));
}
