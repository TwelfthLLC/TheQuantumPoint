use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortDirection {
    In,
    Out,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortKind {
    Exec,
    Data,
}

#[derive(Debug, Clone, Copy)]
pub struct PortSpec {
    pub name: &'static str,
    pub direction: PortDirection,
    pub kind: PortKind,
}

impl PortSpec {
    pub const fn exec_in(name: &'static str) -> Self {
        Self {
            name,
            direction: PortDirection::In,
            kind: PortKind::Exec,
        }
    }

    pub const fn exec_out(name: &'static str) -> Self {
        Self {
            name,
            direction: PortDirection::Out,
            kind: PortKind::Exec,
        }
    }

    pub const fn data_in(name: &'static str) -> Self {
        Self {
            name,
            direction: PortDirection::In,
            kind: PortKind::Data,
        }
    }

    pub const fn data_out(name: &'static str) -> Self {
        Self {
            name,
            direction: PortDirection::Out,
            kind: PortKind::Data,
        }
    }
}

pub const PORTS_START: &[PortSpec] = &[PortSpec::exec_out("exec")];

pub const PORTS_IF: &[PortSpec] = &[
    PortSpec::exec_in("exec"),
    PortSpec::exec_out("true"),
    PortSpec::exec_out("false"),
    PortSpec::exec_out("done"),
];

pub const PORTS_DEFAULT: &[PortSpec] = &[PortSpec::exec_in("exec"), PortSpec::exec_out("exec")];

pub const PORTS_LOOP: &[PortSpec] = &[
    PortSpec::exec_in("exec"),
    PortSpec::exec_out("body"),
    PortSpec::exec_out("done"),
];

pub const PORTS_SWITCH: &[PortSpec] = &[
    PortSpec::exec_in("exec"),
    PortSpec::exec_out("case1"),
    PortSpec::exec_out("case2"),
    PortSpec::exec_out("case3"),
    PortSpec::exec_out("case4"),
    PortSpec::exec_out("case5"),
    PortSpec::exec_out("case6"),
    PortSpec::exec_out("default"),
    PortSpec::exec_out("done"),
];

pub const PORTS_TRY: &[PortSpec] = &[
    PortSpec::exec_in("exec"),
    PortSpec::exec_out("try"),
    PortSpec::exec_out("catch"),
    PortSpec::exec_out("done"),
];

pub const PORTS_RETURN: &[PortSpec] = &[PortSpec::exec_in("exec")];
