use rust_efsm::bound::TransitionBound;
use rust_efsm::gviz::GvGraph;
use rust_efsm::machine::{MachineBuilder, Transition, Update};
use rust_efsm::monitor::Monitor;
use std::fmt;
use std::u32;
use tracing::info;

#[derive(Default, Clone)]
struct AddUpdate {
    amount: u32,
}

impl fmt::Display for AddUpdate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "d += {}", self.amount)
    }
}

impl Update for AddUpdate {
    type D = u32;
    type I = u8;

    fn update(&self, data: Self::D, _input: &Self::I) -> Self::D {
        data + self.amount
    }

    fn update_interval(&self, interval: TransitionBound<Self::D>) -> TransitionBound<Self::D> {
        let (lower, upper) = interval.as_explicit();
        TransitionBound {
            lower: Some(lower + self.amount),
            upper: upper.checked_add(self.amount),
        }
    }
}

impl From<u32> for AddUpdate {
    fn from(amount: u32) -> Self {
        AddUpdate { amount }
    }
}

fn main() {
    // Prints INFO events to STDOUT.
    tracing_subscriber::fmt::init();

    // Define a machine following the specification from the first example in the assignment.
    // Machine operates on a u32 register with u8 (ASCII) input.
    let machine = MachineBuilder::<u32, u8, AddUpdate>::new()
        .with_transition(
            "s0",
            Transition {
                to_location: "s0".into(),
                enable: |_, letter| *letter != b'b',
                update: 0.into(),
                bound: TransitionBound {
                    lower: None,
                    upper: Some(10),
                },

                // Notice the omission of certain members which get the default.
                ..Default::default()
            },
        )
        .with_transition(
            "s0",
            Transition {
                to_location: "s1".into(),
                enable: |_, letter| *letter == b'b',

                // Because the From<u32> trait is implemented for AddUpdate, the compiler will know
                // that a 1 here actually means AddUpdate { amount: 1 }.
                update: 1.into(),

                // Here we explicitly set the bounds, which is not required due to ..Default::default pattern below.
                // Since many transitions may not have bounds, we consider this the default.
                // If a member is not explicitly set in the constructor, ..Default::default will fill it with the default value.
                bound: TransitionBound {
                    lower: None,
                    upper: Some(3),
                },

                ..Default::default()
            },
        )
        // Define a similar transition to before,
        // this time an explicit bound is assigned.
        .with_transition(
            "s1",
            Transition {
                to_location: "s1".into(),
                enable: |_, letter| *letter == b'b',
                update: 1.into(),
                ..Default::default()
            },
        )
        .with_transition(
            "s1",
            Transition {
                to_location: "s3".into(),
                enable: |_, letter| *letter != b'b',
                update: 0.into(),
                bound: TransitionBound {
                    lower: None,
                    upper: Some(3),
                },
                ..Default::default()
            },
        )
        .with_accepting("s1")
        .build();

    let machine = (move || {
        let copy = machine.clone();
        if let Ok(mut monitor) = Monitor::new("s0", 0, machine) {
            info!("start monitoring");
            for input in vec![b'b', b'b', b'b'] {
                match monitor.next(&input) {
                    Ok(verdict) => info!("input: {}, verdict: {:?}", input as char, verdict),
                    Err(e) => info!("error: {:?}", e),
                }
            }
        } else {
            info!("invalid monitor");
        }

        copy
    })();

    let machine = (move || {
        let copy = machine.clone();
        if let Ok(mut monitor) = Monitor::new("s0", 0, machine) {
            info!("start monitoring");
            for input in vec![b'b', b'a', b'a'] {
                match monitor.next(&input) {
                    Ok(verdict) => info!("input: {}, verdict: {:?}", input as char, verdict),
                    Err(e) => info!("error: {:?}", e),
                }
            }
        } else {
            info!("invalid monitor");
        }

        copy
    })();

    let gv: GvGraph = machine.into();
    std::fs::write::<_, String>("machine.gv", gv.into()).unwrap();
}
