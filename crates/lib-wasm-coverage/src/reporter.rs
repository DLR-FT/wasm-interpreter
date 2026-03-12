use core::error::Error;
use std::eprintln;

use gimli::EndianSlice;
use wasmparser::{Chunk, Parser, Payload};

fn report_source_lines(wasm_bytecode: &[u8], execution_trace: &[usize]) {
    let mut cur = Parser::new(0);

    let mut custom_sections = std::collections::HashMap::new();
    for thing in cur.parse_all(wasm_bytecode) {
        match thing.unwrap() {
            Payload::CustomSection(csr) => {
                custom_sections.insert(csr.name(), csr.data());
            }

            _ => {
                eprintln!("ignoring");
            }
        }
    }

    let dwarf = gimli::Dwarf::load::<_, std::boxed::Box<dyn Error>>(|section_id| {
        let data = custom_sections
            .get(section_id.name())
            .copied()
            .unwrap_or(&[]);
        Ok(EndianSlice::new(data, gimli::LittleEndian))
    })
    .unwrap();

    // TODO continue
    eprintln!("{dwarf:?}");
}
