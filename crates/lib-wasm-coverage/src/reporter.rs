// TODO what does it mean if the column is non zero, but the line is?

use core::error::Error;
use std::{eprintln, path};

use gimli::{Dwarf, EndianSlice};
use wasmparser::{Parser, Payload};

#[derive(Debug,Clone,PartialEq, Eq)]
pub struct DwarfAddr2LineLookup {
    pub pc_to_source_file_cache: std::collections::HashMap<u64, (usize, u64, u64, bool)>,
    pub source_file_list: std::vec::Vec<path::PathBuf>,
}

pub fn parse_dwarf(wasm_bytecode: &[u8]) -> DwarfAddr2LineLookup {
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

    let mut result = DwarfAddr2LineLookup::new();
    result.load(&dwarf, offset);
    result
}

#[derive(Debug)]
pub struct SourceCodeLocation<'a> {
    pub path: &'a path::Path,
    pub line: u64,
    pub col: u64,
}

impl<'a> std::fmt::Display for SourceCodeLocation<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let Self { path, line, col } = self;
        write!(f, "{}:{line} column {col}", path.display())
    }
}

impl DwarfAddr2LineLookup
{
    pub fn new() -> Self {
        Self {
            pc_to_source_file_cache: Default::default(),
            source_file_list: Default::default(),
        }
    }

    pub fn lookup<'a>(&'a self, pc: u64) -> Option<SourceCodeLocation<'a>> {
        self.pc_to_source_file_cache
            .get(&pc)
            .map(|(idx, line, col, _)| SourceCodeLocation {
                path: self.source_file_list[*idx].as_path(),
                line: *line,
                col: *col,
            })
    }

    pub fn load<R: gimli::Reader>(&mut self, dwarf: &Dwarf<R>, offset: u64) -> Result<(), std::boxed::Box<dyn Error>> {
        // temporary map to accelerate the `source location -> index` lookup
        let mut temp_lut = std::collections::HashMap::new();

        // iterate over the compilation units
        let mut iter = dwarf.units();
        while let Some(header) = iter.next()? {
            let unit = dwarf.unit(header)?;
            let unit = unit.unit_ref(dwarf);

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
                            .insert(pc+offset, (current_pc_location_idx, line, column, false));

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
