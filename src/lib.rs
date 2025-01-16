use std::collections::{HashMap, HashSet, VecDeque};
use tracing::{info, warn, error};

#[derive(Clone, Copy, Debug)]
struct Input {
    i1: bool,
    i2: u64,
}

#[derive(Debug)]
struct RegisterFile {
    r1: u64,
}

type Validate = fn(&Input) -> bool;
type Enable = fn(&RegisterFile) -> bool;
type Update = fn(RegisterFile, &Input) -> RegisterFile;

struct Transition {
    validate: Validate,
    enable: Enable,
    s_out: String,
    update: Update,
}

impl Transition {
    pub fn new(validate: Validate, enable: Enable, s_out: &str, update: Update) -> Self {
       Transition { 
           validate,
           enable, 
           s_out: s_out.into(), 
           update 
       }
    }
}

struct Machine {
    states: HashMap<String, Vec<Transition>>,
    accepting: HashSet<String>,
}

impl Machine {
    pub fn new(states: HashMap<String, Vec<Transition>>, accepting: HashSet<String>) -> Self {
        Machine { states, accepting } 
    }

    pub fn exec(self, s_init: &str, is: Vec<Input>) -> bool {
        info!("executing input sequence");

        let mut rf = RegisterFile { r1: 0 };
        let mut s: String = s_init.into();

        for i in is {
            info!("received input '{:?}'", i);
 
            let next = match self.states.get(&s) {
                Some(ts) => {
                    let mut next: Option<&Transition> = None;
                    for t in ts {
                        if (t.validate)(&i) && (t.enable)(&rf) {
                            if !next.is_none() {
                                panic!(">1 possible transition from state '{}'", s);
                            }

                            next = Some(t);
                        }
                    }                     
                    next
                },
                None => None, 
            };

            if let Some(next) = next {
                info!("found transition from state '{}' to state '{}'", s, next.s_out);
                rf = (next.update)(rf, &i);
                s = next.s_out.clone();
            } else {
                warn!("no valid transition for this input");
            }
        }
        
        info!("reached end of input in state '{}' with register file '{:?}'", s, rf);
        self.accepting.contains(&s)
    }
}

struct MachineBuilder {
    states: HashMap<String, Vec<Transition>>,
    accepting: HashSet<String>,
}

impl MachineBuilder {
    pub fn new() -> Self {
        MachineBuilder { states: HashMap::new(), accepting: HashSet::new(), }
    }

    pub fn with_transition(mut self, s_in: &str, t: Transition) -> Self {
        info!("add transition from state '{}' to state '{}'", s_in, t.s_out);
        self.states.entry(s_in.into()).or_insert(Vec::new()).push(t);
        self 
    }

    pub fn with_accepting(mut self, s: &str) -> Self {
        self.accepting.insert(s.into());
        self
    }

    pub fn build(self) -> Machine {
        info!("build machine with {} states", self.states.keys().len());
        Machine::new(self.states, self.accepting)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        tracing_subscriber::fmt::init();

        let machine = MachineBuilder::new()
            .with_transition("s0", Transition::new(|i| { i.i1 }, |rf|{ true }, "s1", |mut rf, &i|{ rf.r1 = i.i2; rf }))
            .with_transition("s0", Transition::new(|i| { !i.i1 }, |rf|{ true }, "s0", |rf, _|{ rf }))
            .with_transition("s1", Transition::new(|i| { i.i1 }, |rf|{ rf.r1 <= 7 }, "s1", |mut rf, _|{ rf.r1 += 4; rf }))
            .with_transition("s1", Transition::new(|i| { i.i1 }, |rf|{ rf.r1 >= 8 }, "s0", |rf, _|{ rf }))
            .with_transition("s1", Transition::new(|i| { !i.i1 }, |rf|{ true }, "s1", |rf, _|{ rf }))
            .with_accepting("s1")
            .build();

        assert!(machine.exec("s0", vec![
                Input {i1: true, i2: 6},
                Input {i1: false, i2: 0},
                Input {i1: true, i2: 0},
                Input {i1: false, i2: 0},
                Input {i1: false, i2: 0},
                Input {i1: false, i2: 0},
                Input {i1: false, i2: 0},
                Input {i1: false, i2: 0},
                Input {i1: true, i2: 0},
        ]));
    }
}
