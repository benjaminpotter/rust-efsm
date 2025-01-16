use std::collections::{HashMap, HashSet, VecDeque};
use tracing::{error, info, warn};

type Validate<I> = fn(&I) -> bool;
type Enable<D> = fn(&D) -> bool;
type Update<D, I> = fn(D, &I) -> D;

struct Transition<D, I> {
    validate: Validate<I>,
    enable: Enable<D>,
    s_out: String,
    update: Update<D, I>,
}

impl<D, I> Transition<D, I> {
    pub fn new(
        validate: Validate<I>,
        enable: Enable<D>,
        s_out: &str,
        update: Update<D, I>,
    ) -> Self {
        Transition {
            validate,
            enable,
            s_out: s_out.into(),
            update,
        }
    }
}

struct Machine<D, I> {
    states: HashMap<String, Vec<Transition<D, I>>>,
    accepting: HashSet<String>,
}

impl<D: Default, I> Machine<D, I> {
    pub fn new(states: HashMap<String, Vec<Transition<D, I>>>, accepting: HashSet<String>) -> Self {
        Machine { states, accepting }
    }

    pub fn exec(self, s_init: &str, is: Vec<I>) -> bool {
        info!("executing input sequence");

        let mut rf = D::default();
        let mut s: String = s_init.into();

        for i in is {
            info!("received input");

            let next = match self.states.get(&s) {
                Some(ts) => {
                    let mut next: Option<&Transition<D, I>> = None;
                    for t in ts {
                        if (t.validate)(&i) && (t.enable)(&rf) {
                            if !next.is_none() {
                                panic!(">1 possible transition from state '{}'", s);
                            }

                            next = Some(t);
                        }
                    }
                    next
                }
                None => None,
            };

            if let Some(next) = next {
                info!(
                    "found transition from state '{}' to state '{}'",
                    s, next.s_out
                );
                rf = (next.update)(rf, &i);
                s = next.s_out.clone();
            } else {
                warn!("no valid transition for this input");
            }
        }

        info!("reached end of input in state '{}'", s);
        self.accepting.contains(&s)
    }
}

struct MachineBuilder<D, I> {
    states: HashMap<String, Vec<Transition<D, I>>>,
    accepting: HashSet<String>,
}

impl<D: Default, I> MachineBuilder<D, I> {
    pub fn new() -> Self {
        MachineBuilder {
            states: HashMap::new(),
            accepting: HashSet::new(),
        }
    }

    pub fn with_transition(mut self, s_in: &str, t: Transition<D, I>) -> Self {
        info!(
            "add transition from state '{}' to state '{}'",
            s_in, t.s_out
        );
        self.states.entry(s_in.into()).or_insert(Vec::new()).push(t);
        self
    }

    pub fn with_accepting(mut self, s: &str) -> Self {
        self.accepting.insert(s.into());
        self
    }

    pub fn build(self) -> Machine<D, I> {
        info!("build machine with {} states", self.states.keys().len());
        Machine::new(self.states, self.accepting)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy)]
    struct Input {
        i1: bool,
        i2: u64,
    }

    #[derive(Default)]
    struct RegisterFile {
        r1: u64,
    }

    #[test]
    fn it_works() {
        tracing_subscriber::fmt::init();

        let machine = MachineBuilder::new()
            .with_transition(
                "s0",
                Transition::new(
                    |i: &Input| i.i1,
                    |rf: &RegisterFile| true,
                    "s1",
                    |mut rf, &i| {
                        rf.r1 = i.i2;
                        rf
                    },
                ),
            )
            .with_transition(
                "s0",
                Transition::new(
                    |i: &Input| !i.i1,
                    |rf: &RegisterFile| true,
                    "s0",
                    |rf, _| rf,
                ),
            )
            .with_transition(
                "s1",
                Transition::new(
                    |i: &Input| i.i1,
                    |rf: &RegisterFile| rf.r1 <= 7,
                    "s1",
                    |mut rf, _| {
                        rf.r1 += 4;
                        rf
                    },
                ),
            )
            .with_transition(
                "s1",
                Transition::new(
                    |i: &Input| i.i1,
                    |rf: &RegisterFile| rf.r1 >= 8,
                    "s0",
                    |rf, _| rf,
                ),
            )
            .with_transition(
                "s1",
                Transition::new(
                    |i: &Input| !i.i1,
                    |rf: &RegisterFile| true,
                    "s1",
                    |rf, _| rf,
                ),
            )
            .with_accepting("s1")
            .build();

        assert!(!machine.exec(
            "s0",
            vec![
                Input { i1: true, i2: 6 },
                Input { i1: false, i2: 0 },
                Input { i1: true, i2: 0 },
                Input { i1: false, i2: 0 },
                Input { i1: false, i2: 0 },
                Input { i1: false, i2: 0 },
                Input { i1: false, i2: 0 },
                Input { i1: false, i2: 0 },
                Input { i1: true, i2: 0 },
            ]
        ));
    }
}
