struct Interpreter<'a> {
    pub bytes: &'a [u8],
}

impl<'a> Interpreter<'a> {
    pub fn do_work(&mut self) {
        // ...
    }
}

#[ouroboros::self_referencing]
struct InterpreterHolder {
    bytes: Vec<u8>,
    #[borrows(bytes)]
    #[covariant]
    interpreter: Interpreter<'this>,
}

fn evi() -> InterpreterHolder {
    InterpreterHolderBuilder {
        bytes: vec![1, 2, 3, 4],
        interpreter_builder: |bytes| Interpreter { bytes },
    }
    .build()
}

fn my_fn() {
    let instr = vec![0, 1, 0, 1];

    let mut holder: Option<InterpreterHolder> = None;

    for ins in instr {
        match ins {
            0 => {
                holder = Some(evi());
            }
            _ => holder
                .as_mut()
                .unwrap()
                .with_mut(|fields| fields.interpreter.do_work()),
        }
    }
}
