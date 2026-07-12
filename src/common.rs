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

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct StaticData {
    pub name: String,
    pub visibility: Visibility,
    pub is_read_only: bool,
    pub alignment: u8,
    pub bytes: Vec<u8>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExternDecl {
    pub name: String,
    pub param: Vec<RawType>,
    pub ret: RawType,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Ordering {
    Relaxed,
    Release,
    Acquire,
    AcqRel,
    SeqCst,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq, Eq, Hash)]
pub enum SyncScope {
    Group,
    Cluster,
    Device,
    #[default]
    System,
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
