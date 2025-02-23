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
You will also find useful examples are embedded into the documentation.
They are not provided here to minimize the chance of an outdated example.

