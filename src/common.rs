#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RawType {
    Token,
    Ptr,
    Int(u8),
    Float(FloatType),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FloatType {
    F16,
    F32,
    F64,
    BF16,
    Posit32,
    Posit64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Sign {
    Signed,
    UnSign,
}
