use rust_efsm::{Machine, MachineBuilder, Transition, TransitionBound};
use std::collections::HashSet;
use std::u32;
use tracing::info;

fn main() {
    // Prints INFO events to STDOUT.
    tracing_subscriber::fmt::init();

    // Define a machine following the specification from the first example in the assignment.
    // Machine operates on a u32 register with u8 (ASCII) input.
    let machine = MachineBuilder::<u32, u8>::new()
        // Define a transition from s0 to s1,
        // where input is a 'c',
        // there are no bounds on the transition,
        // and the transition will increment the u32 register.
        .with_transition(
            "s0",
            Transition {
                s_out: "s1".into(),
                validate: |letter| *letter == b'c',
                update: |r1, _| r1 + 1,

                // All bounds are strict.
                //
                // Justification:
                // Any bound over u32 defined with inclusive operators can also be defined using strict operators.
                // In the edge case (e.g., 0 <= r1), we simply ignore the bound by using None.
                // In other words, a lower bound of None is equivalent to 0 <= r1.
                // Conversely, an upper bound of None is equivalent to r1 <= u32::MAX.
                bound: TransitionBound::unbounded(),

                // Here we explicitly set the bounds.
                // Since many transitions may not have bounds, we consider this the default.
                // If a member is not explicitly set in the constructor, ..Default::default will
                // fill it with the default value.
                ..Default::default()
            },
        )
        .with_transition(
            "s0",
            Transition {
                s_out: "s0".into(),
                validate: |letter| *letter == b'b',

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
                validate: |letter| *letter == b'a',
                enable: |r1| *r1 < 8,
                update: |r1, _| r1 + 4,

                // Here we explicitly set the bounds.
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
                s_out: "s0".into(),
                validate: |letter| *letter == b'd',
                enable: |r1| *r1 >= 8,
                bound: TransitionBound {
                    lower: None,
                    upper: Some(9),
                },
                ..Default::default()
            },
        )
        .with_transition(
            "s1",
            Transition {
                s_out: "s1".into(),
                validate: |letter| *letter == b't',
                ..Default::default()
            },
        )
        .with_accepting("s1")
        .build();

    let non_empty = non_empty_states(&machine, "s0");
    info!("found non-empty states {:?}", non_empty);

    assert!(non_empty == HashSet::<String>::from(["s0".into(), "s1".into()]))
}

fn non_empty_states(machine: &Machine<u32, u8>, s_init: &str) -> HashSet<String> {
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
    machine: &Machine<u32, u8>,
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
                walk_transitions(
                    machine,
                    non_empty,
                    path.clone(),
                    transition.s_out.clone(),
                    intersection,
                    depth + 1,
                );
            }
        }
    }
}
