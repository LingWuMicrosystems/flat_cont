#![no_std]
extern crate alloc;

use alloc::{string::String, vec::Vec};
use serde::{Deserialize, Serialize};

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
    pub target_arch: u8,

    pub static_data: Vec<StaticData>,
    pub externals: Vec<ExternDecl>,

    pub nodes: Vec<Node>,
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
pub struct ExternDecl(pub String);

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct FlatContinuation {
    pub name: Option<String>,
    pub param: Vec<RawType>,
    // pub node_start: NodeId,
    // pub node_count: NodeId,
    pub contained_nodes: Vec<NodeId>,
    pub terminator: Terminator,
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RawType {
    Ptr,
    Scalar(u16), // bit
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Terminator {
    Jump {
        target: ContId,
        args: Vec<NodeId>,
    },
    Branch {
        cond: NodeId,
        then_target: ContId,
        else_target: ContId,
        args: Vec<NodeId>,
    },
    Switch {
        case: NodeId,
        targets: Vec<ContId>,
        args: Vec<NodeId>,
    },
    Return(Vec<NodeId>),
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Node {
    Const(i64),
    DataRef(DataId),
    ContRef(ContId),
    // load store
    Load {
        data_type: RawType,
        signed: bool,
        effect_state: NodeId,
        addr: NodeId,
    },
    Store {
        data_type: RawType,
        signed: bool,
        effect_state: NodeId,
        addr: NodeId,
        value: NodeId,
    },
    // Compute
    Icmp(ICond, NodeId, NodeId),

    Unary(Opcode, NodeId),
    Binary(Opcode, NodeId, NodeId),
    Ternary(Opcode, NodeId, NodeId, NodeId),
    Call {
        target: ContId,
        args: Vec<NodeId>,
    },
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

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum Opcode {
    Invalid = 0,
    Add,
    Sub,
    Mul,
    Div,
    DivSign,
    Rem,
    RemSign,

    And,
    Or,
    Xor,
    Not,

    Shl,
    Shr,
    ShrSign,
    Eq,
    Ne,

    Lt,
    Le,
    Gt,
    Ge,

    Minu,
    Mins,
    Maxu,
    Maxs,

    Select,
}
