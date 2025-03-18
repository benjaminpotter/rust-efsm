use rust_efsm::{Machine, MachineBuilder, Transition, TransitionBound, Update};
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
                s_out: "s1".into(),
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
                s_out: "s0".into(),
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
                s_out: "s1".into(),
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
                s_out: "s0".into(),
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
                s_out: "s1".into(),
                enable: |_, letter| *letter == b't',
                ..Default::default()
            },
        )
        .with_accepting("s1")
        .build();

    let non_empty = non_empty_states(&machine, "s0");
    info!("found non-empty states {:?}", non_empty);

    assert!(non_empty == HashSet::<String>::from(["s0".into(), "s1".into()]));

    machine.exec("s0", 2, vec![b'c', b'a', b't']);
}

fn non_empty_states(machine: &Machine<u32, u8, AddUpdate>, s_init: &str) -> HashSet<String> {
    // TODO: Check that the machine is actually non-deterministic.
    // The following algorithm is only correct in that case.

    let mut non_empty = machine.get_accepting();
    walk_transitions(
        machine,
        &mut non_empty,
        vec![],
        s_init.into(),
        TransitionBound::unbounded(),
        0,
    );

    non_empty
}

fn walk_transitions(
    machine: &Machine<u32, u8, AddUpdate>,
    non_empty: &mut HashSet<String>,
    mut path: Vec<String>,
    curr: String,
    interval: TransitionBound<u32>,
    depth: u32,
) {
    // How do we handle cycles?
    // Recursion limit?
    if depth > 1000 {
        return;
    }

    // If current is already in non-empty, then add all of path to non_empty.
    if non_empty.contains(&curr) {
        info!("found path {:?} to accepting state", path);
        for state in path {
            non_empty.insert(state);
        }

        return;
    }

    // If it isn't check for transitions out of this current state.
    if let Some(transitions) = machine.get_transitions(&curr) {
        // Append the current node to the path before recursing.
        path.push(curr);

        // Loop over all transitions on current,
        for transition in transitions {
            // Apply the intersection of the range,
            // If the intersection is non-zero,
            if let Some(intersection) = transition.bound.intersect(&interval) {
                let interval = intersection.shifted_by(transition.update.amount);

                walk_transitions(
                    machine,
                    non_empty,
                    path.clone(),
                    transition.s_out.clone(),
                    interval,
                    depth + 1,
                );
            }
        }
    }
}
