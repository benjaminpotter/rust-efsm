use rust_efsm::mon::Monitor;
use rust_efsm::{Machine, MachineBuilder, StateInterval, Transition, TransitionBound, Update};
use std::collections::HashSet;
use std::u32;
use tracing::info;

#[derive(Default)]
struct AddUpdate {
    amount: u32,
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
        // Define a transition from s0 to s1,
        // where input is a 'c',
        // there are no bounds on the transition,
        // and the transition will increment the u32 register.
        .with_transition(
            "s0",
            Transition {
                to_location: "s1".into(),
                enable: |_, letter| *letter == b'c',

                // Because the From<u32> trait is implemented for AddUpdate, the compiler will know
                // that a 1 here actually means AddUpdate { amount: 1 }.
                update: 1.into(),

                // Here we explicitly set the bounds, which is not required due to ..Default::default pattern below.
                // Since many transitions may not have bounds, we consider this the default.
                // If a member is not explicitly set in the constructor, ..Default::default will fill it with the default value.
                bound: TransitionBound::unbounded(),

                ..Default::default()
            },
        )
        .with_transition(
            "s0",
            Transition {
                to_location: "s0".into(),
                enable: |_, letter| *letter == b'b',

                // Notice the omission of certain members which get the default.
                ..Default::default()
            },
        )
        // Define a similar transition to before,
        // this time an explicit bound is assigned.
        .with_transition(
            "s1",
            Transition {
                to_location: "s1".into(),
                enable: |_, letter| *letter == b'a',
                update: 4.into(),

                // All bounds are inclusive.
                //
                // Justification:
                // Any bound over u32 defined with strict operators can also be defined using inclusive operators.
                //
                // What does None imply?
                // A lower bound of None is equivalent to 0 <= r1.
                // Conversely, an upper bound of None is equivalent to r1 <= u32::MAX.
                bound: TransitionBound {
                    lower: None,
                    upper: Some(7),
                },
                ..Default::default()
            },
        )
        .with_transition(
            "s1",
            Transition {
                to_location: "s0".into(),
                enable: |_, letter| *letter == b'd',

                // Notice that the bound is converted to a strict representation.
                // We express r1 >= 8 as r1 > 7.
                bound: TransitionBound {
                    lower: Some(8),
                    upper: None,
                },
                ..Default::default()
            },
        )
        .with_transition(
            "s1",
            Transition {
                to_location: "s1".into(),
                enable: |_, letter| *letter == b't',
                ..Default::default()
            },
        )
        .with_accepting("s1")
        .build();

    // let monitor = Monitor::from_machine(machine).unwrap();

    dbg!(machine.find_sink_state_intervals_from(StateInterval {
        location: "s0".into(),
        interval: TransitionBound::unbounded(),
    }));
}
