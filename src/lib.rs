#![no_std]
extern crate alloc;

pub mod common;

use alloc::vec::Vec;

use crate::common::{RawType, Sign};

#[derive(Debug)]
pub struct FCIR {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub u32);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ContId(pub u32);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FlatContGraph {
    pub nodes: Vec<Node>,
    pub continuations: Vec<Continuation>,
    pub entry: ContId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Continuation {
    pub param_count: usize,
    pub contained_nodes: Vec<NodeId>,
    pub terminator: Terminator,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Node {
    Const(i64),
    Param(usize),
    Proj(NodeId, u16),
    // load store
    Load {
        effect_state: NodeId,
        data_type: RawType,
        addr: NodeId,
    },
    Store {
        effect_state: NodeId,
        data_type: RawType,
        addr: NodeId,
        value: NodeId,
    },

    // Compute
    Add(NodeId, NodeId),
    Sub(NodeId, NodeId),
    Mul(NodeId, NodeId),
    Div(Sign, NodeId, NodeId),
    Rem(Sign, NodeId, NodeId),

    And(NodeId, NodeId),
    Or(NodeId, NodeId),
    Xor(NodeId, NodeId),
    Not(NodeId),

    ShiftLeft(NodeId, NodeId),
    ShiftRight(Sign, NodeId, NodeId),

    CmpEq(NodeId, NodeId),
    CmpNe(NodeId, NodeId),

    CmpGt(Sign, NodeId, NodeId),
    CmpLe(Sign, NodeId, NodeId),

    Min(Sign, NodeId, NodeId),
    Max(Sign, NodeId, NodeId),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
