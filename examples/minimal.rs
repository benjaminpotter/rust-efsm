use rust_efsm::{MachineBuilder, Transition};

fn main() {
    // Prints INFO events to STDOUT.
    tracing_subscriber::fmt::init();

    // Define a new machine via MachineBuilder that accepts i32 as input and operates on i32 as data.
    let machine = MachineBuilder::<i32, i32>::new()
        // Add a single self-looping transition.
        .with_transition(
            "Count",
            Transition {
                // Here we indicate the self-loop.
                s_out: "Count".into(),

                // These two function can be used to selectively take this transition based on the current input and data.
                validate: |_| true,
                enable: |_| true,

                // Everytime we are in this state and recieve an input, add it to counter.
                update: |counter, input| counter + input,

                // Fill out the rest of the transition with default values.
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
