use alloc::{boxed::Box, string::String, vec::Vec};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct DataId(pub u32);
#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct ExternId(pub u32);

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum RawType {
    Token,
    Ptr,
    Scalar(u16),
    Vector { elem_bits: u16 },
    Array(u32, Box<RawType>),
    Struct(Vec<RawType>),
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

/// A value that can appear as an operand to a node or terminator.
///
/// Follows LLVM's model: constants and parameters are first-class values
/// that don't need to live in the node pool before they can be referenced.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Value {
    /// Reference to a node's result (index into the node pool).
    Node(u32),
    /// Compile-time constant.
    Const(u64, RawType),
    /// Value parameter (index into the continuation/region's params list).
    Param(usize),
    /// Effect parameter (index into the continuation/region's effects list).
    Effect(usize),
}

impl Value {
    /// Extract the node index, panicking if this is not a `Node`.
    #[track_caller]
    pub fn as_node(&self) -> u32 {
        match self {
            Value::Node(n) => *n,
            _ => panic!("expected Value::Node, got {:?}", self),
        }
    }
}
