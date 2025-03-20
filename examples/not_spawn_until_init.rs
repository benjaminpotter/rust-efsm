//! This example operates on a sequence of atomic propositions.
//! If it sees init before spawn, it accepts.
//! Otherwise, it rejects.
//!
//! Essentially we consider the LTL: not spawn until init.

use rust_efsm::{MachineBuilder, Transition, Update};

#[derive(Clone, Copy, Debug, PartialEq)]
enum Ap {
    Init,
    Spawn,
    Other,
}

enum UpdateType {
    True,
    Identity,
}

struct MachineUpdate(UpdateType);

impl Update for MachineUpdate {
    type D = bool;
    type I = Ap;

    fn update(&self, data: Self::D, _input: &Self::I) -> Self::D {
        match self.0 {
            UpdateType::Identity => data,
            UpdateType::True => true,
        }
    }
}

impl Default for MachineUpdate {
    fn default() -> Self {
        MachineUpdate(UpdateType::True)
    }
}

fn main() {
    tracing_subscriber::fmt::init();

    let machine = MachineBuilder::<bool, Ap, MachineUpdate>::new()
        .with_transition(
            "Accept",
            Transition {
                s_out: "Accept".into(),
                enable: |_, i| *i == Ap::Other,
                update: MachineUpdate(UpdateType::Identity),
                ..Default::default()
            },
        )
        .with_transition(
            "Accept",
            Transition {
                s_out: "Accept".into(),
                enable: |d, i| *i == Ap::Init,
                ..Default::default()
            },
        )
        .with_transition(
            "Accept",
            Transition {
                s_out: "Accept".into(),
                enable: |&is_init, &i| i == Ap::Spawn && is_init,
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
