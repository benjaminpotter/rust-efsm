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
        .with_transition(
            "Accept",
            Transition {
                s_out: "Accept".into(),
                validate: |i| *i == Ap::Other,
                update: |is_init, _| is_init,
                ..Default::default()
            },
        )
        .with_transition(
            "Accept",
            Transition {
                s_out: "Accept".into(),
                validate: |i| *i == Ap::Init,
                ..Default::default()
            },
        )
        .with_transition(
            "Accept",
            Transition {
                s_out: "Accept".into(),
                validate: |i| *i == Ap::Spawn,
                enable: |is_init| *is_init,
                ..Default::default()
            },
        )
        .with_accepting("Accept")
        .build();

    // Should accept.
    assert!(machine.exec("Accept", false, vec![Ap::Other, Ap::Init, Ap::Spawn]));
    assert!(machine.exec(
        "Accept",
        false,
        vec![Ap::Init, Ap::Other, Ap::Spawn, Ap::Other]
    ));

    // Should reject.
    assert!(machine.exec(
        "Accept",
        false,
        vec![Ap::Spawn, Ap::Other, Ap::Other, Ap::Init]
    ));
}
