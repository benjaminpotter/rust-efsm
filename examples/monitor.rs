use rust_efsm::bound::Bound;
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

    fn update<I>(&self, data: Self::D, _input: &I) -> Self::D {
        data + self.amount
    }

    fn update_interval(&self, interval: Bound<Self::D>) -> Bound<Self::D> {
        let (lower, upper) = interval.as_explicit();
        Bound {
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
    tracing_subscriber::fmt::init();

    let machine = MachineBuilder::<u32, u8, AddUpdate>::new()
        .with_transition(
            "s0",
            Transition {
                to_location: "s0".into(),
                enable: |_, letter| *letter != b'b',
                update: 0.into(),
                bound: Bound {
                    lower: None,
                    upper: Some(10),
                },

                ..Default::default()
            },
        )
        .with_transition(
            "s0",
            Transition {
                to_location: "s1".into(),
                enable: |_, letter| *letter == b'b',
                update: 1.into(),
                bound: Bound {
                    lower: None,
                    upper: Some(3),
                },
                ..Default::default()
            },
        )
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
                bound: Bound {
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
            for input in vec![b'c', b'b', b'c'] {
                if let Ok(verdict) = monitor.next(&input) {
                    info!("input: {}, verdict: {:?}", input as char, verdict);

                    if let Some(_) = verdict {
                        break;
                    }
                } else {
                    info!("error");
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
