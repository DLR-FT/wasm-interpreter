// TODO what does it mean if the column is non zero, but the line is?

use core::error::Error;
use std::{eprintln, path};

use gimli::{Dwarf, EndianSlice};
use std::prelude::rust_2024::*;
use wasmparser::{Parser, Payload};

pub fn report_source_lines(wasm_bytecode: &[u8], execution_trace: impl Iterator<Item = u64>) {
    let cur = Parser::new(0);
    let mut offset: u64 = 0;

    let mut custom_sections = std::collections::HashMap::new();
    for thing in cur.parse_all(wasm_bytecode) {
        match thing.unwrap() {
            Payload::CustomSection(csr) => {
                custom_sections.insert(csr.name(), csr.data());
            }
            Payload::CodeSectionStart { range, .. } => {
                offset = range.start.try_into().unwrap();
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
    eprintln!("{dwarf:#?}");

    let thing = dwarf.units().next().unwrap().unwrap();

    eprintln!("DEBUG HERE\n{:#?}", &thing);

    let mut line_lookup_cacher = DwarfAddr2LineLookup::new(dwarf);
    eprintln!("loading");
    line_lookup_cacher.load().unwrap();
    eprintln!("looking up");

    // let mut already_seen_pc = std::collections::BTreeSet::new();
    for pc in execution_trace {
        if
        //already_seen_pc.insert(pc)
        //&&
        let Some(scl) = line_lookup_cacher.lookup(pc - offset) {
            eprintln!("pc = {pc:#x?} <- {scl}");
        }
    }
}

struct DwarfAddr2LineLookup<R: gimli::Reader> {
    dwarf: Dwarf<R>,
    pc_to_source_file_cache: std::collections::HashMap<u64, (usize, u64, u64)>,
    source_file_list: Vec<path::PathBuf>,
}

#[derive(Debug)]
pub struct SourceCodeLocation<'a> {
    path: &'a path::Path,
    line: u64,
    col: u64,
}

impl<'a> std::fmt::Display for SourceCodeLocation<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let Self { path, line, col } = self;
        write!(f, "{}:{line} column {col}", path.display())
    }
}

impl<R: gimli::Reader> DwarfAddr2LineLookup<R>
where
    <R as gimli::Reader>::Offset: core::fmt::LowerHex + std::hash::Hash,
{
    pub fn new(dwarf: Dwarf<R>) -> Self {
        Self {
            dwarf,
            pc_to_source_file_cache: Default::default(),
            source_file_list: Default::default(),
        }
    }

    pub fn lookup<'a>(&'a self, pc: u64) -> Option<SourceCodeLocation<'a>> {
        self.pc_to_source_file_cache
            .get(&pc)
            .map(|(idx, line, col)| SourceCodeLocation {
                path: self.source_file_list[*idx].as_path(),
                line: *line,
                col: *col,
            })
    }

    pub fn load(&mut self) -> Result<(), std::boxed::Box<dyn Error>> {
        // temporary map to accelerate the `source location -> index` lookup
        let mut temp_lut = std::collections::HashMap::new();

        // iterate over the compilation units
        let mut iter = self.dwarf.units();
        while let Some(header) = iter.next()? {
            let unit = self.dwarf.unit(header)?;
            let unit = unit.unit_ref(&self.dwarf);

            // get the line program for the compilation unit
            if let Some(program) = unit.line_program.clone() {
                // get the compilation directory of the unit
                let comp_dir = if let Some(ref dir) = unit.comp_dir {
                    path::PathBuf::from(dir.to_string_lossy()?.into_owned())
                } else {
                    path::PathBuf::new()
                };

                // Iterate over the line program rows.
                let mut rows = program.rows();
                while let Some((header, row)) = rows.next_row()? {
                    if row.end_sequence() {
                        // End of sequence indicates a possible gap in addresses.
                        // eprintln!("{:#x} end-sequence", row.address());
                    } else {
                        // Determine the path. Real applications should cache this for performance.
                        let mut path = path::PathBuf::new();
                        if let Some(file) = row.file(header) {
                            path.clone_from(&comp_dir);

                            // The directory index 0 is defined to correspond to the compilation unit directory.
                            if file.directory_index() != 0
                                && let Some(dir) = file.directory(header)
                            {
                                path.push(unit.attr_string(dir)?.to_string_lossy()?.as_ref());
                            }

                            path.push(
                                unit.attr_string(file.path_name())?
                                    .to_string_lossy()?
                                    .as_ref(),
                            );
                        }

                        // Determine line/column. DWARF line/column is never 0, so we use that
                        // but other applications may want to display this differently.
                        let line = match row.line() {
                            Some(line) => line.get(),
                            None => 0,
                        };
                        let column = match row.column() {
                            gimli::ColumnType::LeftEdge => 0,
                            gimli::ColumnType::Column(column) => column.get(),
                        };

                        let current_pc_location_idx =
                            *temp_lut.entry(path.clone()).or_insert_with(|| {
                                let idx = self.source_file_list.len();
                                self.source_file_list.push(path.clone());
                                idx
                            });

                        let pc = row.address();
                        self.pc_to_source_file_cache
                            .insert(pc, (current_pc_location_idx, line, column));

                        // eprintln!(
                        //     "{:#x} {}:{}:{}",
                        //     row.address(),
                        //     path.display(),
                        //     line,
                        //     column
                        // );
                    }
                }
            }
        }
        Ok(())
    }
}
