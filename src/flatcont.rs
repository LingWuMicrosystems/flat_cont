use alloc::{string::String, vec::Vec};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use crate::common::{
    AtomicRMWCode, DataId, ExternDecl, ExternId, FunctionVisibility, ICond, Opcode, Ordering,
    RawType, StaticData, SyncScope,
};

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct ContId(pub u32);
#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct FunId(pub u32);
#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct ValueId(pub u32);
#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct EffectId(pub u32);

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ValueRef {
    /// Value parameter (index into FlatContinuation.params).
    Param(usize),
    /// Reference to a local node within a FlatContinuation.
    Value(ValueId),
    /// Compile-time constant.
    Const(u64, RawType),

    DataRef(DataId),
    ExternRef(ExternId),
    FunRef(FunId),

    /// The sole ordinary value produced by an effect node.
    ValueFromEffect(EffectId),
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EffectPath {
    Param(usize),
    Effect(EffectId),
    NewEffect(Option<String>),
    EffectMerge(Vec<EffectId>),
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct FlatContGraph {
    pub name: Option<String>,

    /// The program-entry function. `Some(fun)` marks the function laid out first
    /// in the code segment; `None` denotes a statically-linkable library graph.
    #[serde(default)]
    pub entry: Option<FunId>,

    pub static_data: Vec<StaticData>,
    pub externals: Vec<ExternDecl>,

    pub functions: Vec<Function>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Function {
    /// Function-level visibility / export name (`Private` by default).
    #[serde(default)]
    pub visibility: FunctionVisibility,
    pub continuations: Vec<FlatContinuation>,
    pub entry: ContId,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct FlatContinuation {
    pub effects_params: Vec<String>,
    pub params: Vec<RawType>,
    pub effects: Vec<Effect>,
    pub dataflow: Vec<Value>,
    pub terminator: Terminator,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Terminator {
    Jump {
        effect_args: Vec<EffectPath>,
        common_args: Vec<ValueRef>,
        target: ContId,
    },
    Branch {
        effect_args: Vec<EffectPath>,
        common_args: Vec<ValueRef>,
        cond: ValueRef,
        then_target: ContId,
        else_target: ContId,
    },
    Switch {
        effect_args: Vec<EffectPath>,
        common_args: Vec<ValueRef>,
        case: ValueRef,
        targets: Vec<ContId>,
    },
    Return {
        effect_args: Vec<EffectPath>,
        common_args: Vec<ValueRef>,
    },
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Value {
    GEP(RawType, ValueRef, SmallVec<[ValueRef; 3]>),
    Select(ValueRef, ValueRef, ValueRef),
    Icmp(ICond, ValueRef, ValueRef),
    // Fcmp(FCond, FlatContValue, FlatContValue),
    Compute(Opcode, SmallVec<[ValueRef; 4]>),
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct AtomicLS {
    pub ordering: Ordering,
    pub sync_scope: SyncScope,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Effect {
    // load store
    Load {
        data_type: RawType,
        effect_state: EffectPath,
        atomic: Option<AtomicLS>,
        addr: ValueRef,
        // signed: bool,
    },
    Store {
        data_type: RawType,
        effect_state: EffectPath,
        atomic: Option<AtomicLS>,
        addr: ValueRef,
        value: ValueRef,
    },

    AtomicCAS {
        data_type: RawType,
        effect_state: EffectPath,
        ordering: Ordering,
        sync_scope: SyncScope,
        addr: ValueRef,
        old: ValueRef,
        new: ValueRef,
        weak: bool,
    },
    AtomicRMW {
        data_type: RawType,
        effect_state: EffectPath,
        ordering: Ordering,
        sync_scope: SyncScope,
        addr: ValueRef,
        value: ValueRef,
        operator: AtomicRMWCode,
    },
    Call {
        target: ValueRef,
        effect_args: Vec<EffectPath>,
        args: Vec<ValueRef>,
    },
    Alloc(RawType),
}
