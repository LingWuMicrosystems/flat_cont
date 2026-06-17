use alloc::{string::String, vec::Vec};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use crate::common::{
    AtomicRMWCode, DataId, ExternDecl, ExternId, ICond, Opcode, RawType, StaticData,
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
        effect_args: Vec<(String, NodeId)>,
        common_args: Vec<NodeId>,
        target: ContId,
    },
    Branch {
        effect_args: Vec<(String, NodeId)>,
        common_args: Vec<NodeId>,
        cond: NodeId,
        then_target: ContId,
        else_target: ContId,
    },
    Switch {
        effect_args: Vec<(String, NodeId)>,
        common_args: Vec<NodeId>,
        case: NodeId,
        targets: Vec<ContId>,
    },
    Return {
        effect_args: Vec<(String, NodeId)>,
        common_args: Vec<NodeId>,
    },
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Node {
    Const(u64, RawType),
    DataRef(DataId),
    ExternRef(ExternId),
    ContRef(ContId),
    // load store
    Load {
        data_type: RawType,
        effect_state: NodeId,
        addr: NodeId,
        signed: bool,
    },
    Store {
        data_type: RawType,
        effect_state: NodeId,
        addr: NodeId,
        value: NodeId,
    },
    AtomicCAS {
        data_type: RawType,
        effect_state: NodeId,
        addr: NodeId,
        old: NodeId,
        new: NodeId,
    },
    AtomicRMW {
        data_type: RawType,
        effect_state: NodeId,
        addr: NodeId,
        value: NodeId,
        operator: AtomicRMWCode,
    },
    // Compute
    GEP(RawType, NodeId, SmallVec<[NodeId; 3]>),
    Select(NodeId, NodeId, NodeId),
    Icmp(ICond, NodeId, NodeId),
    // Fcmp(FCond, NodeId, NodeId),
    Compute(Opcode, SmallVec<[NodeId; 4]>),
    Proj(NodeId, u8),
    TokenMerge(Vec<NodeId>),
    Call {
        target: ContId,
        effect_args: Vec<(String, NodeId)>,
        args: Vec<NodeId>,
    },
}
