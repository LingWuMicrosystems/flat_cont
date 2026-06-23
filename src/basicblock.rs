use alloc::{string::String, vec::Vec};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use crate::common::*;

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct NodeId(pub u32);
#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct ContId(pub u32);

/// SSA value in the BasicBlockGraph IR.
///
/// `Node` refers to a node in the global node pool (`BasicBlockGraph.nodes`).
/// `Param` / `Effect` refer to basic-block parameter slots.
/// `Const` is a compile-time immediate.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Value {
    /// Reference to a node in the global pool.
    Node(NodeId),
    /// Compile-time constant.
    Const(u64, RawType),
    /// Value parameter (slot index within a basic block).
    Param(usize),
    /// Effect parameter (slot index within a basic block).
    Effect(usize),
}

impl Value {
    /// Extract the node index, panicking if this is not a `Node`.
    #[track_caller]
    pub fn as_node(&self) -> NodeId {
        match self {
            Value::Node(n) => *n,
            _ => panic!("expected BbValue::Node, got {:?}", self),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicBlockGraph {
    pub name: Option<String>,

    pub static_data: Vec<StaticData>,
    pub externals: Vec<ExternDecl>,

    pub nodes: Vec<Node>,
    pub bbs: Vec<BasicBlock>,
}

impl BasicBlockGraph {
    pub fn add_bb(&mut self) -> ContId {
        let id = ContId(self.bbs.len() as u32);
        self.bbs.push(BasicBlock {
            node_ids: Vec::new(),
            terminator: Terminator::Return {
                effect_args: Vec::new(),
                common_args: Vec::new(),
            },
        });
        id
    }

    pub fn add_node(&mut self, node: Node) -> NodeId {
        let id = NodeId(self.nodes.len() as u32);
        self.nodes.push(node);
        id
    }

    pub fn add_to_block(&mut self, bb: ContId, nid: NodeId) {
        self.bbs[bb.0 as usize].node_ids.push(nid);
    }

    pub fn set_term(&mut self, bb: ContId, term: Terminator) {
        self.bbs[bb.0 as usize].terminator = term;
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicBlock {
    pub node_ids: Vec<NodeId>,
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
    Param(usize),
    EffectParam(usize),
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
    // Fcmp(FCond, BbValue, BbValue),
    Compute(Opcode, SmallVec<[Value; 4]>),
    Proj(Value, u8),
    TokenMerge(Vec<Value>),
    Alloc(RawType),
    Call {
        target: Value,
        effect_args: Vec<(String, Value)>,
        args: Vec<Value>,
    },
}
