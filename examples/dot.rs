use rust_efsm::{MachineBuilder, Transition, Update};
use std::fs::write;
use std::{fmt, fmt::Display};

#[derive(Debug, PartialEq)]
enum Ap {
    Init,
    Spawn,
    Other,
}

#[derive(Debug)]
enum UpdateType {
    Identity,
    SetInit,
}

struct Updater(UpdateType);

impl From<UpdateType> for Updater {
    fn from(ty: UpdateType) -> Self {
        Updater(ty)
    }
}

impl Default for Updater {
    fn default() -> Self {
        Updater(UpdateType::Identity)
    }
}

impl Update for Updater {
    type D = bool;
    type I = Ap;

    fn update(&self, flag: Self::D, _input: &Self::I) -> Self::D {
        match self.0 {
            UpdateType::Identity => flag,
            UpdateType::SetInit => true,
        }
    }
}

impl Display for Updater {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

fn main() {
    tracing_subscriber::fmt::init();
    let machine = MachineBuilder::<bool, Ap, Updater>::new()
        .with_transition(
            "start",
            Transition {
                s_out: "start".into(),
                enable: |flag, i| !flag && *i == Ap::Init,
                enable_hint: Some("not init and input=init".into()),
                update: UpdateType::SetInit.into(),
                ..Default::default()
            },
        )
        .with_transition(
            "start",
            Transition {
                s_out: "start".into(),
                enable: |flag, i| !flag && *i == Ap::Other,
                enable_hint: Some("not init and input=other".into()),
                ..Default::default()
            },
        )
        .with_transition(
            "start",
            Transition {
                s_out: "start".into(),
                enable: |flag, _| *flag,
                enable_hint: Some("init".into()),
                ..Default::default()
            },
        )
        .with_transition(
            "start",
            Transition {
                s_out: "end".into(),
                enable: |flag, i| !flag && *i == Ap::Spawn,
                enable_hint: Some("not init and input=spawn".into()),
                ..Default::default()
            },
        )
        .with_transition(
            "end",
            Transition {
                s_out: "end".into(),
                ..Default::default()
            },
        )
        .with_accepting("start")
        .build();

    write("out.gv", machine.get_dot_buffer()).unwrap();
}
