use alloc::{string::String, vec::Vec};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use crate::common::{
    AtomicRMWCode, DataId, ExternDecl, ExternId, ICond, Opcode, RawType, StaticData, Value,
};

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct NodeId(pub u32);
#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct ContId(pub u32);

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct FlatContGraph {
    pub name: Option<String>,

    pub static_data: Vec<StaticData>,
    pub externals: Vec<ExternDecl>,

    pub continuations: Vec<FlatContinuation>,
}

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct ExportedSymbol {
//     pub name: String,
//     pub cont: ContId,
// }

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct FlatContinuation {
    pub effects: Vec<String>,
    pub params: Vec<RawType>,
    pub nodes: Vec<Node>,
    pub terminator: Terminator,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Terminator {
    Jump {
        effect_args: Vec<(String, Value)>,
        common_args: Vec<Value>,
        target: ContId,
    },
    Branch {
        effect_args: Vec<(String, Value)>,
        common_args: Vec<Value>,
        cond: Value,
        then_target: ContId,
        else_target: ContId,
    },
    Switch {
        effect_args: Vec<(String, Value)>,
        common_args: Vec<Value>,
        case: Value,
        targets: Vec<ContId>,
    },
    Return {
        effect_args: Vec<(String, Value)>,
        common_args: Vec<Value>,
    },
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Node {
    // TODO: these will eventually become Value variants once RVSDG transitions
    Const(u64, RawType),
    DataRef(DataId),
    ExternRef(ExternId),
    ContRef(ContId),
    // load store
    Load {
        data_type: RawType,
        effect_state: Value,
        addr: Value,
        signed: bool,
    },
    Store {
        data_type: RawType,
        effect_state: Value,
        addr: Value,
        value: Value,
    },
    AtomicCAS {
        data_type: RawType,
        effect_state: Value,
        addr: Value,
        old: Value,
        new: Value,
    },
    AtomicRMW {
        data_type: RawType,
        effect_state: Value,
        addr: Value,
        value: Value,
        operator: AtomicRMWCode,
    },
    // Compute
    GEP(RawType, Value, SmallVec<[Value; 3]>),
    Select(Value, Value, Value),
    Icmp(ICond, Value, Value),
    // Fcmp(FCond, Value, Value),
    Compute(Opcode, SmallVec<[Value; 4]>),
    Proj(Value, u8),
    TokenMerge(Vec<Value>),
    Call {
        target: ContId,
        effect_args: Vec<(String, Value)>,
        args: Vec<Value>,
    },
}
