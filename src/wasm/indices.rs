macro_rules! def_idx_types {
    ($($name:ident),*) => {
        $(
            /// <https://webassembly.github.io/spec/core/binary/modules.html#indices>
            #[derive(Copy, Clone, Debug)]
            pub struct $name(pub u32);
        )*
    };
}

def_idx_types!(TypeIdx, FuncIdx, TableIdx, MemIdx, GlobalIdx, ElemIdx, DataIdx, LocalIdx, LabelIdx);
