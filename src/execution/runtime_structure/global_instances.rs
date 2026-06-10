use crate::{GlobalType, Value};

#[derive(Debug)]
pub struct GlobalInst {
    pub ty: GlobalType,
    /// Must be of the same type as specified in `ty`
    pub value: Value,
}
