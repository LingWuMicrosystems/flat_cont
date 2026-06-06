#![no_std]
extern crate alloc;

use alloc::{boxed::Box, string::String, vec::Vec};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct NodeId(pub u32);
#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct ContId(pub u32);
#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct DataId(pub u32);
#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct ExternId(pub u32);

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FlatContGraph {
    pub static_data: Vec<StaticData>,
    pub externals: Vec<ExternDecl>,

    pub continuations: Vec<FlatContinuation>,
    pub exports: Vec<ExportedSymbol>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedSymbol {
    pub name: String,
    pub cont: ContId,
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Visibility {
    Private,
    Public,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticData {
    pub name: String,
    pub visibility: Visibility,
    pub is_read_only: bool,
    pub alignment: u8,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternDecl {
    pub name: String,
    pub param: Vec<RawType>,
    pub ret: RawType,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct FlatContinuation {
    pub name: Option<String>,
    pub effects: Vec<String>,
    pub params: Vec<RawType>,
    pub nodes: Vec<Node>,
    pub terminator: Terminator,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum RawType {
    Token,
    Ptr,
    Scalar(u16),
    Vector { elem_bits: u16 },
    Array(u32, Box<RawType>),
    Struct(Vec<RawType>),
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
    Return(Vec<NodeId>),
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Node {
    Const(u64, RawType),
    DataRef(DataId),
    ExternRef(ExternId),
    ContRef(ContId),
    Param(usize),
    EffectParam(usize),
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
    Call {
        target: ContId,
        effect_args: Vec<(String, NodeId)>,
        args: Vec<NodeId>,
    },
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum AtomicRMWCode {
    Swap,
    Add,
    Sub,
    And,
    Nand,
    Or,
    Xor,
    Max,
    Min,
    MaxSign,
    MinSign,
    Fadd,
    Fsub,
    Fmax,
    Fmin,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Opcode {
    Add,
    Sub,
    Mul,
    Div,
    DivSign,
    Rem,
    RemSign,
    ShiftLeft,
    ShiftRight,
    ShiftRightSign,
    And,
    Or,
    Xor,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum ICond {
    Eq,
    Ne,
    Gt,
    Ge,
    Lt,
    Le,
    GtSign,
    GeSign,
    LtSign,
    LeSign,
}
