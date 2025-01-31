This project provides a Rust library for defining and executing extended finite state machines.

### Installing

#### First, install Rust:
You can find instructions on the rust-lang [website](https://www.rust-lang.org/tools/install).

#### Second, clone this library:
```
git clone https://github.com/benjaminpotter/rust-efsm.git
cd rust-efsm
```

#### Third, try an example:
```
cargo run --example not_spawn_until_init
```

You should not be required to install any dependencies manually.
Cargo should resolve any required crates automatically.
It references the Cargo.toml file found in the project root for this information.
If you are running into missing dependencies, this is probably because cargo cannot find the TOML file.
Ensure you have cloned the entire repository.

### Documentation
After you have successfully executed an example, take a look at the documentation.
The documentation for this project follows the Rust documentation style.
The functionality of other documentation tools you may be familiar with, such as Doxygen, is built into cargo.
Cargo extracts documentation comments from the source code and generates pretty looking HTML.

#### To view the documentation in your browser:
```
cargo doc --open
```

### Usage
Please reference the examples for more detailed information about usage.

#### Here is a minimal example:
```
use rust_efsm::{MachineBuilder, Transition};

fn main() {
    // Prints INFO events to STDOUT.
    tracing_subscriber::fmt::init();

    // Define a new machine via MachineBuilder that accepts i32 as input and operates on i32 as data.
    let machine = MachineBuilder::<i32, i32>::new()

        // Add a single self-looping transition.
        .with_transition("Count", Transition::new(

                // These two function can be used to selectively take this transition based on the current input and data.
                |_| { true },
                |_| { true },

                // Here we indicate the self-loop.
                "Count",

                // Everytime we are in this state and recieve an input, add it to counter.
                |counter, input| { counter + input }))

        // Always accept.
        .with_accepting("Count")

        // Return a new machine as defined above.
        .build();

    // Execute the machine on the input sequence <1, 2, 3>.
    assert!(machine.exec("Count", vec![1, 2, 3]));
}
```
