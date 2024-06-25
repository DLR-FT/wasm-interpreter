macro_rules! def_idx_types {
    ($($name:ident),*) => {
        $(
            /// <https://webassembly.github.io/spec/core/binary/modules.html#indices>
            pub type $name = usize;
        )*
    };
}

// #[allow(dead_code)]
def_idx_types!(TypeIdx, FuncIdx, TableIdx, MemIdx, GlobalIdx, /* ElemIdx, DataIdx, */ LocalIdx/* , LabelIdx */);
