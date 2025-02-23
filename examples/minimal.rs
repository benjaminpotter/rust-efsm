use rust_efsm::{MachineBuilder, Transition, Update};

// Define an update routine for our counter.
#[derive(Default)]
struct CounterUpdate;

impl Update for CounterUpdate {
    // These types should match the D and I types passed to the builder.
    type D = i32;
    type I = i32;

    // This routine is called anytime a transition is taken.
    fn update(&self, data: Self::D, input: &Self::I) -> Self::D {
        // Here we accumulate inputs, *counting* the total.
        data + input
    }
}

fn main() {
    // Prints INFO events to STDOUT.
    tracing_subscriber::fmt::init();

    // Define a new machine via MachineBuilder that accepts i32 as input and operates on i32 as data.
    let machine = MachineBuilder::<i32, i32, CounterUpdate>::new()
        // Add a single self-looping transition.
        .with_transition(
            "Count",
            Transition {
                // Here we indicate the self-loop.
                s_out: "Count".into(),
                ..Default::default()
            },
        )
        // Always accept.
        .with_accepting("Count")
        // Return a new machine as defined above.
        .build();

    // Execute the machine on the input sequence <1, 2, 3>.
    assert!(machine.exec("Count", 0, vec![1, 2, 3]));
}
